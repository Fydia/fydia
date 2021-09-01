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
    if let Ok(_) = std::fs::File::open("../target/debug/fydia") {
        Command::new("../target/debug/fydia")
            .stdout(Stdio::null())
            .spawn()
            .unwrap()
            .wait()
            .expect("Error");
    } else if let Ok(_) = std::fs::File::open("../target/release/fydia") {
        Command::new("../target/release/fydia")
            .stdout(Stdio::null())
            .spawn()
            .unwrap()
            .wait()
            .expect("Error");
    } else {
        println!("No Fydia executable. Try to build fydia: cargo build -p fydia")
    }
}

mod test;
