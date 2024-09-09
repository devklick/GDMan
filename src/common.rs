use std::env;
use strum_macros::{Display, EnumString, VariantNames};

pub trait FromOS<T> {
    fn from_os() -> Result<T, String>;
}

#[derive(Clone, Debug, PartialEq, Display)]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
}

impl FromOS<Platform> for Platform {
    fn from_os() -> Result<Platform, String> {
        match env::consts::OS {
            "linux" => Ok(Platform::Linux),
            "windows" => Ok(Platform::Windows),
            "macos" => Ok(Platform::MacOS),
            _ => Err(format!("Unsupported platform {}", env::consts::OS)),
        }
    }
}

#[derive(EnumString, VariantNames, clap::ValueEnum, Clone, Debug, PartialEq, Display, Copy)]
#[strum(serialize_all = "lowercase")]
pub enum Flavour {
    Standard,
    Mono,
}

#[derive(EnumString, VariantNames, clap::ValueEnum, Clone, Debug, PartialEq, Display)]
#[strum(serialize_all = "lowercase")]
pub enum Architecture {
    #[cfg(unix)]
    Arm32,
    #[cfg(unix)]
    Arm64,
    #[cfg(any(unix, windows))]
    X86,
    #[cfg(any(unix, windows))]
    X64,
    #[cfg(target_os = "macos")]
    Universal,
}

impl FromOS<Architecture> for Architecture {
    fn from_os() -> Result<Architecture, String> {
        #[cfg(target_os = "macos")]
        if Platform::from_os().unwrap() == Platform::MacOS {
            return Ok(Architecture::Universal);
        }

        // TODO: This is probably way off, but it's a starting point
        match env::consts::ARCH {
            #[cfg(unix)]
            "arm" => Ok(Architecture::Arm32),
            #[cfg(unix)]
            "aarch64" => Ok(Architecture::Arm64),
            #[cfg(any(unix, windows))]
            "x86" => Ok(Architecture::X86),
            #[cfg(any(unix, windows))]
            "x86_64" => Ok(Architecture::X64),
            _ => Err("Unsupported architecture".to_owned()),
        }
    }
}
