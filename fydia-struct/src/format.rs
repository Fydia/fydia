//! This module is related to Formatted representation of User, Server, Channel

/// `UserFormat` used to represent a `User` as a String over Instance
#[allow(missing_docs)]
#[derive(Debug, Default, PartialEq)]
pub struct UserFormat {
    pub name: String,
    pub domain: String,
    pub port: Option<u16>,
}

impl UserFormat {
    /// Take a generic type that implement `Into<String>`
    /// and return a `Option<UserFormat>`
    ///
    /// Option is None when String cannot be convert as `UserFormat`
    ///
    ///# Examples
    ///```
    /// use fydia_struct::format::UserFormat;
    /// let user_format = UserFormat::from_string("User@domain.com");
    ///
    /// assert_eq!(
    /// user_format,
    /// Some(UserFormat {
    ///    name: "User".to_string(),
    ///    domain: "domain.com".to_string(),
    ///    port: Some(80)
    /// })
    ///);
    /// ```
    /// Example with a port
    ///```
    /// use fydia_struct::format::UserFormat;
    /// let user_format = UserFormat::from_string("User@domain.com:980");
    ///
    /// assert_eq!(
    /// user_format,
    /// Some(UserFormat {
    ///    name: "User".to_string(),
    ///    domain: "domain.com".to_string(),
    ///    port: Some(980)
    /// })
    ///);
    /// ```
    /// User@domain.com | User@domain.com:port
    pub fn from_string<T: Into<String>>(str: T) -> Option<Self> {
        let from: String = str.into();
        let mut n = 0;

        for i in from.char_indices() {
            if i.1 == '@' {
                n = i.0;
            }
        }

        let (username, domain) = from.split_at(n);
        let domain = domain.trim_start_matches('@');
        let url = reqwest::Url::parse(format!("http://{domain}").as_str()).ok()?;
        let domain = url.domain()?;
        let port = url.port_or_known_default();

        Some(Self {
            name: username.to_string(),
            domain: domain.to_string(),
            port,
        })
    }
}

/// `ServerFormat` used to represent a `Server` as a String over Instance
#[allow(missing_docs)]
#[derive(Debug, Default, PartialEq)]
pub struct ServerFormat {
    pub name: String,
    pub domain: String,
    pub port: Option<u16>,
}

impl ServerFormat {
    /// Take a generic type that implement `Into<String>`
    /// and return a `Option<ServerFormat>`
    ///
    /// Option is None when String cannot be convert as `ServerFormat`
    ///
    ///# Examples
    ///```
    /// use fydia_struct::format::ServerFormat;
    /// let server_format = ServerFormat::from_string("Server$domain.com");
    ///
    /// assert_eq!(
    /// server_format,
    /// Some(ServerFormat {
    ///    name: "Server".to_string(),
    ///    domain: "domain.com".to_string(),
    ///    port: Some(80)
    /// })
    ///);
    /// ```
    /// Example with a port
    ///```
    /// use fydia_struct::format::ServerFormat;
    /// let server_format = ServerFormat::from_string("Server$domain.com:980");
    ///
    /// assert_eq!(
    /// server_format,
    /// Some(ServerFormat {
    ///    name: "Server".to_string(),
    ///    domain: "domain.com".to_string(),
    ///    port: Some(980)
    /// })
    ///);
    /// ```
    pub fn from_string<T: Into<String>>(str: T) -> Option<Self> {
        let from: String = str.into();
        let mut n = 0;

        for i in from.char_indices() {
            if i.1 == '$' {
                n = i.0;
            }
        }

        if n == 0 {
            return None;
        }

        let (name, domain) = from.split_at(n);
        let domain = domain.trim_start_matches('$');

        let url = url::Url::parse(format!("http://{domain}").as_str()).ok()?;
        let domain = url.domain()?;
        let port = url.port_or_known_default();

        Some(Self {
            name: name.to_string(),
            domain: domain.to_string(),
            port,
        })
    }
}

/// `ChannelFormat` used to represent a `Channel` as a String over Instance
#[allow(missing_docs)]
#[derive(Debug, Default, PartialEq)]
pub struct ChannelFormat {
    pub channel: String,
    pub server: String,
    pub domain: String,
    pub port: Option<u16>,
}

impl ChannelFormat {
    /// Take a generic type that implement `Into<String>`
    /// and return a `Option<ChannelFormat>`
    ///
    /// Option is None when String cannot be convert as `ChannelFormat`
    ///
    /// # Examples
    ///```
    /// use fydia_struct::format::ChannelFormat;
    /// let channel_format = ChannelFormat::from_string("Channel#Server$domain.com");
    ///
    /// assert_eq!(
    /// channel_format,
    /// Some(ChannelFormat {
    ///    channel: "Channel".to_string(),
    ///    server: "Server".to_string(),
    ///    domain: "domain.com".to_string(),
    ///    port: Some(80)
    /// })
    ///);
    /// ```
    /// Example with a port
    ///```
    /// use fydia_struct::format::ChannelFormat;
    /// let channel_format = ChannelFormat::from_string("Channel#Server$domain.com:980");
    ///
    /// assert_eq!(
    /// channel_format,
    /// Some(ChannelFormat {
    ///    channel: "Channel".to_string(),
    ///    server: "Server".to_string(),
    ///    domain: "domain.com".to_string(),
    ///    port: Some(980)
    /// })
    ///);
    /// ```
    pub fn from_string<T: Into<String>>(str: T) -> Option<Self> {
        let from: String = str.into();
        let mut n = 0;

        for i in from.char_indices() {
            if i.1 == '#' {
                n = i.0;
            }
        }

        let (channelname, part) = from.split_at(n);
        for i in part.char_indices() {
            if i.1 == '$' {
                n = i.0;
            }
        }

        let (servername, domain) = part.split_at(n);
        let domain = domain.trim_start_matches('$');
        let url = reqwest::Url::parse(format!("http://{domain}").as_str()).ok()?;
        let domain = url.domain()?;

        Some(Self {
            channel: channelname.to_string(),
            server: servername.trim_start_matches('#').to_string(),
            domain: domain.to_string(),
            port: url.port_or_known_default(),
        })
    }
}
