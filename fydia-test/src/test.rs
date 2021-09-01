mod test {

    #[tokio::test]
    pub async fn test_create_user() -> Result<(), ()> {
        use std::{
            thread::{sleep, spawn},
            time::Duration,
        };

        use crate::launch_server;

        spawn(|| launch_server());

        loop {
            match ureq::post("http://127.0.0.1:8888/api/user/create").send_string(
                r#"{"name":"TEST_USER","email":"EMAIL@EMAIL.COM","password":"TEST_PASSWORD"}"#,
            ) {
                Ok(e) => match e.status() {
                    200 => {
                        return Ok(());
                    }

                    _ => {
                        return Err(());
                    }
                },
                Err(e) => match e.kind() {
                    ureq::ErrorKind::HTTP => {
                        return Err(());
                    }
                    _ => {}
                },
            }
            sleep(Duration::from_secs(1))
        }
    }
}
