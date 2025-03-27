use clap::Parser;
use std::env;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[arg(short = 'd', long = "workingDirectory", value_hint = clap::ValueHint::DirPath)]
    working_dir: Option<PathBuf>,
    #[arg(short = 'v', long = "desiredVersion")]
    desired_version: Option<String>,
}

fn main() {
    run(env::args().collect());
}

fn run(args: Vec<String>) {
    let args = Args::parse_from(args);

    println!("Flutter rust checker version {}", env!("CARGO_PKG_VERSION"));

    match args.working_dir {
        Some(working_dir) => {
            env::set_current_dir(working_dir).unwrap();
        }
        None => {
            println!("No working directory specified, using current directory");
        }
    };

    let current_dir = env::current_dir().unwrap();

    println!("Current directory is: {:?}", current_dir);

    match args.desired_version {
        Some(desired_version) => {
            if !desired_version.trim().is_empty() {
                println!("DesiredVersion: {}", desired_version)
            }
        }
        None => {}
    }
}
