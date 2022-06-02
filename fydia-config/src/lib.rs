use fydia_utils::serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(crate = "fydia_utils::serde")]
pub enum DatabaseType {
    Mysql,
    PgSql,
    Sqlite,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "fydia_utils::serde")]
pub struct DatabaseConfig {
    pub database_type: DatabaseType,
    pub ip: String,
    port: i16,
    name: String,
    password: String,
    database_name: String,
}

impl DatabaseConfig {
    pub fn new<T: Into<String>>(
        ip: T,
        port: i16,
        name: T,
        password: T,
        database_name: T,
        database_type: DatabaseType,
    ) -> Self {
        Self {
            ip: ip.into(),
            port,
            name: name.into(),
            password: password.into(),
            database_name: database_name.into(),
            database_type,
        }
    }

    pub fn format_url(&self) -> String {
        match &self.database_type {
            DatabaseType::Mysql => format!(
                "mysql://{}:{}@{}/{}",
                self.name, self.password, self.ip, self.database_name
            ),
            DatabaseType::PgSql => format!(
                "postgres://{}:{}@{}/{}",
                self.name, self.password, self.ip, self.database_name
            ),
            DatabaseType::Sqlite => format!("sqlite://{}.db", self.ip),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            port: 0,
            name: "default".to_string(),
            password: "default".to_string(),
            database_name: "default".to_string(),
            database_type: DatabaseType::Mysql,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "fydia_utils::serde")]
pub struct ServerConfig {
    pub ip: String,
    pub port: i16,
}

impl ServerConfig {
    pub fn new() -> Self {
        Self {
            ip: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "fydia_utils::serde")]
pub struct InstanceConfig {
    pub domain: String, // URL OR IP
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl InstanceConfig {
    pub fn new() -> Self {
        Self {
            domain: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "fydia_utils::serde")]
pub struct Config {
    pub instance: InstanceConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            instance: InstanceConfig::new(),
            server: ServerConfig::new(),
            database: DatabaseConfig::default(),
        }
    }
}

impl Config {
    /// Read to a file
    ///
    /// # Errors
    /// Return an error if :
    /// * File cannot be read
    pub fn read_config<T: Into<String>>(parse: T) -> Result<Self, String> {
        toml::from_str(parse.into().as_str()).map_err(|error| error.to_string())
    }

    /// Write to a file
    ///
    /// # Errors
    /// Return an error if :
    /// * File cannot be written
    pub fn write_to<T: Into<String>>(&self, path: T) -> Result<(), String> {
        std::fs::write(path.into().as_str(), self.serialize_to_string()?.as_str())
            .map_err(|error| error.to_string())
    }

    /// Serialize `Config` as JSON
    ///
    /// # Errors
    /// Return an error if :
    /// * `Config` cannot be serialized
    pub fn serialize_to_string(&self) -> Result<String, String> {
        toml::to_string(self).map_err(|error| error.to_string())
    }
    pub fn format_ip(&self) -> String {
        format!("{}:{}", self.server.ip, self.server.port)
    }
}

pub fn get_config_or_init() -> Config {
    if let Ok(e) = std::fs::read("config.toml") {
        let read_config = String::from_utf8(e).expect("Error");
        Config::read_config(read_config).expect("Cannot read config")
    } else {
        let config = Config::default();
        config.write_to("config.toml").expect("Error");
        panic!("Change Config with your database config");
    }
}
