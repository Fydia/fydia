use super::{channel::SqlChannel, members::SqlMembers, role::SqlRoles, user::SqlUser};
use fydia_struct::{
    channel::{Channel, ChannelError, ChannelId, ChannelType},
    instance::Instance,
    messages::{Message, MessageError, MessageType, MessageTypeError},
    permission::Permission,
    roles::{Role, RoleError},
    server::{Members, MembersError, Server, ServerError, ServerId, Servers},
    user::{Token, User, UserError, UserId},
};
use fydia_utils::async_trait;
use migration::{ColumnRef, DbErr, IntoCondition, SimpleExpr};
use sea_orm::{ColumnTrait, DatabaseConnection as DbConnection, EntityTrait, QueryFilter};
use shared::sea_orm;
use thiserror::Error;

#[async_trait::async_trait]
pub trait BasicModel {
    type Entity: EntityTrait;
    type StructSelf;

    async fn to_struct(&self, db: &DbConnection) -> Result<Self::StructSelf, ModelError>;
    async fn get_model_by_id(
        id: &str,
        executor: &DbConnection,
    ) -> Result<<Self::Entity as EntityTrait>::Model, ModelError>;

    async fn get_model_by(
        simpl: &[SimpleExpr],
        executor: &DbConnection,
    ) -> Result<<Self::Entity as EntityTrait>::Model, ModelError> {
        let mut find = <Self::Entity>::find();

        for i in simpl.iter() {
            find = find.filter(i.clone().into_condition());
        }

        Ok(find
            .one(executor)
            .await?
            .ok_or_else(|| ModelError::ModelNotExist(RequestError::new(simpl)))?)
    }

    async fn get_models_by(
        simpl: &[SimpleExpr],
        executor: &DbConnection,
    ) -> Result<Vec<<Self::Entity as EntityTrait>::Model>, ModelError> {
        let mut find = <Self::Entity>::find();

        for i in simpl {
            find = find.filter(i.clone().into_condition());
        }

        let mes = find.all(executor).await?;

        Ok(mes)
    }
}

#[async_trait::async_trait]
impl BasicModel for entity::channels::Model {
    type StructSelf = fydia_struct::channel::Channel;
    type Entity = entity::channels::Entity;

    async fn to_struct(&self, _: &DbConnection) -> Result<Self::StructSelf, ModelError> {
        let channel_type = ChannelType::from_int(self.channel_type);

        let parent_id = ServerId::new(self.server_id.clone());

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
    ) -> Result<<Self::Entity as EntityTrait>::Model, ModelError> {
        Self::get_model_by(&[entity::channels::Column::Id.eq(id)], executor).await
    }
}

#[async_trait::async_trait]
impl BasicModel for entity::server::Model {
    type StructSelf = fydia_struct::server::Server;
    type Entity = entity::server::Entity;

    async fn to_struct(&self, executor: &DbConnection) -> Result<Self::StructSelf, ModelError> {
        let id = ServerId::new(self.id.clone());
        let members = Members::users_of(&id, executor).await?;
        let roles = Role::by_server_id(&id.id, executor).await?;
        let channel = Channel::by_serverid(&id, executor).await?;

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
    ) -> Result<<<Self as BasicModel>::Entity as EntityTrait>::Model, ModelError> {
        Self::get_model_by(&[entity::server::Column::Id.eq(id)], executor).await
    }
}

#[async_trait::async_trait]
impl BasicModel for entity::user::Model {
    type StructSelf = fydia_struct::user::User;
    type Entity = entity::user::Entity;

    async fn to_struct(&self, executor: &DbConnection) -> Result<Self::StructSelf, ModelError> {
        let servers = Members::servers_of(&UserId::new(self.id), executor).await?;

        Ok(User {
            id: UserId::new(self.id),
            name: self.name.clone(),
            description: self.description.clone(),
            email: self.email.clone(),
            instance: Instance::default(),
            token: Token::new(self.token.clone()),
            password: Some(self.password.clone()),
            servers: Servers(servers),
        })
    }

    async fn get_model_by_id(
        id: &str,
        executor: &DbConnection,
    ) -> Result<<<Self as BasicModel>::Entity as EntityTrait>::Model, ModelError> {
        Self::get_model_by(&[entity::user::Column::Id.eq(id)], executor).await
    }
}

#[async_trait::async_trait]
impl BasicModel for entity::messages::Model {
    type StructSelf = fydia_struct::messages::Message;
    type Entity = entity::messages::Entity;

    async fn to_struct(&self, executor: &DbConnection) -> Result<Self::StructSelf, ModelError> {
        let author_id = User::by_id(self.author_id, executor).await?;

        let message_type = MessageType::from_string(&self.message_type)?;

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
    ) -> Result<<<Self as BasicModel>::Entity as EntityTrait>::Model, ModelError> {
        Self::get_model_by(&[entity::messages::Column::Id.eq(id)], executor).await
    }
}

#[async_trait::async_trait]
impl BasicModel for entity::permission::role::Model {
    type StructSelf = fydia_struct::permission::Permission;
    type Entity = entity::permission::role::Entity;

    async fn to_struct(&self, executor: &DbConnection) -> Result<Self::StructSelf, ModelError> {
        let channel = Channel::by_id(
            &ChannelId {
                id: self.channel.clone(),
            },
            executor,
        )
        .await?;

        let role = Role::by_id(self.role, &channel.parent_id, executor).await?;

        Ok(Permission::role(role.id, Some(channel.id), self.value))
    }

    async fn get_model_by_id(
        _: &str,
        _: &DbConnection,
    ) -> Result<<<Self as BasicModel>::Entity as EntityTrait>::Model, ModelError> {
        Err(ModelError::NoPrimaryKey)
    }
}

#[async_trait::async_trait]
impl BasicModel for entity::permission::user::Model {
    type StructSelf = fydia_struct::permission::Permission;
    type Entity = entity::permission::user::Entity;

    async fn to_struct(&self, executor: &DbConnection) -> Result<Self::StructSelf, ModelError> {
        let user = User::by_id(self.user, executor).await?;

        let channel = Channel::by_id(
            &ChannelId {
                id: self.channel.clone(),
            },
            executor,
        )
        .await?;

        Ok(Permission::user(user.id, Some(channel.id), self.value))
    }

    async fn get_model_by_id(
        _: &str,
        _: &DbConnection,
    ) -> Result<<<Self as BasicModel>::Entity as EntityTrait>::Model, ModelError> {
        Err(ModelError::NoPrimaryKey)
    }
}

#[derive(Debug)]
pub struct RequestError {
    expr: Vec<SimpleExpr>,
}

impl RequestError {
    pub fn get_column_names(&self) -> Vec<String> {
        let mut t = Vec::new();
        for el in &self.expr {
            if let SimpleExpr::Binary(simplexpr, _, _) = el {
                if let SimpleExpr::Column(ColumnRef::TableColumn(_, column)) = *simplexpr.clone() {
                    t.push(column.to_string());
                }
            }
        }

        t
    }
    pub fn new(values: &[SimpleExpr]) -> Self {
        Self {
            expr: values.to_vec(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("No primary key")]
    NoPrimaryKey,
    #[error("This model doesn't exists")]
    ModelNotExist(RequestError),
    #[error("{0}")]
    ChannelError(Box<ChannelError>),
    #[error("{0}")]
    RoleError(Box<RoleError>),
    #[error("{0}")]
    UserError(Box<UserError>),
    #[error("{0}")]
    ServerError(Box<ServerError>),
    #[error("{0}")]
    MessageTypeError(Box<MessageTypeError>),
    #[error("{0}")]
    MembersError(Box<MembersError>),
    #[error("{0}")]
    Other(String),
}

impl From<MembersError> for ModelError {
    fn from(value: MembersError) -> Self {
        Self::MembersError(Box::new(value))
    }
}

impl From<RoleError> for ModelError {
    fn from(value: RoleError) -> Self {
        ModelError::RoleError(Box::new(value))
    }
}

impl From<ChannelError> for ModelError {
    fn from(value: ChannelError) -> Self {
        ModelError::ChannelError(Box::new(value))
    }
}

impl From<ModelError> for ChannelError {
    fn from(_: ModelError) -> Self {
        ChannelError::CannotGetFromDatabase
    }
}

impl From<UserError> for ModelError {
    fn from(value: UserError) -> Self {
        ModelError::UserError(Box::new(value))
    }
}

impl From<ServerError> for ModelError {
    fn from(value: ServerError) -> Self {
        ModelError::ServerError(Box::new(value))
    }
}

impl From<DbErr> for ModelError {
    fn from(value: DbErr) -> Self {
        ModelError::Other(value.to_string())
    }
}

impl From<MessageTypeError> for ModelError {
    fn from(value: MessageTypeError) -> Self {
        Self::MessageTypeError(Box::new(value))
    }
}

impl From<ModelError> for UserError {
    fn from(value: ModelError) -> Self {
        match value {
            ModelError::NoPrimaryKey => Self::CannotGetById,
            ModelError::ModelNotExist(f) => {
                let columns = f.get_column_names();

                if columns.contains(&"id".to_string()) {
                    return Self::CannotGetById;
                } else if columns.contains(&"token".to_string()) {
                    return Self::CannotGetByToken;
                } else {
                    Self::Other("Cannot get user".to_string())
                }
            }
            ModelError::ChannelError(_) => todo!(),
            ModelError::RoleError(_) => todo!(),
            ModelError::UserError(_) => todo!(),
            ModelError::ServerError(_) => todo!(),
            ModelError::MessageTypeError(_) => todo!(),
            ModelError::MembersError(_) => todo!(),
            ModelError::Other(_) => todo!(),
        }
    }
}

impl From<ModelError> for ServerError {
    fn from(value: ModelError) -> Self {
        match value {
            ModelError::NoPrimaryKey => Self::CannotGetById,
            ModelError::ModelNotExist(f) => {
                let columns = f.get_column_names();

                if columns.contains(&"id".to_string()) {
                    return Self::CannotGetById;
                } else {
                    error!("Unhandled error: {:?}", f);
                    todo!()
                }
            }
            _ => Self::ModelToStruct,
        }
    }
}

impl From<ModelError> for MessageError {
    fn from(value: ModelError) -> Self {
        match value {
            ModelError::NoPrimaryKey => Self::CannotGetById,
            ModelError::ModelNotExist(f) => {
                let columns = f.get_column_names();

                if columns.contains(&"id".to_string()) {
                    return Self::CannotGetById;
                } else {
                    error!("Unhandled error: {:?}", f);
                    todo!()
                }
            }
            _ => Self::ModelToStruct,
        }
    }
}
