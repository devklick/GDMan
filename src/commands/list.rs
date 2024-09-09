use crate::gdman;
use clap::Parser;

use super::common::RunCommand;

#[derive(Parser)]
pub struct ListVersionsCommand {}

impl RunCommand for ListVersionsCommand {
    async fn run(self) -> Result<(), String> {
        let versions = gdman::get_installed_versions()?;

        if versions.len() == 0 {
            log::info!("No versions installed");
        } else {
            for version in versions {
                log::info!("{}", version.name_parts.version_name);
            }
        }

        return Ok(());
    }
}
