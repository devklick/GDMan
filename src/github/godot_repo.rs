use crate::{
    common::{Architecture, Flavour, Platform},
    gd_semver::parse_semver_version,
    github::github_repo as gh,
};

const OWNER: &str = "godotengine";
const REPO: &str = "godot";

pub async fn find_release_with_asset(
    version_exact: &Option<semver::Version>,
    version_like: &Option<semver::VersionReq>,
    platform: &Platform,
    architecture: &Architecture,
    flavour: &Flavour,
    client: &reqwest::Client,
) -> Result<gh::Release, String> {
    let mut asset_name_checks: Vec<String> = Vec::new();

    log::trace!("Determining checks to identify target release");

    match generate_asset_name(platform, architecture, flavour) {
        Ok(asset_name) => asset_name_checks.push(asset_name),
        Err(err) => return Err(err),
    }

    return gh::find_release_with_asset(
        OWNER,
        REPO,
        version_exact,
        version_like,
        asset_name_checks,
        client,
    )
    .await;
}

pub struct GodotVersionNameParts {
    pub version_string: String,
    pub version: semver::Version,
    pub version_name: String,
    pub platform: Platform,
    pub architecture: Architecture,
    pub flavour: Flavour,
}

impl Clone for GodotVersionNameParts {
    fn clone(&self) -> Self {
        Self {
            version_string: self.version_string.clone(),
            version: self.version.clone(),
            version_name: self.version_name.clone(),
            platform: self.platform.clone(),
            architecture: self.architecture.clone(),
            flavour: self.flavour.clone(),
        }
    }
}

pub fn parse_version_name(version_name: &str) -> Result<GodotVersionNameParts, String> {
    log::trace!("Parsing version name {version_name}");

    let (version_string, version) = parse_version_from_version_name(version_name)?;
    log::trace!(
        "Found version name to contain version {} (raw version {})",
        version.to_string(),
        version_string
    );
    let flavour = parse_flavour_from_version_name(version_name)?;
    log::trace!("Found version name to contain flavour {flavour}");

    let platform = parse_platform_from_version_name(version_name)?;
    log::trace!("Found version name to contain platform {platform}");

    let architecture = parse_architecture_from_version_name(version_name, &platform)?;
    log::trace!("Found version name to contain architecture {architecture}");

    return Ok(GodotVersionNameParts {
        architecture,
        flavour,
        platform,
        version,
        version_name: version_name.to_string(),
        version_string,
    });
}

fn parse_version_from_version_name(
    version_name: &str,
) -> Result<(String, semver::Version), String> {
    let start = match version_name.find("_v") {
        None => return Err("Invalid version name".to_owned()),
        Some(s) => s,
    };
    let end = match version_name[start + 1..].find("_") {
        None => return Err("Invalid version name".to_owned()),
        Some(s) => s,
    };

    let str_start = start + 2;
    let str_end = start + end + 1;
    let version_string = version_name[str_start..str_end].to_owned();

    let semver_version = parse_semver_version(&version_string, &Some(vec!["stable".to_owned()]))?;

    return Ok((version_string, semver_version));
}

fn parse_flavour_from_version_name(version_name: &str) -> Result<Flavour, String> {
    return match version_name.contains("mono") {
        true => Ok(Flavour::Mono),
        false => Ok(Flavour::Standard),
    };
}

fn parse_platform_from_version_name(version_name: &str) -> Result<Platform, String> {
    if version_name.contains("win32") || version_name.contains("win64") {
        return Ok(Platform::Windows);
    }
    if version_name.contains("macos") {
        return Ok(Platform::MacOS);
    }
    if version_name.contains("linux") {
        return Ok(Platform::Linux);
    }
    return Err("Invalid version name".to_owned());
}

fn parse_architecture_from_version_name(
    version_name: &str,
    platform: &Platform,
) -> Result<Architecture, String> {
    match platform {
        Platform::MacOS => return Ok(Architecture::Universal),
        Platform::Windows => {
            if version_name.contains("win32") {
                return Ok(Architecture::X86);
            }
            if version_name.contains("win64") {
                return Ok(Architecture::X64);
            }
            return Err("Invalid version name".to_owned());
        }
        Platform::Linux => {
            if version_name.contains("arm32") {
                return Ok(Architecture::Arm32);
            }
            if version_name.contains("arm64") {
                return Ok(Architecture::Arm64);
            }
            if version_name.contains("x86_64") {
                return Ok(Architecture::X64);
            }
            if version_name.contains("x86_32") {
                return Ok(Architecture::X86);
            }
            return Err("Invalid version name".to_owned());
        }
    }
}

pub fn generate_version_name(
    version: &str,
    platform: &Platform,
    architecture: &Architecture,
    flavour: &Flavour,
) -> Result<String, String> {
    match generate_asset_name(platform, architecture, flavour) {
        Err(e) => Err(e),
        Ok(asset_name) => Ok(["Godot_v", &version, "_", &asset_name].join("")),
    }
}

// Generates the expected name of the Godot download asset.
// This reverse engineers the naming convention used by Godot.
// The naming convention is likely to change at some point, so we may
// need to implement different logic depending on the target version
fn generate_asset_name(
    platform: &Platform,
    architecture: &Architecture,
    flavour: &Flavour,
) -> Result<String, String> {
    let mut parts: Vec<&str> = Vec::new();

    if *flavour == Flavour::Mono {
        parts.push(&"mono_");
    }

    match platform {
        Platform::Windows => {
            parts.push("win");

            match architecture {
                Architecture::X64 => parts.push("64"),
                Architecture::X86 => parts.push("32"),
                _ => {
                    return Err(format!(
                        "Architecture {architecture} not supported on {platform} platform"
                    )
                    .to_owned())
                }
            }
            if *flavour != Flavour::Mono {
                parts.push(".exe");
            }
        }
        Platform::Linux => {
            parts.push("linux");

            match flavour {
                Flavour::Mono => parts.push("_"),
                _ => parts.push("."),
            }
            match architecture {
                Architecture::Arm32 => parts.push("arm32"),
                Architecture::Arm64 => parts.push("arm64"),
                Architecture::X64 => parts.push("x86_64"),
                Architecture::X86 => parts.push("x86_32"),
                _ => {
                    return Err(format!(
                        "Architecture {architecture} not supported on {platform} platform"
                    )
                    .to_owned())
                }
            }
        }
        Platform::MacOS => {
            parts.push("macos.universal");
        }
    }

    return Ok(parts.join(""));
}
