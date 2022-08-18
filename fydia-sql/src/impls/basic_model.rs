use fydia_struct::{
    channel::{Channel, ChannelId, ChannelType},
    instance::Instance,
    messages::{Message, MessageType},
    roles::Role,
    server::{Members, Server, ServerId, Servers},
    user::{User, UserId},
};
use fydia_utils::async_trait;
use migration::{IntoCondition, SimpleExpr};
use sea_orm::{ColumnTrait, DatabaseConnection as DbConnection, EntityTrait, QueryFilter};

use super::{members::SqlMembers, role::SqlRoles, server::SqlServerId, user::SqlUser};

#[async_trait::async_trait]
pub trait BasicModel {
    type Entity: EntityTrait;
    type StructSelf;

    async fn to_struct(&self, db: &DbConnection) -> Result<Self::StructSelf, String>;

    async fn get_model_by(
        simpl: &[SimpleExpr],
        executor: &DbConnection,
    ) -> Result<<Self::Entity as EntityTrait>::Model, String> {
        let mut find = <Self::Entity>::find();

        for i in simpl.iter() {
            find = find.filter(i.clone().into_condition());
        }

        Ok(find
            .one(executor)
            .await
            .map_err(|err| err.to_string())?
            .ok_or_else(|| String::from("Model doesn't exists"))?)
    }

    async fn get_models_by(
        simpl: &[SimpleExpr],
        executor: &DbConnection,
    ) -> Result<Vec<<Self::Entity as EntityTrait>::Model>, String> {
        let mut find = <Self::Entity>::find();

        for i in simpl {
            find = find.filter(i.clone().into_condition());
        }

        Ok(find.all(executor).await.map_err(|err| err.to_string())?)
    }
    async fn get_model_by_id(
        id: &str,
        executor: &DbConnection,
    ) -> Result<<Self::Entity as EntityTrait>::Model, String>;
}

#[async_trait::async_trait]
impl BasicModel for entity::channels::Model {
    type StructSelf = fydia_struct::channel::Channel;
    type Entity = entity::channels::Entity;

    async fn to_struct(&self, _: &DbConnection) -> Result<Self::StructSelf, String> {
        let channel_type = ChannelType::from_int(self.channel_type);

        let parent_id = ServerId::new(self.parent_id.clone());

        Ok(Channel {
            id: ChannelId::new(self.id.clone()),
            name: self.name.clone(),
            parent_id,
            channel_type,
            description: self.description.clone().unwrap_or_default(),
        })
    }

    async fn get_model_by_id(
        id: &str,
        executor: &DbConnection,
    ) -> Result<<Self::Entity as EntityTrait>::Model, String> {
        Self::get_model_by(&[entity::channels::Column::Id.eq(id)], executor).await
    }
}

#[async_trait::async_trait]
impl BasicModel for entity::server::Model {
    type StructSelf = fydia_struct::server::Server;
    type Entity = entity::server::Entity;

    async fn to_struct(&self, executor: &DbConnection) -> Result<Self::StructSelf, String> {
        let id = ServerId::new(self.id.clone());
        let members = Members::users_of(&id, executor).await?;
        let roles = Role::by_server_id(id.id.clone(), executor).await?;

        let channel = id.get(executor).await?.channel; //::get_channels_by_server_id(&id, executor).await?;

        Ok(Server {
            id,
            name: self.name.clone(),
            owner: UserId::new(self.owner),
            icon: self.icon.clone().unwrap_or_else(|| "Error".to_string()),
            members,
            channel,
            roles,
            emoji: Vec::new(),
        })
    }

    async fn get_model_by_id(
        id: &str,
        executor: &DbConnection,
    ) -> Result<<<Self as BasicModel>::Entity as EntityTrait>::Model, String> {
        Self::get_model_by(&[entity::server::Column::Id.eq(id)], executor).await
    }
}

#[async_trait::async_trait]
impl BasicModel for entity::user::Model {
    type StructSelf = fydia_struct::user::User;
    type Entity = entity::user::Entity;

    async fn to_struct(&self, executor: &DbConnection) -> Result<Self::StructSelf, String> {
        let servers = Members::servers_of(&UserId::new(self.id), executor).await?;

        Ok(User {
            id: UserId::new(self.id),
            name: self.name.clone(),
            description: self.description.clone(),
            email: self.email.clone(),
            instance: Instance::default(),
            token: Some(self.token.clone()),
            password: Some(self.password.clone()),
            servers: Servers(servers),
        })
    }

    async fn get_model_by_id(
        id: &str,
        executor: &DbConnection,
    ) -> Result<<<Self as BasicModel>::Entity as EntityTrait>::Model, String> {
        Self::get_model_by(&[entity::user::Column::Id.eq(id)], executor).await
    }
}

#[async_trait::async_trait]
impl BasicModel for entity::messages::Model {
    type StructSelf = fydia_struct::messages::Message;
    type Entity = entity::messages::Entity;

    async fn to_struct(&self, executor: &DbConnection) -> Result<Self::StructSelf, String> {
        let author_id = User::by_id(self.author_id, executor)
            .await
            .ok_or_else(|| "Error Author_Id".to_string())?;

        let message_type = MessageType::from_string(&self.message_type)
            .ok_or_else(|| "Error Message_type".to_string())?;

        Ok(Message {
            id: self.id.clone(),
            content: self.content.clone().unwrap_or_default(),
            message_type,
            edited: self.edited != 0,
            timestamp: fydia_struct::messages::Date::parse_from_naivetime(self.timestamp),
            channel_id: ChannelId::new(self.channel_id.clone()),
            author_id,
        })
    }

    async fn get_model_by_id(
        id: &str,
        executor: &DbConnection,
    ) -> Result<<<Self as BasicModel>::Entity as EntityTrait>::Model, String> {
        Self::get_model_by(&[entity::messages::Column::Id.eq(id)], executor).await
    }
}
