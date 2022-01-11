use fydia_struct::{instance::Instance, server::Server, user::User};

use crate::{impls::user::SqlUser, sqlpool::DbConnection};

pub async fn insert_samples(db: &DbConnection) {
    warn!("Insert Sample Values");

    let user = User::new("user", "user@sample.com", "user", Instance::default());
    if let Err(error) = user.insert_user(db).await {
        println!("{}", error);
    }

    Server::default();
}
