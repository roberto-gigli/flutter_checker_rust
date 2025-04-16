use clap::Parser;
use futures::executor::block_on;
use futures::future::join_all;
use futures::{join, FutureExt};
use serde_yaml::Value;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{env, fmt};

#[derive(Debug)]
enum ShellError<T: Error> {
    OSNotSupported,
    CommandFailed(T),
}

impl<T: Error> Display for ShellError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

impl<T: Error> Error for ShellError<T> {}

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

    async fn update(&mut self) {
        join_all([
            async { self.project_version = get_project_version().await }.boxed(),
            async { self.flutter_version = get_flutter_version().await }.boxed(),
            async { self.flutter_path = get_flutter_path().await }.boxed(),
            async { self.flutter_root_path = get_flutter_root_path().await }.boxed(),
        ])
        .await;
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
    let future = run(&args);

    block_on(future)
}

trait ShellCommand {
    fn new_shell<S: AsRef<OsStr>>(program: S) -> Command;
}

impl ShellCommand for Command {
    fn new_shell<S: AsRef<OsStr>>(program: S) -> Command {
        match env::consts::OS {
            "windows" => {
                let mut command = Command::new("cmd");
                command.arg("/C").arg(program);
                command
            }
            "macos" | "linux" => {
                let mut command = Command::new("sh");
                command.arg("-c").arg(program);
                command
            }
            _ => Command::new(program),
        }
    }
}

async fn shell_run(
    shell_command: &str,
    cwd: Option<&PathBuf>,
    console_print: bool,
) -> Result<String, ShellError<std::io::Error>> {
    let mut command = match env::consts::OS {
        "windows" | "macos" | "linux" => Command::new_shell(shell_command),
        _ => {
            return Err(ShellError::OSNotSupported);
        }
    };

    match cwd {
        Some(path) => {
            command.current_dir(path);
        }
        None => {}
    }

    if console_print {
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
    }

    let result = command.output();

    let output = result.map_err(|e| ShellError::CommandFailed(e))?;

    return Ok(String::from_utf8_lossy(&[output.stdout, output.stderr].concat()).to_string());
}

async fn get_flutter_version() -> Option<String> {
    println!("Getting flutter version...");
    let output = shell_run("flutter --version", None, false).await.ok()?;

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

async fn get_flutter_command_path() -> Option<PathBuf> {
    match env::consts::OS {
        "windows" => {
            let output = shell_run("where flutter", None, false).await.ok()?;

            let path = output.split("\n").nth(0)?.trim();

            if path.is_empty() {
                return None;
            }

            Some(path.into())
        }
        "macos" | "linux" => {
            let output = shell_run("which flutter", None, false).await.ok()?;
            let path = output.trim();

            if path.is_empty() {
                return None;
            }

            Some(path.into())
        }
        _ => None,
    }
}

async fn get_git_command_path() -> Option<PathBuf> {
    match env::consts::OS {
        "windows" => {
            let output = shell_run("where git", None, false).await.ok()?;

            let path = output.split("\n").nth(0)?.trim();

            if path.is_empty() {
                return None;
            }

            Some(path.into())
        }
        "macos" | "linux" => {
            let output = shell_run("which git", None, false).await.ok()?;
            let path = output.trim();

            if path.is_empty() {
                return None;
            }

            Some(path.into())
        }
        _ => None,
    }
}

async fn get_flutter_path() -> Option<PathBuf> {
    println!("Getting flutter path...");
    let flutter_command_path = get_flutter_command_path().await?;
    let flutter_path = flutter_command_path.parent()?;
    Some(flutter_path.to_owned())
}

async fn get_flutter_root_path() -> Option<PathBuf> {
    println!("Getting flutter root path...");
    let flutter_path = get_flutter_path().await?;
    let flutter_root_path = flutter_path.parent()?;
    Some(flutter_root_path.to_owned())
}

async fn get_project_version() -> Option<String> {
    println!("Getting project version...");
    let mut pubspec_file = File::open("pubspec.yaml").ok()?;
    let mut buf = String::new();

    pubspec_file.read_to_string(&mut buf).ok()?;

    let pubspec: Value = serde_yaml::from_str(&buf).ok()?;

    Some(
        pubspec
            .get("environment")?
            .get("flutter")?
            .as_str()?
            .to_string(),
    )
}

async fn change_flutter_version(version: &str, status: &Status) -> Result<(), Box<dyn Error>> {
    match version {
        "stable" | "beta" | "main" | "master" => {
            shell_run(format!("flutter channel {}", version).as_ref(), None, true).await?;
            return Ok(());
        }
        _ => {}
    };

    println!("Cleaning flutter working tree...");
    shell_run("git reset --hard", status.flutter_path.as_ref(), true).await?;

    println!("Checking out {version}...");
    shell_run("git fetch", status.flutter_path.as_ref(), true).await?;
    shell_run(
        &format!("git checkout {version}"),
        status.flutter_path.as_ref(),
        true,
    )
    .await?;

    println!("Cleaning flutter working tree...");
    shell_run("git reset --hard", status.flutter_path.as_ref(), true).await?;

    if version == "3.29.0" {
        //https://github.com/flutter/flutter/issues/163308#issuecomment-2661479464
        println!("Applying workaround for Flutter 3.29.0...");
        println!("Removing /engine/src/.gn...");

        match env::consts::OS {
            "windows" => {
                let _ = shell_run(
                    "del .\\engine\\src\\.gn",
                    status.flutter_root_path.as_ref(),
                    true,
                )
                .await;
            }
            "macos" | "linux" => {
                let _ = shell_run(
                    "rm -rf ./engine/src/.gn",
                    status.flutter_root_path.as_ref(),
                    true,
                )
                .await;
            }
            _ => {}
        };
    }

    println!("Running flutter doctor...");
    shell_run("flutter doctor", None, true).await?;

    shell_run("flutter clean", None, true).await?;

    shell_run("flutter pub upgrade", None, true).await?;

    if env::consts::OS == "macos" {
        println!("Running pod install...");
        shell_run("pod install", Some("./ios".into()).as_ref(), true).await?;
    }
    println!("Completed.");
    Ok(())
}

async fn run(args: &Args) {
    println!("Flutter checker rust version {}", env!("CARGO_PKG_VERSION"));

    let (flutter_command, git_command) = join!(get_flutter_command_path(), get_git_command_path());

    if flutter_command == None {
        println!("Flutter not found. Please install it and add it to path");
        return;
    }

    if git_command == None {
        println!("Git not found. Please install it and add it to path");
        return;
    }

    match &args.working_dir {
        Some(working_dir) => match env::set_current_dir(working_dir) {
            Ok(_) => {}
            Err(e) => {
                print!("Could not set the working directory, make sure --workingDirectory (-d) is a correct path\n{}", e);
                return;
            }
        },
        None => {
            println!("No working directory specified, using current directory");
        }
    };

    let current_dir = env::current_dir().unwrap();

    println!("Current directory is: {}", current_dir.display());

    let mut status = Status::new();

    println!("Loading current status...");
    status.update().await;

    status.print();

    match &args.desired_version {
        Some(desired_version) if !desired_version.is_empty() => {
            println!("Desired version: {desired_version}");

            if desired_version == status.flutter_version.as_deref().unwrap_or("None") {
                println!("Flutter version is already {desired_version}");
                return;
            }

            println!("Syncing flutter version with desired version...");

            match change_flutter_version(&desired_version, &status).await {
                Ok(_) => {}
                Err(e) => {
                    println!("Could not change flutter version\\nError: {}", e);
                    return;
                }
            };
            status.update().await;
            status.print();
            return;
        }
        _ => {}
    }

    match &status.project_version {
        Some(project_version) if !project_version.is_empty() => {
            if project_version == status.flutter_version.as_deref().unwrap_or("None") {
                println!("Flutter version is already {project_version}");
                return;
            }
            println!("Flutter version is not synced with project version. Syncing...");

            match change_flutter_version(&project_version, &status).await {
                Ok(_) => {}
                Err(e) => {
                    println!("Could not change flutter version\\nError: {}", e);
                    return;
                }
            };
            status.update().await;
            status.print();
            return;
        }
        _ => {
            println!("No project version found. Please specify a version with --desiredVersion or set the project version in pubspec.yaml");
            return;
        }
    }

    // let test = shell_run("flutter --version", &None).await;
    // println!("Test: {}", test);
}
