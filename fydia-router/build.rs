use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    let split = git_hash.split_at(8);
    println!("cargo:rustc-env=GIT_HASH={}", split.0);
    if cfg!(feature = "flutter_client") {
        Command::new("git")
            .current_dir(".")
            .args(&[
                "clone",
                "https://github.com/fydia/fydiapackages",
                "--depth",
                "1",
            ])
            .output()
            .unwrap();

        Command::new("flutter")
            .current_dir("./fydiapackages/fydiaclient")
            .args(&["build", "web"])
            .output()
            .unwrap();
    }
}
