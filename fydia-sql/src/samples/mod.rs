use fydia_struct::{instance::Instance, server::Server, user::User};

use crate::{
    impls::{server::SqlServer, user::SqlUser},
    sqlpool::DbConnection,
};

pub async fn insert_samples(db: &DbConnection) {
    warn!("Insert Sample Values");

    let mut user = User::new("user", "user@sample.com", "user", Instance::default());
    if let Err(error) = user.insert_user_and_update(db).await {
        error!(error);
    }

    let server = Server::new("server_default", user.id.clone());
    if let Err(error) = server.insert_server(db).await {
        error!(error);
    }

    if let Err(error) = user.insert_server(server.id, db).await {
        error!(error);
    }
}
