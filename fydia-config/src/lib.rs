use serde::{Deserialize, Serialize};
use std::process::exit;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum DatabaseType {
    Mysql,
    PgSql,
    Sqlite,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub database_type: DatabaseType,
    pub ip: String,
    port: i16,
    name: String,
    password: String,
    database_name: String,
}

impl DatabaseConfig {
    pub fn new() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            port: 0,
            name: "default".to_string(),
            password: "default".to_string(),
            database_name: "default".to_string(),
            database_type: DatabaseType::Mysql,
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
            DatabaseType::Sqlite => format!("sqlite://{}.sql", self.name),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
            database: DatabaseConfig::new(),
        }
    }
}

impl Config {
    pub fn read_config(parse: String) -> Self {
        toml::from_str(parse.as_str()).expect("Error")
    }
    pub fn write_to(&self, path: String) -> std::io::Result<()> {
        std::fs::write(path.as_str(), self.serialize_to_string().as_str())
    }
    pub fn serialize_to_string(&self) -> String {
        toml::to_string(self).expect("Error")
    }
    pub fn format_ip(&self) -> String {
        format!("{}:{}", self.server.ip, self.server.port)
    }
}

pub fn get_config_or_init() -> Config {
    if let Ok(e) = std::fs::read("config.toml") {
        let read_config = String::from_utf8(e).expect("Error");
        Config::read_config(read_config)
    } else {
        let config = Config::default();
        config.write_to("config.toml".to_string()).expect("Error");
        println!("Change Config with your database config");
        exit(127);
    }
}
