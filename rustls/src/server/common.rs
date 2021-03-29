use crate::hash_hs;
use crate::key;
use crate::kx;
use crate::msgs::handshake::SessionID;

pub struct HandshakeDetails {
    pub transcript: hash_hs::HandshakeHash,
    pub session_id: SessionID,
}

impl HandshakeDetails {
    pub fn new() -> HandshakeDetails {
        HandshakeDetails {
            transcript: hash_hs::HandshakeHash::new(),
            session_id: SessionID::empty(),
        }
    }
}

pub struct ServerKxDetails {
    pub kx: kx::KeyExchange,
}

impl ServerKxDetails {
    pub fn new(kx: kx::KeyExchange) -> ServerKxDetails {
        ServerKxDetails { kx }
    }
}

pub struct ClientCertDetails {
    pub cert_chain: Vec<key::Certificate>,
}

impl ClientCertDetails {
    pub fn new(chain: Vec<key::Certificate>) -> ClientCertDetails {
        ClientCertDetails { cert_chain: chain }
    }
}
