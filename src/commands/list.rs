use crate::gdman;
use clap::Parser;

use super::common::RunCommand;

#[derive(Parser)]
pub struct ListVersionsCommand {}

impl RunCommand for ListVersionsCommand {
    async fn run(self) -> Result<(), String> {
        gdman::get_installed_versions()?
            .iter()
            .for_each(|version| log::info!("{}", version.name_parts.version_name));

        return Ok(());
    }
}
