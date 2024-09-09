use clap::Parser;

use crate::gdman;

use super::common::RunCommand;

#[derive(Parser)]
pub struct CurrentVersionCommand {}

impl RunCommand for CurrentVersionCommand {
    async fn run(self) -> Result<(), String> {
        match gdman::get_current_version() {
            Err(err) => {
                if err.starts_with("Cant determine current version, godot link not found at") {
                    log::info!("No version active");
                    return Ok(());
                }
                return Err("".to_string());
            }
            Ok(current) => {
                log::info!("{}", current.name_parts.version_name);
                return Ok(());
            }
        };
    }
}
