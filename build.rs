use build_script_file_gen::gen_file_str;
use std::process::Command;

fn main() {
    let description = Command::new("git")
        .args(["describe", "--always"])
        .output()
        .expect("git has to be installed")
        .stdout;

    gen_file_str("VERSION", &String::from_utf8(description).unwrap());
}
