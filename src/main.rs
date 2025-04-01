use clap::Parser;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fmt};

trait Printable: fmt::Display {
    fn print(&self) {
        println!("{self}");
    }
}

#[derive(Parser)]
struct Args {
    #[arg(short = 'd', long = "workingDirectory", value_hint = clap::ValueHint::DirPath)]
    working_dir: Option<PathBuf>,
    #[arg(short = 'v', long = "desiredVersion")]
    desired_version: Option<String>,
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Working directory: {:?}",
            self.working_dir
                .as_ref()
                .and_then(|path| path.to_str())
                .unwrap_or("None")
        )?;

        writeln!(
            f,
            "Working directory: {:?}",
            self.working_dir
                .as_ref()
                .and_then(|path| path.to_str())
                .unwrap_or("None")
        )?;
        write!(
            f,
            "Desired version: {:?}",
            self.desired_version.as_deref().unwrap_or("None")
        )
    }
}

impl Printable for Args {}

struct Status {
    project_version: Option<String>,
    flutter_version: Option<String>,
    flutter_path: Option<PathBuf>,
    flutter_root_path: Option<PathBuf>,
}

impl Status {
    const fn new() -> Status {
        Status {
            project_version: None,
            flutter_version: None,
            flutter_path: None,
            flutter_root_path: None,
        }
    }

    fn update(&mut self) {
        self.flutter_version = get_flutter_version();
        self.flutter_path = get_flutter_path();
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Project version: {}",
            self.project_version.as_deref().unwrap_or("None")
        )?;
        writeln!(
            f,
            "Flutter version: {}",
            self.flutter_version.as_deref().unwrap_or("None")
        )?;
        writeln!(
            f,
            "Flutter path: {}",
            self.flutter_path
                .as_ref()
                .and_then(|path| path.to_str())
                .unwrap_or("None")
        )?;
        write!(
            f,
            "Flutter root path: {}",
            self.flutter_root_path
                .as_ref()
                .and_then(|path| path.to_str())
                .unwrap_or("None")
        )
    }
}

impl Printable for Status {}

fn main() {
    let args = Args::parse_from(env::args().collect::<Vec<String>>());
    args.print();
    run(&args);
}

fn system_run(command: &str, cwd: &Option<PathBuf>) -> String {
    let mut command = Command::new(command);

    match &cwd {
        Some(path) => {
            command.current_dir(path);
        }
        None => {}
    }

    let result = command.output();

    match result {
        Ok(output) => String::from_utf8([output.stdout, output.stderr].concat())
            .unwrap_or_else(|e| e.to_string()),
        Err(e) => e.to_string(),
    }
}

fn get_flutter_version() -> Option<String> {
    let output = system_run("flutter --version", &None);

    Some(
        output
            .split("\n")
            .nth(0)?
            .split(" ")
            .nth(1)?
            .trim()
            .to_string(),
    )
}

fn get_flutter_path() -> Option<PathBuf> {
    match env::consts::OS {
        "windows" => {
            let output = system_run("where flutter", &None);

            let mut split: Vec<&str> = output.split("\n").nth(0)?.split("\\").collect();
            split.pop();

            Some(split.join("\\").into())
        }
        "macos" | "linux" => {
            let output = system_run("which flutter", &None);

            let mut split: Vec<&str> = output.split("\n").nth(0)?.split("/").collect();
            split.pop();

            Some(split.join("/").into())
        }
        _ => None,
    }
}

fn run(args: &Args) {
    println!("Flutter rust checker version {}", env!("CARGO_PKG_VERSION"));

    match &args.working_dir {
        Some(working_dir) => {
            env::set_current_dir(working_dir).unwrap();
        }
        None => {
            println!("No working directory specified, using current directory");
        }
    };

    let current_dir = env::current_dir().unwrap();

    println!("Current directory is: {:?}", current_dir);

    match &args.desired_version {
        Some(desired_version) => {
            if !desired_version.trim().is_empty() {
                println!("DesiredVersion: {}", desired_version)
            }
        }
        None => {}
    }

    let mut status = Status::new();

    status.update();

    status.print();
}
