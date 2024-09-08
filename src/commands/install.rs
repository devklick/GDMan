use std::str::FromStr;

use reqwest::Client;

use crate::clap_enum_variants;
use crate::gd_semver::flatten_version;
use crate::github::godot_repo as gd;

use crate::common::{Architecture, Flavour, FromOS, Platform};
use crate::{gd_semver::MaybeVersionOrVersionReq, gdman};

use clap::{Args, Parser};

use super::common::RunCommand;

#[derive(Args)]
#[group(required = true, multiple = false)]
struct VersionOrLatest {
    #[arg(
        short,
        long,
        help = "Installs the latest version",
        group = "version_or_latest"
    )]
    latest: bool,

    #[arg(short, long, help = "Specifies the version to install", value_parser=MaybeVersionOrVersionReq::from_str, group="version_or_latest")]
    version: Option<MaybeVersionOrVersionReq>,
}

#[derive(Parser)]
pub struct InstallVersionCommand {
    #[command(flatten)]
    version_or_latest: VersionOrLatest,

    #[arg(short, long, help = "Specifies the target platform", value_enum, default_value_t=Platform::from_os().unwrap(), value_parser=clap_enum_variants!(Platform))]
    platform: Platform,

    #[arg(short, long, help = "Specifies the target architecture", value_enum, default_value_t=Architecture::from_os().unwrap(), value_parser=clap_enum_variants!(Architecture))]
    architecture: Architecture,

    #[arg(short, long, help = "The \"flavour\" (for lack of a better name) of version to install", value_enum, default_value_t=Flavour::Standard, value_parser=clap_enum_variants!(Flavour))]
    flavour: Flavour,
}

impl RunCommand for InstallVersionCommand {
    async fn run(self) -> Result<(), String> {
        let (version_input, version_like, version_exact) =
            flatten_version(&self.version_or_latest.version);

        if let Some(_) = version_exact {
            if gdman::activate_by_parts_if_installed(
                &version_input,
                &self.platform,
                &self.architecture,
                &self.flavour,
            )? {
                return Ok(());
            }
        }

        let client = Client::new();

        let release = gd::find_release_with_asset(
            &version_exact,
            &version_like,
            &self.platform,
            &self.architecture,
            &self.flavour,
            &client,
        )
        .await?;

        let asset = release.assets.first().unwrap();
        let version_name = asset.name.trim_end_matches(".zip");

        if gdman::activate_by_name_if_installed(version_name)? {
            return Ok(());
        }

        gdman::download_godot_version(version_name, &client, &asset.browser_download_url).await?;

        return Ok(());
    }
}
