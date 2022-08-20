mod tests {
    mod user {

        #[test]
        pub fn test() {}
    }

    mod permission {
        use crate::{
            permission::{self, Permission, PermissionValue, Permissions},
            user::UserId,
        };
        /* #[cfg(test)]
        const PERMISSION_TO_TEST: u64 =
            PermissionValue::Read as u64 | PermissionValue::Write as u64;
        #[test]
        pub fn can() {
            assert_eq!(
                PermissionValue::can(PERMISSION_TO_TEST, PermissionValue::Read),
                true
            );
        }
        #[test]
        pub fn cant() {
            assert_eq!(
                PermissionValue::can(PERMISSION_TO_TEST, PermissionValue::Admin),
                false
            );
        }*/

        #[test]
        pub fn cant() {
            let permissions = Permissions::new(vec![
                Permission::user(
                    UserId::new(0),
                    crate::channel::ChannelId::new("eejahjdfghakjsdhjaksjhdkj"),
                    PermissionValue::Read as u64 | PermissionValue::Write as u64,
                ),
                Permission::user(
                    UserId::new(0),
                    crate::channel::ChannelId::new("eejahjdfghakjsdhjaksjhdkj"),
                    PermissionValue::Admin as u64 | PermissionValue::Write as u64,
                ),
            ]);

            let value =
                permissions.calculate(crate::channel::ChannelId::new("eejahjdfghakjsdhjaksjhdkj"));

            assert!(value.can_vec(&[
                PermissionValue::Read,
                PermissionValue::Write,
                PermissionValue::Admin
            ]));
        }
    }
    mod formated {
        mod user {
            use crate::format::UserFormat;

            #[test]
            pub fn userformat_1() {
                assert_eq!(
                    Some(UserFormat {
                        name: "User".into(),
                        domain: "localhost.com".into(),
                        port: Some(80)
                    }),
                    UserFormat::from_string("User@localhost.com")
                );
            }

            #[test]
            pub fn userformat_2() {
                assert_eq!(
                    Some(UserFormat {
                        name: "User@@@@🙊/\\*/".into(),
                        domain: "localhost.com".into(),
                        port: Some(80)
                    }),
                    UserFormat::from_string("User@@@@🙊/\\*/@localhost.com")
                );
            }
        }
        mod server {
            use crate::format::ServerFormat;

            #[test]
            pub fn server_1() {
                assert_eq!(
                    Some(ServerFormat {
                        name: "Server".into(),
                        domain: "localhost.com".into(),
                        port: Some(80)
                    }),
                    ServerFormat::from_string("Server$localhost.com")
                );
            }

            #[test]
            pub fn server_2() {
                assert_eq!(
                    Some(ServerFormat {
                        name: "Server$$$$🙊/\\*/".into(),
                        domain: "localhost.com".into(),
                        port: Some(80)
                    }),
                    ServerFormat::from_string("Server$$$$🙊/\\*/$localhost.com")
                );
            }
        }
        mod channel {
            use crate::format::ChannelFormat;

            #[test]
            pub fn channel_1() {
                assert_eq!(
                    Some(ChannelFormat {
                        channel: "Channel".into(),
                        server: "server".into(),
                        domain: "localhost.com".into(),
                        port: Some(80)
                    }),
                    ChannelFormat::from_string("Channel#server$localhost.com")
                );
            }

            #[test]
            pub fn channel_2() {
                assert_eq!(
                    Some(ChannelFormat {
                        channel: "Channel\\è##".into(),
                        server: "Server🙊/\\*/".into(),
                        domain: "localhost.com".into(),
                        port: Some(80)
                    }),
                    ChannelFormat::from_string("Channel\\è###Server🙊/\\*/$localhost.com")
                );
            }
        }
    }
}
