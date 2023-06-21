/// DNS name validation according to RFC1035, but with underscores allowed.
use std::error::Error as StdError;
use std::fmt;

/// A type which encapsulates an owned string that is a syntactically valid DNS name.
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct DnsName(String);

impl<'a> DnsName {
    /// Produce a borrowed `DnsNameRef` from this owned `DnsName`.
    pub fn borrow(&'a self) -> DnsNameRef<'a> {
        DnsNameRef(self.as_ref())
    }

    /// Validate the given bytes are a DNS name if they are viewed as ASCII.
    pub fn try_from_ascii(bytes: &[u8]) -> Result<Self, InvalidDnsNameError> {
        // nb. a sequence of bytes that is accepted by `validate()` is both
        // valid UTF-8, and valid ASCII.
        String::from_utf8(bytes.to_vec())
            .map_err(|_| InvalidDnsNameError)
            .and_then(Self::try_from)
    }
}

impl TryFrom<String> for DnsName {
    type Error = InvalidDnsNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        validate(value.as_bytes())?;
        Ok(Self(value))
    }
}

impl AsRef<str> for DnsName {
    fn as_ref(&self) -> &str {
        AsRef::<str>::as_ref(&self.0)
    }
}

/// A type which encapsulates a borrowed string that is a syntactically valid DNS name.
#[derive(Eq, Hash, PartialEq, Debug)]
pub struct DnsNameRef<'a>(&'a str);

impl<'a> DnsNameRef<'a> {
    /// Copy this object to produce an owned `DnsName`.
    pub fn to_owned(&'a self) -> DnsName {
        DnsName(self.0.to_string())
    }

    /// Copy this object to produce an owned `DnsName`, smashing the case to lowercase
    /// in one operation.
    pub fn to_lowercase_owned(&'a self) -> DnsName {
        DnsName(self.0.to_lowercase())
    }
}

impl<'a> TryFrom<&'a str> for DnsNameRef<'a> {
    type Error = InvalidDnsNameError;

    fn try_from(value: &'a str) -> Result<DnsNameRef<'a>, Self::Error> {
        validate(value.as_bytes())?;
        Ok(DnsNameRef(value))
    }
}

impl<'a> AsRef<str> for DnsNameRef<'a> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

/// The provided input could not be parsed because
/// it is not a syntactically-valid DNS Name.
#[derive(Debug)]
pub struct InvalidDnsNameError;

impl fmt::Display for InvalidDnsNameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid dns name")
    }
}

impl StdError for InvalidDnsNameError {}

fn validate(input: &[u8]) -> Result<(), InvalidDnsNameError> {
    use State::*;
    let mut state = Start;

    /// "Labels must be 63 characters or less."
    const MAX_LABEL_LENGTH: usize = 63;

    for ch in input {
        state = match (state, ch) {
            (Start | Next | Hyphen { .. }, b'.') => return Err(InvalidDnsNameError),
            (Subsequent { .. }, b'.') => Next,
            (Subsequent { len }, _) if len >= MAX_LABEL_LENGTH => return Err(InvalidDnsNameError),
            (Start | Next, b'a'..=b'z' | b'A'..=b'Z') => Subsequent { len: 1 },
            (Subsequent { len }, b'-') => Hyphen { len: len + 1 },
            (
                Subsequent { len } | Hyphen { len },
                b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'-' | b'0'..=b'9',
            ) => Subsequent { len: len + 1 },
            _ => return Err(InvalidDnsNameError),
        };
    }

    if matches!(state, Start | Hyphen { .. }) {
        return Err(InvalidDnsNameError);
    }

    Ok(())
}

enum State {
    Start,
    Next,
    Subsequent { len: usize },
    Hyphen { len: usize },
}

#[cfg(test)]
mod test {
    static TESTS: &[(&'static str, bool)] = &[
        ("", false),
        ("localhost", true),
        ("LOCALHOST", true),
        (".localhost", false),
        ("..localhost", false),
        ("1.2.3.4", false),
        ("127.0.0.1", false),
        ("absolute.", true),
        ("absolute..", false),
        ("multiple.labels.absolute.", true),
        ("foo.bar.com", true),
        ("infix-hyphen-allowed.com", true),
        ("-prefixhypheninvalid.com", false),
        ("suffixhypheninvalid-.com", false),
        ("foo.lastlabelendswithhyphen-", false),
        ("infix_underscore_allowed.com", true),
        ("_prefixunderscoreinvalid.com", false),
        ("labelendswithnumber1.bar.com", true),
        ("xn--bcher-kva.example", true),
        (
            "sixtythreesixtythreesixtythreesixtythreesixtythreesixtythreesix.com",
            true,
        ),
        (
            "sixtyfoursixtyfoursixtyfoursixtyfoursixtyfoursixtyfoursixtyfours.com",
            false,
        ),
    ];

    #[test]
    fn test_validation() {
        for (input, expected) in TESTS {
            println!("test: {:?} expected valid? {:?}", input, expected);
            let name_ref = super::DnsNameRef::try_from(*input);
            assert_eq!(*expected, name_ref.is_ok());
            let name = super::DnsName::try_from(input.to_string());
            assert_eq!(*expected, name.is_ok());
        }
    }

    #[test]
    fn error_is_debug() {
        assert_eq!(
            format!("{:?}", super::InvalidDnsNameError),
            "InvalidDnsNameError"
        );
    }

    #[test]
    fn error_is_display() {
        assert_eq!(
            format!("{}", super::InvalidDnsNameError),
            "invalid dns name"
        );
    }

    #[test]
    fn dns_name_is_debug() {
        let example = super::DnsName::try_from("example.com".to_string()).unwrap();
        assert_eq!(format!("{:?}", example), "DnsName(\"example.com\")");
    }

    #[test]
    fn dns_name_traits() {
        let example = super::DnsName::try_from("example.com".to_string()).unwrap();
        assert_eq!(example, example); // PartialEq

        use std::collections::HashSet;
        let mut h = HashSet::<super::DnsName>::new();
        h.insert(example);
    }

    #[test]
    fn try_from_ascii_rejects_bad_utf8() {
        assert_eq!(
            format!("{:?}", super::DnsName::try_from_ascii(b"\x80")),
            "Err(InvalidDnsNameError)"
        );
    }

    #[test]
    fn dns_name_ref_is_debug() {
        let example = super::DnsNameRef::try_from("example.com").unwrap();
        assert_eq!(format!("{:?}", example), "DnsNameRef(\"example.com\")");
    }

    #[test]
    fn dns_name_ref_traits() {
        let example = super::DnsNameRef::try_from("example.com").unwrap();
        assert_eq!(example, example); // PartialEq

        use std::collections::HashSet;
        let mut h = HashSet::<super::DnsNameRef>::new();
        h.insert(example);
    }
}
