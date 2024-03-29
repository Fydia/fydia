use std::convert::TryFrom;

use super::{
    basic_model::BasicModel,
    channel::SqlChannelId,
    delete, get_set_column, insert,
    user::{SqlUser, UserFrom},
};
use fydia_struct::{
    channel::ChannelId,
    permission::{Permission, PermissionError, Permissions},
    roles::RoleId,
    server::ServerId,
    sqlerror::{GenericError, GenericSqlError},
    user::UserId,
};
use fydia_utils::async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use shared::sea_orm;

#[async_trait]
pub trait PermissionSql {
    async fn of_role_in_channel(
        channelid: &ChannelId,
        role: &RoleId,
        db: &DatabaseConnection,
    ) -> Result<Permission, PermissionError>;

    async fn of_user_in_channel(
        channelid: &ChannelId,
        user: &UserId,
        db: &DatabaseConnection,
    ) -> Result<Permission, PermissionError>;
    async fn of_user_with_role_in_channel(
        channelid: &ChannelId,
        user: &UserId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, PermissionError>;
    async fn of_user(
        user: &UserId,
        serverid: &ServerId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, PermissionError>;
    async fn of_channel(
        channelid: &ChannelId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, PermissionError>;
    async fn insert(&self, db: &DatabaseConnection) -> Result<(), PermissionError>;
    async fn update_value(self, db: &DatabaseConnection) -> Result<Permission, PermissionError>;
    async fn delete(mut self, db: &DatabaseConnection) -> Result<(), PermissionError>;
}

#[async_trait]
impl PermissionSql for Permission {
    async fn of_user(
        user: &UserId,
        serverid: &ServerId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, PermissionError> {
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
    ) -> Result<Permission, PermissionError> {
        let Ok(Some(model)) = entity::permission::role::Entity::find()
            .filter(entity::permission::role::Column::Role.eq(roleid.get_id_cloned()?))
            .filter(entity::permission::role::Column::Channel.eq(channelid.id.as_str()))
            .one(db)
            .await else {
                return Err(PermissionError::CannotGetByChannelAndRole);
            };

        let permission = model.to_struct(db).await?;

        Ok(permission)
    }

    async fn of_user_in_channel(
        channelid: &ChannelId,
        user: &UserId,
        db: &DatabaseConnection,
    ) -> Result<Permission, PermissionError> {
        let Ok(Some(model)) = entity::permission::user::Entity::find()
            .filter(entity::permission::user::Column::User.eq(user.0.clone().get_id_cloned()?))
            .filter(entity::permission::user::Column::Channel.eq(channelid.id.as_str()))
            .one(db)
            .await else {return Err(PermissionError::CannotGetByChannelAndUser)};

        let permission = model.to_struct(db).await?;

        Ok(permission)
    }

    async fn of_user_with_role_in_channel(
        channelid: &ChannelId,
        user: &UserId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, PermissionError> {
        let channel = channelid.channel(db).await?;
        let user = user.to_user(db).await?;
        let roles = user.roles(&channel.parent_id, db).await?;

        let mut vec = Vec::new();

        for i in &roles {
            if let Ok(perm) = Self::of_role_in_channel(channelid, &i.id, db).await {
                vec.push(perm);
            }
        }
        if let Ok(perm) = Self::of_user_in_channel(channelid, &user.id, db).await {
            vec.push(perm);
        }

        return Ok(Permissions::new(vec));
    }
    async fn of_channel(
        channelid: &ChannelId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, PermissionError> {
        let Ok(result) = entity::permission::user::Entity::find()
            .filter(entity::permission::user::Column::Channel.eq(channelid.id.as_str()))
            .all(db)
            .await else {
                return Err( PermissionError::CannotGetByChannel)
            };

        let mut vec = Vec::new();
        for i in result {
            vec.push(i.to_struct(db).await?);
        }

        let Ok(result) = entity::permission::role::Entity::find()
            .filter(entity::permission::role::Column::Channel.eq(channelid.id.as_str()))
            .all(db)
            .await else {
                return Err( PermissionError::CannotGetByChannel)
            };

        for i in result {
            vec.push(i.to_struct(db).await?);
        }

        Ok(Permissions::new(vec))
    }

    async fn insert(&self, db: &DatabaseConnection) -> Result<(), PermissionError> {
        match self.permission_type {
            fydia_struct::permission::PermissionType::Role(_) => {
                let am = entity::permission::role::ActiveModel::try_from(self.clone())?;

                insert(am, db).await?;
            }
            fydia_struct::permission::PermissionType::User(_) => {
                let am = entity::permission::user::ActiveModel::try_from(self.clone())?;

                insert(am, db).await?;
            }

            fydia_struct::permission::PermissionType::Channel(_) => {
                return Err(PermissionError::PermissionTypeError)
            }
        }

        Ok(())
    }

    async fn update_value(self, db: &DatabaseConnection) -> Result<Permission, PermissionError> {
        let channelid = self
            .channelid
            .clone()
            .ok_or_else(|| PermissionError::NoChannelId)?
            .id;

        match &self.permission_type {
            fydia_struct::permission::PermissionType::Role(role) => {
                let am = entity::permission::role::ActiveModel::try_from(&self)?;
                let set_column = get_set_column(&am);
                entity::permission::role::Entity::update(am)
                    .filter(entity::permission::role::Column::Channel.eq(channelid.as_str()))
                    .filter(entity::permission::role::Column::Role.eq(role.get_id_cloned()?))
                    .exec(db)
                    .await
                    .map(|_| ())
                    .map_err(|f| {
                        GenericSqlError::CannotUpdate(GenericError {
                            set_column,
                            error: f.to_string(),
                        })
                    })?;
            }

            fydia_struct::permission::PermissionType::User(user) => {
                let am = entity::permission::user::ActiveModel::try_from(&self)?;
                let set_column = get_set_column(&am);
                entity::permission::user::Entity::update(am)
                    .filter(entity::permission::user::Column::Channel.eq(channelid.as_str()))
                    .filter(entity::permission::user::Column::User.eq(user.0.get_id_cloned()?))
                    .exec(db)
                    .await
                    .map(|_| ())
                    .map_err(|f| {
                        GenericSqlError::CannotUpdate(GenericError {
                            set_column,
                            error: f.to_string(),
                        })
                    })?;
            }
            fydia_struct::permission::PermissionType::Channel(_) => {
                return Err(PermissionError::PermissionTypeError)
            }
        }

        Ok(self)
    }

    async fn delete(mut self, db: &DatabaseConnection) -> Result<(), PermissionError> {
        match &self.permission_type {
            fydia_struct::permission::PermissionType::Role(_) => {
                let am = entity::permission::role::ActiveModel::try_from(&self)?;

                delete(am, db).await?;
            }
            fydia_struct::permission::PermissionType::User(_) => {
                let am = entity::permission::user::ActiveModel::try_from(&self)?;

                delete(am, db).await?;
            }
            fydia_struct::permission::PermissionType::Channel(_) => {
                return Err(PermissionError::PermissionTypeError)
            }
        }

        drop(self);

        Ok(())
    }
}
