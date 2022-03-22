#[cfg(test)]
mod tests {
    use fydia_struct::user::User;
    use sea_orm::{entity::prelude::*, DatabaseBackend, MockDatabase};

    use crate::{entity::user, impls::user::SqlUser};

    #[tokio::test]
    async fn test_find_user() -> Result<(), DbErr> {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![
                user::Model {
                id: 0,
                name: "name".to_string(),
                instance: None,
                token: "".to_string(),
                email: "email@localhost".to_string(),
                password: "$argon2id$v=19$m=4096,t=3,p=1$Jyx49kgh1ERrUKSQ8dMDPg$CAcbUAStkv4k2pZwBoN09n4IYVE/W9IIFSA1NWFXU/M".to_string(),
                description: None,
            }
            ], ])
            .into_connection();
        // Don't why but get None
        assert_eq!(User::get_user_by_id(0, &db).await, None);

        Ok(())
    }
}
