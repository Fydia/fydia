use std::convert::TryFrom;

use fydia_struct::{
    channel::ChannelId,
    permission::{Permission, Permissions},
    roles::Role,
    user::UserId,
};
use fydia_utils::async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use super::{
    basic_model::BasicModel,
    channel::SqlChannelId,
    delete, insert,
    user::{SqlUser, UserFrom},
};

#[async_trait]
pub trait PermissionSql {
    async fn by_role(
        channelid: &ChannelId,
        role: &Role,
        db: &DatabaseConnection,
    ) -> Result<Permissions, String>;

    async fn by_user(
        channelid: &ChannelId,
        user: &UserId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, String>;

    async fn by_channel(
        channelid: &ChannelId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, String>;

    async fn insert(&self, db: &DatabaseConnection) -> Result<(), String>;

    async fn delete(mut self, db: &DatabaseConnection) -> Result<(), String>;
}

#[async_trait]
impl PermissionSql for Permission {
    async fn by_role(
        channelid: &ChannelId,
        role: &Role,
        db: &DatabaseConnection,
    ) -> Result<Permissions, String> {
        let result = entity::permission::role::Entity::find()
            .filter(entity::permission::role::Column::Role.eq(role.id.get_id_cloned()?))
            .filter(entity::permission::role::Column::Channel.eq(channelid.id.as_str()))
            .all(db)
            .await
            .map_err(|error| error.to_string())?;

        let mut vec = Vec::new();
        for i in result {
            vec.push(i.to_struct(db).await?);
        }

        Ok(Permissions::new(vec))
    }

    async fn by_user(
        channelid: &ChannelId,
        user: &UserId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, String> {
        let result = entity::permission::user::Entity::find()
            .filter(entity::permission::user::Column::User.eq(user.0.clone().get_id()?))
            .filter(entity::permission::user::Column::Channel.eq(channelid.id.as_str()))
            .all(db)
            .await
            .map_err(|error| error.to_string())?;

        let mut vec = Vec::new();

        for i in result {
            vec.push(i.to_struct(db).await?);
        }

        let channel = channelid.channel(db).await?;

        let roles = user
            .to_user(db)
            .await
            .ok_or(String::from("No user"))?
            .roles(&channel.parent_id, db)
            .await?;

        for i in roles.iter() {
            vec.append(&mut Self::by_role(channelid, i, db).await?.get());
        }

        Ok(Permissions::new(vec))
    }

    async fn by_channel(
        channelid: &ChannelId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, String> {
        let result = entity::permission::user::Entity::find()
            .filter(entity::permission::user::Column::Channel.eq(channelid.id.as_str()))
            .all(db)
            .await
            .map_err(|error| error.to_string())?;

        let mut vec = Vec::new();
        for i in result {
            vec.push(i.to_struct(db).await?);
        }

        let result = entity::permission::role::Entity::find()
            .filter(entity::permission::role::Column::Channel.eq(channelid.id.as_str()))
            .all(db)
            .await
            .map_err(|error| error.to_string())?;

        for i in result {
            vec.push(i.to_struct(db).await?);
        }

        Ok(Permissions::new(vec))
    }

    async fn insert(&self, db: &DatabaseConnection) -> Result<(), String> {
        match self.permission_type {
            fydia_struct::permission::PermissionType::Role(_) => {
                let am = entity::permission::role::ActiveModel::try_from(self.clone())?;

                insert(am, db).await?;
            }
            fydia_struct::permission::PermissionType::User(_) => {
                let am = entity::permission::user::ActiveModel::try_from(self.clone())?;

                insert(am, db).await?;
            }
        }

        Ok(())
    }

    async fn delete(mut self, db: &DatabaseConnection) -> Result<(), String> {
        match &self.permission_type {
            fydia_struct::permission::PermissionType::Role(_) => {
                let am = entity::permission::role::ActiveModel::try_from(&self)?;

                delete(am, db).await?;
            }
            fydia_struct::permission::PermissionType::User(_) => {
                let am = entity::permission::user::ActiveModel::try_from(&self)?;

                delete(am, db).await?;
            }
        }

        drop(self);

        Ok(())
    }
}
