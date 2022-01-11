use fydia_struct::{user::User, instance::Instance};

use crate::{sqlpool::DbConnection, impls::user::SqlUser};

pub async fn insert_samples(db: &DbConnection) {
    warn!("Insert Sample Values");

    let user = User::new("user", "user@sample.com", "user", Instance::default());
    if let Err(error) = user.insert_user(db).await {
        println!("{}", error);
    }





}
