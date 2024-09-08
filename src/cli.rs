use clap::{Parser, Subcommand};

use crate::commands::{
    current::CurrentVersionCommand, install::InstallVersionCommand, list::ListVersionsCommand,
    uninstall::UninstallVersionsCommand, update::UpdateVersionCommand,
};

#[derive(Parser)]
#[command(name = "GDMan")]
#[command(version)]
#[command(author = "devklick")]
#[command(about = "Godot version manager")]
#[command(long_about = "Manage versions of Godot installed on your system via CLI")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(
        long,
        help = "Enables more detailed application logging",
        global = true
    )]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Installs the specified version")]
    Install(InstallVersionCommand),

    #[command(about = "Uninstalls the specified version(s) of Godot")]
    Uninstall(UninstallVersionsCommand),

    #[command(about = "Lists the versions of Godot currently installed on the system")]
    List(ListVersionsCommand),

    #[command(about = "Prints the currently-active version of Godot")]
    Current(CurrentVersionCommand),

    #[command(about = "Update the currently-active version of Godot")]
    Update(UpdateVersionCommand),
}
