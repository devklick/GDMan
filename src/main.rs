mod cli;
mod commands;
mod common;
mod gd_semver;
mod gdman;
mod github;

use std::process::ExitCode;

use clap::Parser;
use fern::colors::{Color, ColoredLevelConfig};

use cli::Commands;
use commands::common::RunCommand;

#[tokio::main]
async fn main() -> ExitCode {
    let args = cli::Args::parse();

    if let Err(init_err) = init_logger(args.verbose) {
        println!("Failed to initialize logger\n{init_err}");
        return ExitCode::FAILURE;
    }

    let res = match args.command {
        Commands::Install(install) => install.run().await,
        Commands::Uninstall(uninstall) => uninstall.run().await,
        Commands::Current(current) => current.run().await,
        Commands::List(list) => list.run().await,
        Commands::Update(update) => update.run().await,
    };

    return match res {
        Err(e) => {
            log::error!("Failed!\n{e}");
            ExitCode::FAILURE
        }
        Ok(_) => ExitCode::SUCCESS,
    };
}

fn init_logger(verbose: bool) -> Result<(), String> {
    let log_level = match verbose {
        true => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,
    };
    let colors = ColoredLevelConfig::new()
        .trace(Color::Magenta)
        .error(Color::Red)
        .debug(Color::Blue);

    let mut builder = fern::Dispatch::new()
        .format(move |out, message, record| {
            let level = record.level();
            if level == log::Level::Info {
                out.finish(format_args!("{}", message));
            } else {
                out.finish(format_args!("{} {}", colors.color(level), message));
            }
        })
        .level(log_level)
        .chain(std::io::stdout());

    if !cfg!(debug_assertions) {
        builder = builder.level_for("lnk", log::LevelFilter::Off);
    }

    return match builder.apply() {
        Err(e) => Err(e.to_string()),
        Ok(_) => Ok(()),
    };
}
