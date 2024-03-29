//! `SeaORM` Entity. Generated by sea-orm-codegen 0.6.0
pub mod role {
    use fydia_struct::permission::{Permission, PermissionError, PermissionType};
    use sea_orm::{entity::prelude::*, Set};
    use shared::sea_orm;
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "permission_roles")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub channel: String,
        #[sea_orm(primary_key, auto_increment = false)]
        pub role: u32,
        #[sea_orm(auto_increment = false)]
        pub value: u64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "crate::roles::Entity",
            from = "Column::Role",
            to = "crate::roles::Column::Id",
            on_update = "Restrict",
            on_delete = "Restrict"
        )]
        Role,
        #[sea_orm(
            belongs_to = "crate::channels::Entity",
            from = "Column::Channel",
            to = "crate::channels::Column::Id",
            on_update = "Restrict",
            on_delete = "Restrict"
        )]
        Channel,
    }

    impl Related<crate::roles::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Role.def()
        }
    }

    impl Related<crate::channels::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Channel.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}

    impl TryFrom<Permission> for ActiveModel {
        type Error = PermissionError;
        fn try_from(perm: Permission) -> Result<Self, Self::Error> {
            let PermissionType::Role(role) = perm.permission_type else {
                return Err(PermissionError::PermissionTypeError);
            };

            Ok(Self {
                channel: Set(perm
                    .channelid
                    .ok_or_else(|| PermissionError::NoChannelId)?
                    .id),
                role: Set(role.get_id()?),
                value: Set(perm.value),
            })
        }
    }

    impl TryFrom<&Permission> for ActiveModel {
        type Error = PermissionError;
        fn try_from(perm: &Permission) -> Result<Self, Self::Error> {
            Self::try_from(perm.clone())
        }
    }
}

pub mod user {
    use fydia_struct::permission::{Permission, PermissionError, PermissionType};
    use sea_orm::{entity::prelude::*, Set};
    use shared::sea_orm;
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "permission_users")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub user: u32,
        #[sea_orm(primary_key, auto_increment = false)]
        pub channel: String,
        #[sea_orm(auto_increment = false)]
        pub value: u64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "crate::channels::Entity",
            from = "Column::Channel",
            to = "crate::channels::Column::Id",
            on_update = "Restrict",
            on_delete = "Restrict"
        )]
        Channel,
        #[sea_orm(
            belongs_to = "crate::user::Entity",
            from = "Column::User",
            to = "crate::user::Column::Id",
            on_update = "Restrict",
            on_delete = "Restrict"
        )]
        User,
    }

    impl Related<crate::channels::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Channel.def()
        }
    }

    impl Related<crate::user::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::User.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}

    impl TryFrom<Permission> for ActiveModel {
        type Error = PermissionError;
        fn try_from(perm: Permission) -> Result<Self, Self::Error> {
            let PermissionType::User( user )= perm.permission_type  else {
                return Err(PermissionError::PermissionTypeError);
            };

            Ok(Self {
                channel: Set(perm
                    .channelid
                    .ok_or_else(|| PermissionError::NoChannelId)?
                    .id),
                user: Set(user.0.get_id()?),
                value: Set(perm.value),
            })
        }
    }

    impl TryFrom<&Permission> for ActiveModel {
        type Error = PermissionError;
        fn try_from(perm: &Permission) -> Result<Self, Self::Error> {
            Self::try_from(perm.clone())
        }
    }
}
