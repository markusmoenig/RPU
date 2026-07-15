use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rpu")]
#[command(about = "RPU CLI for running and building portable cartridges.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    New {
        name: String,
        #[arg(long)]
        path: Option<PathBuf>,
    },
    Run {
        #[arg(default_value = ".")]
        cartridge: PathBuf,
        #[arg(last = true)]
        args: Vec<String>,
    },
    Build {
        #[arg(default_value = ".")]
        cartridge: PathBuf,
    },
    BuildWeb {
        #[arg(default_value = ".")]
        cartridge: PathBuf,
    },
    ServeWeb {
        #[arg(default_value = ".")]
        cartridge: PathBuf,
        #[arg(long, default_value_t = 8000)]
        port: u16,
    },
    ExportXcode {
        #[arg(default_value = ".")]
        cartridge: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let exit_code = match cli.command {
        Command::New { name, path } => {
            rpu_build::new_project(&name, path.as_deref())?;
            0
        }
        Command::Run { cartridge, args } => rpu_build::run_project(&cartridge, &args)?,
        Command::Build { cartridge } => {
            rpu_build::build_project(&cartridge)?;
            0
        }
        Command::BuildWeb { cartridge } => {
            rpu_build::build_web_project(&cartridge)?;
            0
        }
        Command::ServeWeb { cartridge, port } => {
            rpu_build::serve_web_project(&cartridge, port)?;
            0
        }
        Command::ExportXcode { cartridge, output } => {
            rpu_build::export_xcode(&cartridge, output.as_deref())?;
            0
        }
    };

    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    Ok(())
}
