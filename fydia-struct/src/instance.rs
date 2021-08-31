use fydia_crypto::{PrivateKey, PublicKey};
use serde::{Deserialize, Serialize};

#[derive(StateData, Clone)]
pub struct RsaData(pub PrivateKey, pub PublicKey);

#[derive(Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq)]
pub enum Protocol {
    HTTP,
    HTTPS,
}

impl Protocol {
    pub fn format(&self) -> String {
        match self {
            Protocol::HTTP => format!("http://"),
            Protocol::HTTPS => format!("https://"),
        }
    }
    pub fn parse(str: &str) -> Self {
        match str {
            "https://" => Self::HTTPS,
            _ => Self::HTTP,
        }
    }
}

#[derive(Debug, StateData, Serialize, Deserialize, Clone, PartialOrd, PartialEq)]
pub struct Instance {
    pub protocol: Protocol,
    pub domain: String,
    pub port: u16,
}

impl Instance {
    pub fn new(protocol: Protocol, domain: String, port: u16) -> Self {
        Self {
            protocol,
            domain,
            port,
        }
    }

    pub fn from(string: String) -> Option<Self> {
        if let Ok(e) = url::Url::parse(string.as_str()) {
            let protocol = Protocol::parse(e.scheme());
            let domain = e.domain().unwrap().to_string();
            let port = e.port().unwrap_or(8080);
            return Some(Self {
                protocol,
                domain,
                port,
            });
        }

        None
    }

    pub fn format(&self) -> String {
        format!("{}{}:{}", self.protocol.format(), self.domain, self.port)
    }

    pub fn get_public_key(&self) -> Result<PublicKey, ()> {
        match reqwest::blocking::get(format!("{}/api/instance/public_key", self.format())) {
            Ok(res) => match res.text() {
                Ok(string) => match fydia_crypto::pem::get_key_from_string(string) {
                    Some(key) => {
                        return Ok(key);
                    }
                    None => return Err(()),
                },
                Err(_) => return Err(()),
            },
            Err(_) => return Err(()),
        }
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self { protocol: Protocol::HTTP, domain: String::new(), port: 0 }
    }
}
