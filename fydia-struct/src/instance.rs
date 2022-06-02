//! This module is related to federation and instance

use fydia_crypto::{PrivateKey, PublicKey};
use fydia_utils::serde::{Deserialize, Serialize};
use url::Url;
/// `RsaData` contains `PrivateKey` and `PublicKey` of Instance
#[derive(Clone, Debug)]
pub struct RsaData(pub PrivateKey, pub PublicKey);

/// Enum to know if Instance is in HTTP or HTTPS
#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq)]
#[serde(crate = "fydia_utils::serde")]
pub enum Protocol {
    HTTP,
    HTTPS,
}

impl Protocol {
    /// Convert `Protocol` as a `String`
    pub fn format(&self) -> String {
        match self {
            Protocol::HTTP => "http://".to_string(),
            Protocol::HTTPS => "https://".to_string(),
        }
    }

    /// Take `str` and parse to `Protocol`
    ///
    /// # Examples
    ///
    /// ```
    /// use fydia_struct::instance::Protocol;
    /// let protocol = Protocol::parse("https://");
    ///
    /// assert_eq!(protocol, Protocol::HTTPS)
    /// ```
    ///
    /// ```
    /// use fydia_struct::instance::Protocol;
    /// let protocol = Protocol::parse("http://");
    ///
    /// assert_eq!(protocol, Protocol::HTTP);
    /// ```
    pub fn parse(string: &str) -> Self {
        if string.to_lowercase().starts_with("https") {
            return Self::HTTPS;
        }

        Self::HTTP
    }
}

/// `Instance` represents a Instance of Fydia
#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq)]
#[serde(crate = "fydia_utils::serde")]
pub struct Instance {
    pub protocol: Protocol,
    pub domain: String,
    pub port: u16,
}

impl Instance {
    /// Take `Protocol`, generic value that implements `Into<String>` and u16
    /// to return a new instance
    ///
    /// # Examples
    /// ```
    /// use fydia_struct::instance::Instance;
    /// use fydia_struct::instance::Protocol;
    ///
    /// let instance = Instance::new(Protocol::HTTPS, "domain.com", 80);
    ///
    /// assert_eq!(instance, Instance {protocol: Protocol::HTTPS, domain: "domain.com".to_string(),
    /// port: 80});
    /// ```
    pub fn new<T: Into<String>>(protocol: Protocol, domain: T, port: u16) -> Self {
        Self {
            protocol,
            domain: domain.into(),
            port,
        }
    }
    /// Take `Into<String>` and convert it in `Instance`
    ///
    ///
    /// ```
    /// use fydia_struct::instance::Instance;
    /// use fydia_struct::instance::Protocol;
    ///
    /// let instance = Instance::from("http://domain.com:80");
    ///
    /// assert_eq!(instance, Some(Instance {protocol: Protocol::HTTP, domain: "domain.com".to_string(), port: 80}))
    /// ```
    pub fn from<T: Into<String>>(string: T) -> Option<Self> {
        let url = Url::parse(string.into().as_str()).ok()?;
        let protocol = Protocol::parse(url.scheme());
        if let (Some(domain), Some(port)) = (url.domain(), url.port_or_known_default()) {
            Some(Self {
                protocol,
                domain: domain.to_string(),
                port,
            })
        } else {
            None
        }
    }
    /// Format `Instance` as Url
    ///
    /// # Examples
    ///
    /// ```
    /// use fydia_struct::instance::Instance;
    ///
    /// let instance = Instance::from("https://instance.com:80").unwrap();
    ///
    /// assert_eq!(instance.format(), "https://instance.com:80");
    /// ```
    pub fn format(&self) -> String {
        format!("{}{}:{}", self.protocol.format(), self.domain, self.port)
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
