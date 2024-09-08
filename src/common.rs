use std::env;
use strum_macros::{Display, EnumString, VariantNames};

pub trait FromOS<T> {
    fn from_os() -> Result<T, String>;
}

#[derive(EnumString, VariantNames, clap::ValueEnum, Clone, Debug, PartialEq, Display)]
#[strum(serialize_all = "lowercase")]
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
            _ => Err("Unsupported platform".to_owned()),
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
    Arm32,
    Arm64,
    X86,
    X64,
    Universal,
}

impl FromOS<Architecture> for Architecture {
    fn from_os() -> Result<Architecture, String> {
        // TODO: This is probably way off, but it's a starting point
        match env::consts::ARCH {
            "arm" => Ok(Architecture::Arm32),
            "x86" => Ok(Architecture::X86),
            "x86_64" => Ok(Architecture::X64),
            _ => Err("Unsupported architecture".to_owned()),
        }
    }
}
