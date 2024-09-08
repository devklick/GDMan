/*
    Wrapper around semver to provide some custom functionality,
    since Godot versions dont follow the exact versioning that
    rust semver uses.
*/

use std::str::FromStr;

use regex::Regex;
use semver::VersionReq;

const VERSION_REGEX: &str = r"^(?<major>0|[1-9]\d*)(\.(?<minor>0|[1-9]\d*))?(\.(?<patch>0|[1-9]\d*))?(?:-(?<pre>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?<meta>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$";

#[derive(Clone, Debug)]
pub struct MaybeVersionOrVersionReq {
    pub input_str: String,
    pub version_like: semver::VersionReq,
    pub version_exact: Option<semver::Version>,
}

impl FromStr for MaybeVersionOrVersionReq {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        return match VersionReq::parse(s) {
            Ok(r) => match parse_semver_version(s, &None) {
                Err(_) => Ok(MaybeVersionOrVersionReq {
                    version_like: r,
                    version_exact: None,
                    input_str: s.to_string(),
                }),
                Ok(v) => Ok(MaybeVersionOrVersionReq {
                    version_exact: Some(v),
                    version_like: r,
                    input_str: s.to_string(),
                }),
            },
            Err(_) => Err(format!("Invalid version: {s}").to_owned()),
        };
    }
}

pub fn parse_semver_version(
    value: &str,
    ignored_pre_releases: &Option<Vec<String>>,
) -> Result<semver::Version, String> {
    let reg = match Regex::new(&VERSION_REGEX) {
        Err(e) => return Err(e.to_string().to_owned()),
        Ok(r) => r,
    };

    if !reg.is_match(&value) {
        return Err(format!("Invalid version {value}"));
    }

    let captures = reg.captures(value).unwrap();
    // major is required
    let maj = match_or_default(value, captures.name("major"), 0);
    // minor is optional for godot but required by semver - default it to zero
    // e.g. version 1 = 1.0.0
    let min = match_or_default(value, captures.name("minor"), 0);
    // minor is optional for godot but required by semver - default it to zero
    // e.g. version 1.1 = 1.1.0
    let patch = match_or_default(value, captures.name("patch"), 0);

    // version string so far
    let mut version_str = [maj, min, patch].map(|v| v.to_string()).join(".");

    // if we've found a pre-release flag that's not ignored
    if let Some(pre_match) = captures.name("pre") {
        let pre = &value[pre_match.start()..pre_match.end()].to_owned();
        if let Some(ignores) = ignored_pre_releases {
            if !ignores.contains(pre) {
                version_str = version_str + "-" + pre;
            }
        } else {
            version_str = version_str + "-" + pre;
        }
    }
    return match semver::Version::parse(&version_str) {
        Err(e) => Err(e.to_string()),
        Ok(v) => Ok(v),
    };
}

fn match_or_default<T: FromStr>(input: &str, r_match: Option<regex::Match>, default: T) -> T
where
    <T as FromStr>::Err: std::fmt::Debug,
{
    return match r_match {
        None => default,
        Some(v) => input[v.start()..v.end()].parse::<T>().unwrap(),
    };
}

pub fn flatten_version(
    version: &Option<MaybeVersionOrVersionReq>,
) -> (
    Option<String>,
    Option<semver::VersionReq>,
    Option<semver::Version>,
) {
    log::trace!("Checking input version");

    let version = match version {
        Some(v) => v,
        None => {
            log::trace!("No input version specified");
            return (None, None, None);
        }
    };

    log::trace!("Flattening input version");

    let input = version.input_str.clone();

    let (like, exact) = match &version.version_exact {
        Some(v) => (Some(version.version_like.to_owned()), Some(v.clone())),
        None => (Some(version.version_like.to_owned()), None),
    };

    log::trace!(
        "Identified version specified as input = {}, exact = {}, like = {}",
        &input,
        &exact.clone().map_or("none".to_owned(), |f| f.to_string()),
        &like.clone().map_or("none".to_owned(), |f| f.to_string())
    );

    return (Some(input), like, exact);
}
