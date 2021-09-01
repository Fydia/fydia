use std::{
    io::Write,
    process::{exit, Child, Command, Stdio},
};

pub fn launch_server() -> Child {
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
        return Command::new("../target/debug/fydia")
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
    } else if let Ok(_) = std::fs::File::open("../target/release/fydia") {
        return Command::new("../target/release/fydia")
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
    } else {
        println!("No Fydia executable. Try to build fydia: cargo build -p fydia");
        exit(0);
    }
}

mod test;
