use std::convert::TryFrom;

use super::{
    basic_model::BasicModel,
    channel::SqlChannelId,
    delete, insert,
    user::{SqlUser, UserFrom},
};
use fydia_struct::{
    channel::ChannelId,
    permission::{Permission, Permissions},
    response::{FydiaResponse, IntoFydia, MapError},
    roles::RoleId,
    server::ServerId,
    user::UserId,
};
use fydia_utils::async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use shared::sea_orm;

#[async_trait]
pub trait PermissionSql {
    async fn of_role_in_channel<'a>(
        channelid: &ChannelId,
        role: &RoleId,
        db: &DatabaseConnection,
    ) -> Result<Permission, FydiaResponse<'a>>;

    async fn of_user_in_channel<'a>(
        channelid: &ChannelId,
        user: &UserId,
        db: &DatabaseConnection,
    ) -> Result<Permission, FydiaResponse<'a>>;
    async fn of_user_with_role_in_channel<'a>(
        channelid: &ChannelId,
        user: &UserId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, FydiaResponse<'a>>;
    async fn of_user<'a>(
        user: &UserId,
        serverid: &ServerId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, FydiaResponse<'a>>;
    async fn of_channel<'a>(
        channelid: &ChannelId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, FydiaResponse<'a>>;
    async fn insert<'a>(&self, db: &DatabaseConnection) -> Result<(), FydiaResponse<'a>>;
    async fn update_value<'a>(
        self,
        db: &DatabaseConnection,
    ) -> Result<Permission, FydiaResponse<'a>>;
    async fn delete<'a>(mut self, db: &DatabaseConnection) -> Result<(), FydiaResponse<'a>>;
}

#[async_trait]
impl PermissionSql for Permission {
    async fn of_user<'a>(
        user: &UserId,
        serverid: &ServerId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, FydiaResponse<'a>> {
        let user = user.to_user(db).await?;
        let roles = user.roles(serverid, db).await?;
        let mut vec = Vec::new();

        for role in roles {
            vec.push(Permission::role(role.id, None, role.server_permission));
        }

        Ok(Permissions::new(vec))
    }

    async fn of_role_in_channel<'a>(
        channelid: &ChannelId,
        roleid: &RoleId,
        db: &DatabaseConnection,
    ) -> Result<Permission, FydiaResponse<'a>> {
        entity::permission::role::Entity::find()
            .filter(
                entity::permission::role::Column::Role.eq(roleid.get_id_cloned_fydiaresponse()?),
            )
            .filter(entity::permission::role::Column::Channel.eq(channelid.id.as_str()))
            .one(db)
            .await
            .error_to_fydiaresponse()?
            .ok_or_else(|| "No role permission".into_error())?
            .to_struct(db)
            .await
    }

    async fn of_user_in_channel<'a>(
        channelid: &ChannelId,
        user: &UserId,
        db: &DatabaseConnection,
    ) -> Result<Permission, FydiaResponse<'a>> {
        entity::permission::user::Entity::find()
            .filter(
                entity::permission::user::Column::User
                    .eq(user.0.clone().get_id_cloned_fydiaresponse()?),
            )
            .filter(entity::permission::user::Column::Channel.eq(channelid.id.as_str()))
            .one(db)
            .await
            .error_to_fydiaresponse()?
            .ok_or_else(|| "No role permission".into_error())?
            .to_struct(db)
            .await
    }

    async fn of_user_with_role_in_channel<'a>(
        channelid: &ChannelId,
        user: &UserId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, FydiaResponse<'a>> {
        let channel = channelid.channel(db).await?;
        let user = user.to_user(db).await?;
        let roles = user.roles(&channel.parent_id, db).await?;

        let mut vec = Vec::new();

        for i in roles.iter() {
            if let Ok(perm) = Self::of_role_in_channel(channelid, &i.id, db).await {
                vec.push(perm);
            }
        }
        if let Ok(perm) = Self::of_user_in_channel(channelid, &user.id, db).await {
            vec.push(perm);
        }

        return Ok(Permissions::new(vec));
    }
    async fn of_channel<'a>(
        channelid: &ChannelId,
        db: &DatabaseConnection,
    ) -> Result<Permissions, FydiaResponse<'a>> {
        let result = entity::permission::user::Entity::find()
            .filter(entity::permission::user::Column::Channel.eq(channelid.id.as_str()))
            .all(db)
            .await
            .error_to_fydiaresponse()?;

        let mut vec = Vec::new();
        for i in result {
            vec.push(i.to_struct(db).await?);
        }

        let result = entity::permission::role::Entity::find()
            .filter(entity::permission::role::Column::Channel.eq(channelid.id.as_str()))
            .all(db)
            .await
            .error_to_fydiaresponse()?;

        for i in result {
            vec.push(i.to_struct(db).await?);
        }

        Ok(Permissions::new(vec))
    }

    async fn insert<'a>(&self, db: &DatabaseConnection) -> Result<(), FydiaResponse<'a>> {
        match self.permission_type {
            fydia_struct::permission::PermissionType::Role(_) => {
                let am = entity::permission::role::ActiveModel::try_from(self.clone())
                    .error_to_fydiaresponse()?;

                insert(am, db).await?;
            }
            fydia_struct::permission::PermissionType::User(_) => {
                let am = entity::permission::user::ActiveModel::try_from(self.clone())
                    .error_to_fydiaresponse()?;

                insert(am, db).await?;
            }

            fydia_struct::permission::PermissionType::Channel(_) => {
                return Err("Bad Type".into_error())
            }
        }

        Ok(())
    }

    async fn update_value<'a>(
        self,
        db: &DatabaseConnection,
    ) -> Result<Permission, FydiaResponse<'a>> {
        let channelid = self
            .channelid
            .clone()
            .ok_or_else(|| "No channelid".into_error())?
            .id;

        match &self.permission_type {
            fydia_struct::permission::PermissionType::Role(role) => {
                let am = entity::permission::role::ActiveModel::try_from(&self)
                    .error_to_fydiaresponse()?;

                entity::permission::role::Entity::update(am)
                    .filter(entity::permission::role::Column::Channel.eq(channelid.as_str()))
                    .filter(
                        entity::permission::role::Column::Role
                            .eq(role.get_id_cloned_fydiaresponse()?),
                    )
                    .exec(db)
                    .await
                    .map(|_| ())
                    .error_to_fydiaresponse()?;
            }
            fydia_struct::permission::PermissionType::User(user) => {
                let am = entity::permission::user::ActiveModel::try_from(&self)
                    .error_to_fydiaresponse()?;

                entity::permission::user::Entity::update(am)
                    .filter(entity::permission::user::Column::Channel.eq(channelid.as_str()))
                    .filter(
                        entity::permission::user::Column::User
                            .eq(user.0.get_id_cloned_fydiaresponse()?),
                    )
                    .exec(db)
                    .await
                    .map(|_| ())
                    .error_to_fydiaresponse()?;
            }
            fydia_struct::permission::PermissionType::Channel(_) => {
                return Err("Bad Type".into_error())
            }
        }

        Ok(self)
    }

    async fn delete<'a>(mut self, db: &DatabaseConnection) -> Result<(), FydiaResponse<'a>> {
        match &self.permission_type {
            fydia_struct::permission::PermissionType::Role(_) => {
                let am = entity::permission::role::ActiveModel::try_from(&self)
                    .error_to_fydiaresponse()?;

                delete(am, db).await?;
            }
            fydia_struct::permission::PermissionType::User(_) => {
                let am = entity::permission::user::ActiveModel::try_from(&self)
                    .error_to_fydiaresponse()?;

                delete(am, db).await?;
            }
            fydia_struct::permission::PermissionType::Channel(_) => {
                return Err("Bad Type".into_error())
            }
        }

        drop(self);

        Ok(())
    }
}
