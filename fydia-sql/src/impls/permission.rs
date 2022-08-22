use std::convert::TryFrom;

use fydia_struct::{
    channel::ChannelId,
    permission::{Permission, Permissions},
    roles::RoleId,
    server::ServerId,
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
    async fn of_role_in_channel(
        channelid: &ChannelId,
        role: &RoleId,
        db: &DatabaseConnection,
    ) -> Result<Permission, String>;

    async fn of_user_in_channel(
        channelid: &ChannelId,
        user: &UserId,
        db: &DatabaseConnection,
    ) -> Result<Permission, String>;
    async fn of_user_with_role_in_channel(
        channelid: &ChannelId,
        user: &UserId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, String>;
    async fn of_user(
        user: &UserId,
        serverid: &ServerId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, String>;
    async fn of_channel(
        channelid: &ChannelId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, String>;
    async fn insert(&self, db: &DatabaseConnection) -> Result<(), String>;
    async fn update_value(self, db: &DatabaseConnection) -> Result<Permission, String>;
    async fn delete(mut self, db: &DatabaseConnection) -> Result<(), String>;
}

#[async_trait]
impl PermissionSql for Permission {
    async fn of_user(
        user: &UserId,
        serverid: &ServerId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, String> {
        let user = user.to_user(db).await?;
        let roles = user.roles(serverid, db).await?;
        let mut vec = Vec::new();

        for role in roles {
            vec.push(Permission::role(role.id, None, role.server_permission));
        }

        Ok(Permissions::new(vec))
    }

    async fn of_role_in_channel(
        channelid: &ChannelId,
        roleid: &RoleId,
        db: &DatabaseConnection,
    ) -> Result<Permission, String> {
        entity::permission::role::Entity::find()
            .filter(entity::permission::role::Column::Role.eq(roleid.get_id_cloned()?))
            .filter(entity::permission::role::Column::Channel.eq(channelid.id.as_str()))
            .one(db)
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "No role permission".to_string())?
            .to_struct(db)
            .await
    }

    async fn of_user_in_channel(
        channelid: &ChannelId,
        user: &UserId,
        db: &DatabaseConnection,
    ) -> Result<Permission, String> {
        entity::permission::user::Entity::find()
            .filter(entity::permission::user::Column::User.eq(user.0.clone().get_id()?))
            .filter(entity::permission::user::Column::Channel.eq(channelid.id.as_str()))
            .one(db)
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "No user permission".to_string())?
            .to_struct(db)
            .await
    }

    async fn of_user_with_role_in_channel(
        channelid: &ChannelId,
        user: &UserId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, String> {
        let channel = channelid.channel(db).await?;
        let user = user.to_user(db).await?;
        let roles = user.roles(&channel.parent_id, db).await?;

        let mut vec = Vec::new();

        for i in roles.iter() {
            if let Ok(perm) = Self::of_role_in_channel(channelid, &i.id, db).await {
                vec.push(perm);
            }
        }

        vec.push(Self::of_user_in_channel(channelid, &user.id, db).await?);

        return Ok(Permissions::new(vec));
    }
    async fn of_channel(
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

    async fn update_value(self, db: &DatabaseConnection) -> Result<Permission, String> {
        let channelid = self
            .channelid
            .clone()
            .ok_or_else(|| String::from("No channelid"))?
            .id;

        match &self.permission_type {
            fydia_struct::permission::PermissionType::Role(role) => {
                let am = entity::permission::role::ActiveModel::try_from(&self)?;

                entity::permission::role::Entity::update(am)
                    .filter(entity::permission::role::Column::Channel.eq(channelid.as_str()))
                    .filter(entity::permission::role::Column::Role.eq(role.get_id_cloned()?))
                    .exec(db)
                    .await
                    .map(|_| ())
                    .map_err(|error| error.to_string())?;
            }
            fydia_struct::permission::PermissionType::User(user) => {
                let am = entity::permission::user::ActiveModel::try_from(&self)?;

                entity::permission::user::Entity::update(am)
                    .filter(entity::permission::user::Column::Channel.eq(channelid.as_str()))
                    .filter(entity::permission::user::Column::User.eq(user.0.get_id_cloned()?))
                    .exec(db)
                    .await
                    .map(|_| ())
                    .map_err(|error| error.to_string())?;
            }
        }

        Ok(self)
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
