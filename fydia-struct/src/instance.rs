use fydia_crypto::{PrivateKey, PublicKey};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct RsaData(pub PrivateKey, pub PublicKey);

#[derive(Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq)]
pub enum Protocol {
    HTTP,
    HTTPS,
}

impl Protocol {
    pub fn format(&self) -> String {
        match self {
            Protocol::HTTP => "http://".to_string(),
            Protocol::HTTPS => "https://".to_string(),
        }
    }
    pub fn parse(str: &str) -> Self {
        match str {
            "https://" => Self::HTTPS,
            _ => Self::HTTP,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq)]
pub struct Instance {
    pub protocol: Protocol,
    pub domain: String,
    pub port: u16,
}

impl Instance {
    pub fn new<T: Into<String>>(protocol: Protocol, domain: T, port: u16) -> Self {
        Self {
            protocol,
            domain: domain.into(),
            port,
        }
    }

    pub fn from<T: Into<String>>(string: T) -> Option<Self> {
        let url = url::Url::parse(string.into().as_str()).ok()?;
        let protocol = Protocol::parse(url.scheme());
        if let (Some(domain), Some(port)) = (url.domain(), url.port()) {
            Some(Self {
                protocol,
                domain: domain.to_string(),
                port,
            })
        } else {
            None
        }
    }

    pub fn format(&self) -> String {
        format!("{}{}:{}", self.protocol.format(), self.domain, self.port)
    }

    pub fn get_public_key(&self) -> Result<PublicKey, String> {
        let request = reqwest::blocking::get(format!("{}/api/instance/public_key", self.format()))
            .map_err(|f| f.to_string())?;
        let text = request.text().map_err(|f| f.to_string())?;
        let key = fydia_crypto::pem::get_key_from_string(text)
            .ok_or_else(|| "Can't read the key".to_string())?;

        Ok(key)
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            protocol: Protocol::HTTP,
            domain: String::new(),
            port: 0,
        }
    }
}
