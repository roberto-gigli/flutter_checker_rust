use std::{error::Error, fs::read, process::Command};

//El primo
fn main() {
    println!("Loading flutter version...");
    let flutter_version: &str = &get_flutter_version();
    println!("Flutter version {flutter_version}");

    println!("Loading flutter path...");
    let flutter_path = &get_flutter_path();
    println!("Flutter path {flutter_path}");

    println!("Loading project version...");
    let project_version: &str = &get_project_version().expect("Cannot get project version");
    println!("Project version {project_version}");
}

fn command_flutter() -> Command {
    if cfg!(windows) {
        return Command::new("flutter.bat");
    }

    Command::new("flutter")
}

fn command_where() -> Command {
    if cfg!(windows) {
        return Command::new("where.exe");
    }

    Command::new("which")
}

fn get_flutter_version() -> String {
    let output = command_flutter()
        .arg("--version")
        .output()
        .expect("Cannot get flutter version");

    let output_str = String::from_utf8(output.stdout).unwrap_or_default();
    let version = output_str
        .split("\n")
        .nth(0)
        .unwrap_or_default()
        .split(" ")
        .nth(1)
        .unwrap_or_default()
        .to_string();

    return version;
}

fn get_flutter_path() -> String {
    let output = command_where()
        .arg("flutter")
        .output()
        .expect("Cannot get flutter path");

    let output_str = String::from_utf8(output.stdout).unwrap_or_default();

    if cfg!(windows) {
        let mut path_segments = output_str
            .split("\n")
            .nth(0)
            .unwrap_or_default()
            .split("\\")
            .collect::<Vec<&str>>();

        path_segments.pop();

        return path_segments.join("\\");
    }

    let mut path_segments = output_str.trim().split("/").collect::<Vec<&str>>();
    path_segments.pop();

    return path_segments.join("/");
}

fn read_file_as_string(path: &str) -> Result<String, Box<dyn Error>> {
    let byte_content = read(path)?;
    let content = String::from_utf8(byte_content)?;

    return Ok(content);
}

fn get_project_version() -> Option<String> {
    let content = match read_file_as_string("pubspec.yaml") {
        Ok(content) => content,
        Err(_) => return None,
    };

    if content.is_empty() {
        return None;
    }

    match serde_yaml::from_str::<serde_yaml::Value>(&content) {
        Ok(pubspec) => {
            let version = pubspec
                .get("environment")?
                .get("flutter")?
                .as_str()?
                .to_string();

            return Some(version);
        }
        Err(_) => return None,
    };
}
