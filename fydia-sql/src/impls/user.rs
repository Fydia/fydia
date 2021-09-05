use crate::sqlpool::FydiaPool;
use crate::sqlpool::ToAnyRow;
use async_trait::async_trait;
use fydia_struct::instance::Instance;
use fydia_struct::server::Servers;
use fydia_struct::user::Token;
use fydia_struct::{server::ServerId, user::User};
use fydia_utils::generate_string;
use fydia_utils::hash;
use fydia_utils::verify_password;
use sqlx::Row;

#[async_trait]
pub trait SqlUser {
    async fn get_user_by_email_and_password(
        email: String,
        password: String,
        executor: &FydiaPool,
    ) -> Option<Self>
    where
        Self: Sized;
    async fn get_user_by_id(id: i32, executor: &FydiaPool) -> Option<Self>
    where
        Self: Sized;
    async fn get_user_by_token(token: &Token, executor: &FydiaPool) -> Option<Self>
    where
        Self: Sized;
    async fn update_token(&mut self, executor: &FydiaPool) -> Result<String, String>;
    async fn update_name(&mut self, name: String, executor: &FydiaPool) -> Result<(), String>;
    async fn update_password(
        &mut self,
        clear_password: String,
        executor: &FydiaPool,
    ) -> Result<(), String>;
    async fn insert_server(
        &mut self,
        server_short_id: ServerId,
        executor: &FydiaPool,
    ) -> Result<(), String>;
    async fn insert_user(&self, executor: &FydiaPool) -> Result<(), String>;
    async fn delete_account(&self, executor: &FydiaPool) -> Result<(), String>;
    async fn get_user_message() -> Vec<String>;
}

#[async_trait]
impl SqlUser for User {
    async fn get_user_by_email_and_password(
        email: String,
        password: String,
        executor: &FydiaPool,
    ) -> Option<Self> {
        let rawquery = "SELECT * FROM `User` WHERE email=? LIMIT 1;";
        let request = match executor {
            FydiaPool::Mysql(mysql) => {
                let result = sqlx::query(rawquery).bind(email).fetch_one(mysql).await;

                if let Ok(e) = result {
                    if !verify_password(password, e.get("password")) {
                        None
                    } else {
                        Some(e.to_anyrow())
                    }
                } else {
                    None
                }
            }
            FydiaPool::PgSql(pgsql) => {
                let result = sqlx::query(rawquery).bind(email).fetch_one(pgsql).await;

                if let Ok(e) = result {
                    if !verify_password(password, e.get("password")) {
                        None
                    } else {
                        Some(e.to_anyrow())
                    }
                } else {
                    None
                }
            }
            FydiaPool::Sqlite(sqlite) => {
                let result = sqlx::query(rawquery).bind(email).fetch_one(sqlite).await;

                if let Ok(e) = result {
                    if !verify_password(password, e.get("password")) {
                        None
                    } else {
                        Some(e.to_anyrow())
                    }
                } else {
                    None
                }
            }
        };

        request.map(|i| Self {
            id: i.get("id"),
            name: i.get("name"),
            instance: Instance::default(),
            token: i.get("token"),
            email: i.get("email"),
            password: i.get("password"),
            description: i.get("description"),
            server: Servers(
                serde_json::from_str(i.get::<String, &str>("server").as_str()).unwrap_or_default(),
            ),
        })
    }

    async fn get_user_by_id(id: i32, executor: &FydiaPool) -> Option<Self> {
        let rawquery = "SELECT * FROM `User` WHERE id=? LIMIT 1;";
        let request = match executor {
            FydiaPool::Mysql(mysql) => {
                let result = sqlx::query(rawquery).bind(id).fetch_one(mysql).await;
                if let Ok(e) = result {
                    Some(e.to_anyrow())
                } else {
                    None
                }
            }
            FydiaPool::PgSql(pgsql) => {
                let result = sqlx::query(rawquery).bind(id).fetch_one(pgsql).await;

                if let Ok(e) = result {
                    Some(e.to_anyrow())
                } else {
                    None
                }
            }
            FydiaPool::Sqlite(sqlite) => {
                let result = sqlx::query(rawquery).bind(id).fetch_one(sqlite).await;

                if let Ok(e) = result {
                    Some(e.to_anyrow())
                } else {
                    None
                }
            }
        };

        request.map(|i| Self {
            id: i.get("id"),
            name: i.get("name"),
            instance: Instance::default(),
            token: i.get("token"),
            email: i.get("email"),
            password: i.get("password"),
            description: i.get("description"),
            server: Servers(
                serde_json::from_str(i.get::<String, &str>("server").as_str()).unwrap_or_default(),
            ),
        })
    }

    async fn get_user_by_token(token: &Token, executor: &FydiaPool) -> Option<Self> {
        let rawquery = "SELECT * FROM `User` WHERE token=? LIMIT 1;";
        let request = match executor {
            FydiaPool::Mysql(mysql) => {
                let result = sqlx::query(rawquery).bind(&token.0).fetch_one(mysql).await;
                if let Ok(e) = result {
                    Some(e.to_anyrow())
                } else {
                    None
                }
            }
            FydiaPool::PgSql(pgsql) => {
                let result = sqlx::query(rawquery).bind(&token.0).fetch_one(pgsql).await;

                if let Ok(e) = result {
                    Some(e.to_anyrow())
                } else {
                    None
                }
            }
            FydiaPool::Sqlite(sqlite) => {
                let result = sqlx::query(rawquery).bind(&token.0).fetch_one(sqlite).await;

                if let Ok(e) = result {
                    Some(e.to_anyrow())
                } else {
                    None
                }
            }
        };

        request.map(|i| Self {
            id: i.get("id"),
            name: i.get("name"),
            instance: Instance::new(
                fydia_struct::instance::Protocol::HTTP,
                String::from("localhost"),
                0,
            ),
            token: i.get("token"),
            email: i.get("email"),
            password: i.get("password"),
            description: i.get("description"),
            server: Servers(
                serde_json::from_str(i.get::<String, &str>("server").as_str()).unwrap_or_default(),
            ),
        })
    }

    async fn update_token(&mut self, executor: &FydiaPool) -> Result<String, String> {
        let token = generate_string(30);
        let rawquery = "UPDATE `User` SET token=? WHERE id=?;";

        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&token)
                    .bind(self.id)
                    .execute(mysql)
                    .await
                {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&token)
                    .bind(self.id)
                    .execute(pgsql)
                    .await
                {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&token)
                    .bind(self.id)
                    .execute(sqlite)
                    .await
                {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
        }

        self.token = Some(token.clone());
        Ok(token)
    }

    async fn update_name(&mut self, name: String, executor: &FydiaPool) -> Result<(), String> {
        let rawquery = "UPDATE `User` SET name=? WHERE id=?;";

        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&name)
                    .bind(&self.id)
                    .execute(mysql)
                    .await
                {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&name)
                    .bind(&self.id)
                    .execute(pgsql)
                    .await
                {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&name)
                    .bind(&self.id)
                    .execute(sqlite)
                    .await
                {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
        };

        self.name = name;

        Ok(())
    }

    async fn update_password(
        &mut self,
        clear_password: String,
        executor: &FydiaPool,
    ) -> Result<(), String> {
        let password = hash(clear_password);
        let rawquery = "UPDATE `User` SET password=? WHERE id=?;";

        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&password)
                    .bind(&self.id)
                    .execute(mysql)
                    .await
                {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&password)
                    .bind(&self.id)
                    .execute(pgsql)
                    .await
                {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&password)
                    .bind(&self.id)
                    .execute(sqlite)
                    .await
                {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
        };

        self.password = password;

        Ok(())
    }

    async fn insert_server(
        &mut self,
        server_short_id: ServerId,
        executor: &FydiaPool,
    ) -> Result<(), String> {
        let rawquery = "UPDATE `User` SET server=? WHERE id=?;";
        let server = &mut self.server.0;
        server.push(server_short_id);
        let json = match serde_json::to_string(&server) {
            Ok(json) => json,
            Err(error) => error.to_string(),
        };
        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(json)
                    .bind(&self.id)
                    .execute(mysql)
                    .await
                {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(json)
                    .bind(&self.id)
                    .execute(pgsql)
                    .await
                {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(json)
                    .bind(&self.id)
                    .execute(sqlite)
                    .await
                {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
        };

        Ok(())
    }

    async fn insert_user(&self, executor: &FydiaPool) -> Result<(), String> {
        let rawquery = "INSERT INTO `User`
        (name, token, email, password, server)
        VALUES(?,?, ?, ?, ?);
        ";
        let json = match serde_json::to_string(&Servers(Vec::new())) {
            Ok(json) => json,
            Err(error) => return Err(error.to_string()),
        };

        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&self.name)
                    .bind("")
                    .bind(&self.email)
                    .bind(&self.password)
                    .bind(json)
                    .execute(mysql)
                    .await
                {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&self.name)
                    .bind("")
                    .bind(&self.email)
                    .bind(&self.password)
                    .bind(json)
                    .execute(pgsql)
                    .await
                {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&self.name)
                    .bind("")
                    .bind(&self.email)
                    .bind(&self.password)
                    .bind(json)
                    .execute(sqlite)
                    .await
                {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
        };

        Ok(())
    }

    async fn delete_account(&self, executor: &FydiaPool) -> Result<(), String> {
        let rawquery = "DELETE FROM `User` WHERE id=?;";

        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery).bind(&self.id).execute(mysql).await {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery).bind(&self.id).execute(pgsql).await {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery).bind(&self.id).execute(sqlite).await {
                    return match e.as_database_error() {
                        Some(error) => Err(error.to_string()),
                        None => Err("Cannot get database error".to_string()),
                    };
                }
            }
        };

        Ok(())
    }

    async fn get_user_message() -> Vec<String> {
        Vec::new()
    }
}
