use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rpu")]
#[command(about = "RPU CLI for running and building scene-driven 2D apps.")]
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
        project: PathBuf,
    },
    Build {
        #[arg(default_value = ".")]
        project: PathBuf,
    },
    BuildWeb {
        #[arg(default_value = ".")]
        project: PathBuf,
    },
    ServeWeb {
        #[arg(default_value = ".")]
        project: PathBuf,
        #[arg(long, default_value_t = 8000)]
        port: u16,
    },
    ExportXcode {
        #[arg(default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::New { name, path } => rpu_build::new_project(&name, path.as_deref())?,
        Command::Run { project } => rpu_build::run_project(&project)?,
        Command::Build { project } => rpu_build::build_project(&project)?,
        Command::BuildWeb { project } => rpu_build::build_web_project(&project)?,
        Command::ServeWeb { project, port } => rpu_build::serve_web_project(&project, port)?,
        Command::ExportXcode { project, output } => {
            rpu_build::export_xcode(&project, output.as_deref())?
        }
    }

    Ok(())
}
