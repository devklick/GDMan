use clap::Parser;

use crate::gdman;

use super::common::RunCommand;

#[derive(Parser)]
pub struct CurrentVersionCommand {}

impl RunCommand for CurrentVersionCommand {
    async fn run(self) -> Result<(), String> {
        let current = gdman::get_current_version()?;
        log::info!("{}", current.name_parts.version_name);
        return Ok(());
    }
}
