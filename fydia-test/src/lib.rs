use std::{
    io::Write,
    process::{Command, Stdio},
};

pub fn launch_server() {
    let mut file = std::fs::File::create("./config.toml").unwrap();
    file.write(
        br#"
    [instance]
    domain = ""
    
    [server]
    ip = "127.0.0.1"
    port = 8888
    
    [database]
    database_type = "Sqlite"
    ip = "fydia"
    port = 3306
    name = "root"
    password = "root"
    database_name = "fydia""#,
    )
    .expect("Error");

    Command::new("../target/debug/fydia")
        .stdout(Stdio::null())
        .spawn()
        .unwrap()
        .wait()
        .expect("Error");
}

mod test;
