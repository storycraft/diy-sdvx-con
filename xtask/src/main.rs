use core::error::Error;
use std::{env, process::Command};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Cmd {
    #[command(about = "Build and flash firmware")]
    Flash {
        #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
        cargo_args: Vec<String>,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    match Cmd::parse() {
        Cmd::Flash { cargo_args } => {
            cargo_cmd()
                .current_dir("./firmware")
                .args(["run", "-p", "firmware"])
                .args(cargo_args)
                .status()?;
        }
    }

    Ok(())
}

fn cargo_cmd() -> Command {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    Command::new(cargo)
}
