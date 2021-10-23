mod tests {
    mod user {

        #[test]
        pub fn test() {}
    }

    mod permission {
        use crate::permission::Permission;
        #[cfg(test)]
        const PERMISSION_TO_TEST: u32 = Permission::Read as u32 | Permission::Write as u32;
        #[test]
        pub fn can() {
            assert_eq!(Permission::can(PERMISSION_TO_TEST, Permission::Read), true);
        }
        #[test]
        pub fn cant() {
            assert_eq!(
                Permission::can(PERMISSION_TO_TEST, Permission::Admin),
                false
            );
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
                        ..Default::default()
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
                        ..Default::default()
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
                        ..Default::default()
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
                        ..Default::default()
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
                        ..Default::default()
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
                        ..Default::default()
                    }),
                    ChannelFormat::from_string("Channel\\è###Server🙊/\\*/$localhost.com")
                );
            }
        }
    }
}
