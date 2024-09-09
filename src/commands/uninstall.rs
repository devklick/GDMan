use std::{fs, str::FromStr};

use clap::Parser;

use crate::{
    clap_enum_variants,
    common::{Architecture, Flavour},
    gd_semver::MaybeVersionOrVersionReq,
    gdman::{self, GodotVersionInfo},
};

use super::common::RunCommand;

#[derive(Parser)]
pub struct UninstallVersionsCommand {
    #[arg(short, long, help = "Specifies the version(s) to uninstall", value_parser=MaybeVersionOrVersionReq::from_str, group = "version_filter")]
    version: Option<MaybeVersionOrVersionReq>,

    #[arg(short, long, help = "Specifies the target architecture version to uninstall", value_enum,  value_parser=clap_enum_variants!(Architecture))]
    architecture: Option<Architecture>,

    #[arg(short, long, help = "The \"flavour\" (for lack of a better name) of version to uninstall", value_enum, value_parser=clap_enum_variants!(Flavour))]
    flavour: Option<Flavour>,

    #[arg(long, help = "Allows multiple versions to be uninstalled")]
    force: bool,

    #[arg(
        short,
        long,
        help = "Uninstall all versions other than the currently-active version",
        group = "version_filter"
    )]
    unused: bool,
}

impl RunCommand for UninstallVersionsCommand {
    async fn run(self) -> Result<(), String> {
        let current_version = gdman::get_current_version()?;
        let current_version_dir = current_version.path.parent().unwrap().to_path_buf();
        let mut err: Option<String> = None;
        let mut candidates: Vec<GodotVersionInfo> = Vec::new();

        if !self.unused && self.version.is_none() {
            return Err("--version or --unused must be specified".to_owned());
        }

        for version in gdman::get_installed_versions()? {
            if self.unused {
                if current_version_dir != version.path {
                    candidates.push(version);
                }
                continue;
            }

            let v = self.version.clone().unwrap();
            match v.version_exact {
                Some(exact) => {
                    if exact != version.name_parts.version {
                        continue;
                    };
                }
                None => {
                    if !v.version_like.matches(&version.name_parts.version) {
                        continue;
                    }
                }
            }
            if let Some(architecture) = &self.architecture {
                if architecture != &version.name_parts.architecture {
                    continue;
                }
            }
            if let Some(flavour) = &self.flavour {
                if flavour != &version.name_parts.flavour {
                    continue;
                }
            }
            candidates.push(version);
        }

        let candidate_count = candidates.len();
        if candidate_count == 0 {
            log::info!("No versions found to uninstall");
            return Ok(());
        }

        for version in candidates.clone() {
            log::info!("Found {} for uninstall", version.name_parts.version_name);
        }

        if candidate_count > 1 && !&self.force && !&self.unused {
            return Err(
                "Multiple versions can only be uninstalled with the --force or --unused args"
                    .to_owned(),
            );
        }

        for version in candidates {
            if version.path == current_version_dir {
                err = Some(format!(
                    "Cannot uninstall version {} because it is currently active",
                    version.name_parts.version_name
                ));
            } else {
                if let Err(err) = fs::remove_dir_all(version.path) {
                    return Err(format!(
                        "Error uninstalling version {}\n{}",
                        version.name_parts.version_name, err
                    ));
                }
                log::info!("Uninstalled version {}", version.name_parts.version_name);
            }
        }

        return match err {
            None => Ok(()),
            Some(err) => Err(err),
        };
    }
}
