use std::{fs, path::PathBuf, process::Command};

fn main() {
    let description = Command::new("git")
        .args(["describe", "--always"])
        .output()
        .expect("git has to be installed")
        .stdout;

    if !PathBuf::from("bot-data").exists() {
        fs::create_dir("bot-data").unwrap();
    }

    fs::write("bot-data/VERSION", description).unwrap()
}
