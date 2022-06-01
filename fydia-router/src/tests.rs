#[cfg(test)]
mod libtest {
    use crate::*;
    use fydia_config::DatabaseConfig;
    use std::net::SocketAddr;

    pub fn get_sqlite() -> Config {
        Config {
            instance: fydia_config::InstanceConfig {
                domain: "".to_string(),
            },
            server: fydia_config::ServerConfig {
                ip: "0.0.0.0".to_string(),
                port: 4000,
            },
            database: DatabaseConfig::new(
                "fydia_test",
                0,
                "",
                "",
                "fydia_test",
                fydia_config::DatabaseType::Sqlite,
            ),
        }
    }

    pub async fn get_router() -> Router {
        let config = get_sqlite();
        let db = super::super::get_database_connection(&config.database)
            .await
            .unwrap();
        super::super::get_axum_router(
            db,
            &config.instance,
            &config.format_ip(),
            *&config.server.port as u16,
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    pub async fn tests() {
        env_logger::builder()
            .is_test(true)
            .default_format()
            .filter_level(log::LevelFilter::Warn)
            .try_init()
            .unwrap();

        let listener =
            std::net::TcpListener::bind("127.0.0.1:8000".parse::<SocketAddr>().unwrap()).unwrap();

        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(get_router().await.into_make_service())
                .await
                .unwrap();
        });
        std::thread::sleep(Duration::from_secs(2));
        let mut fydia_test = fydia_libtest::Tests::from_file("./tests.toml").unwrap();
        fydia_test.run().await;
    }
}
