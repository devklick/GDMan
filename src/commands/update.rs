use clap::{Args, Parser};
use reqwest::Client;

use crate::gdman;
use crate::github::godot_repo as gd;

use super::common::RunCommand;

#[derive(Args)]
#[group(required = true, multiple = false)]
struct UpdateType {
    #[arg(
        long,
        help = "Updates the currently-active version to the latest patch (default)",
        group = "update_type"
    )]
    patch: bool,
    #[arg(
        long,
        help = "Updates the currently-active version to the latest minor revision",
        group = "update_type"
    )]
    minor: bool,
    #[arg(
        long,
        help = "Updates the currently-active version to the latest major revision",
        group = "update_type"
    )]
    major: bool,
}

#[derive(Parser)]
pub struct UpdateVersionCommand {
    #[command(flatten)]
    update_type: UpdateType,

    #[arg(
        long,
        help = "Uninstall the current version after installing the updated version",
        default_value_t = false
    )]
    uninstall: bool,
}

impl RunCommand for UpdateVersionCommand {
    async fn run(self) -> Result<(), String> {
        let current = gdman::get_current_version()?;
        let current_version_string = current.name_parts.version.to_string();

        log::trace!(
            "Active version of Godot is {}, (platform = {}, architecture = {}, flavour = {})",
            current_version_string,
            current.name_parts.platform,
            current.name_parts.architecture,
            current.name_parts.flavour
        );

        let new_version_like = match (
            &self.update_type.major,
            &self.update_type.minor,
            &self.update_type.patch,
        ) {
            // major = get latest
            (true, false, false) => None,
            // minor
            (false, true, false) => Some(
                semver::VersionReq::parse(format!("^{}", current_version_string).as_str()).unwrap(),
            ),
            // patch or no arg specified, default to patch
            _ => Some(
                semver::VersionReq::parse(format!("~{}", current_version_string).as_str()).unwrap(),
            ),
        };

        if let Some(ref v) = new_version_like {
            log::trace!("Looking for new version like {}", v)
        } else {
            log::trace!("Looking for latest version");
        }

        let client = Client::new();

        let release = gd::find_release_with_asset(
            &None,
            &new_version_like,
            &current.name_parts.platform,
            &current.name_parts.architecture,
            &current.name_parts.flavour,
            &client,
        )
        .await?;

        let asset = release.assets.first().unwrap();
        let version_name = asset.name.trim_end_matches(".zip");

        if gdman::activate_by_name_if_installed(version_name)? {
            if self.uninstall {
                gdman::uninstall_version(&current)?;
            }
            return Ok(());
        }

        gdman::download_godot_version(version_name, &client, &asset.browser_download_url).await?;

        gdman::set_active_godot_version(version_name)?;

        if self.uninstall {
            gdman::uninstall_version(&current)?;
        }

        return Ok(());
    }
}
