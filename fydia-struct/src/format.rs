#[derive(Debug, Default, PartialEq)]
pub struct UserFormat {
    pub name: String,
    pub domain: String,
    pub port: Option<u16>,
}

impl UserFormat {
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
        let domain = domain.trim_start_matches("@");
        match reqwest::Url::parse(format!("http://{}", domain).as_str()) {
            Ok(url) => match url.domain() {
                Some(domain) => match url.port() {
                    Some(port) => {
                        return Some(Self {
                            name: username.to_string(),
                            domain: domain.to_string(),
                            port: Some(port),
                        });
                    }
                    None => {
                        return Some(Self {
                            name: username.to_string(),
                            domain: domain.to_string(),
                            port: None,
                        });
                    }
                },
                None => {
                    println!("No domain");
                    None
                }
            },
            Err(e) => {
                println!("{}", e);
                None
            }
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct ServerFormat {
    pub name: String,
    pub domain: String,
    pub port: Option<u16>,
}

impl ServerFormat {
    /// Server$domain.com | Server$domain.com:port
    pub fn from_string<T: Into<String>>(str: T) -> Option<Self> {
        let from: String = str.into();
        let mut n = 0;

        for i in from.char_indices() {
            if i.1 == '$' {
                n = i.0;
            }
        }
        let (username, domain) = from.split_at(n);
        let domain = domain.trim_start_matches("$");
        match reqwest::Url::parse(format!("http://{}", domain).as_str()) {
            Ok(url) => match url.domain() {
                Some(domain) => match url.port() {
                    Some(port) => {
                        return Some(Self {
                            name: username.to_string(),
                            domain: domain.to_string(),
                            port: Some(port),
                        });
                    }
                    None => {
                        return Some(Self {
                            name: username.to_string(),
                            domain: domain.to_string(),
                            port: None,
                        });
                    }
                },
                None => {
                    println!("No domain");
                    None
                }
            },
            Err(e) => {
                println!("{}", e);
                None
            }
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct ChannelFormat {
    pub channel: String,
    pub server: String,
    pub domain: String,
    pub port: Option<u16>,
}

impl ChannelFormat {
    /// Channel#Server$domain.com | Channel#Server$domain.com:port
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
        let domain = domain.trim_start_matches("$");
        match reqwest::Url::parse(format!("http://{}", domain).as_str()) {
            Ok(url) => match url.domain() {
                Some(domain) => match url.port() {
                    Some(port) => {
                        return Some(Self {
                            channel: channelname.to_string(),
                            server: servername.trim_start_matches("#").to_string(),
                            domain: domain.to_string(),
                            port: Some(port),
                        });
                    }
                    None => {
                        return Some(Self {
                            channel: channelname.to_string(),
                            server: servername.trim_start_matches("#").to_string(),
                            domain: domain.to_string(),
                            port: None,
                        });
                    }
                },
                None => {
                    println!("No domain");
                    None
                }
            },
            Err(e) => {
                println!("{}", e);
                None
            }
        }
    }
}
