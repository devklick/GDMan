use chrono::{DateTime, Utc};
use regex::Regex;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde::Serialize;
use std::borrow::Borrow;

use crate::gd_semver::parse_semver_version;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Release {
    pub url: String,
    // pub assets_url: String,
    // pub upload_url: String,
    // pub html_url: String,
    // pub id: i64,
    // pub author: Author,
    // pub node_id: String,
    pub tag_name: String,
    // pub target_commitish: String,
    // pub name: String,
    // pub draft: bool,
    // pub prerelease: bool,
    // pub created_at: DateTime<Utc>,
    pub published_at: DateTime<Utc>,
    pub assets: Vec<Asset>,
    // pub tarball_url: String,
    // pub zipball_url: String,
    // pub body: String,
    // pub reactions: Option<Reactions>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asset {
    pub url: String,
    // pub id: i64,
    // pub node_id: String,
    pub name: String,
    // pub label: Option<String>,
    // pub uploader: Author,
    // pub content_type: ContentType,
    // pub state: State,
    pub size: i64,
    // pub download_count: i64,
    // pub created_at: DateTime<Utc>,
    // pub updated_at: DateTime<Utc>,
    pub browser_download_url: String,
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub enum ContentType {
//     #[serde(rename = "application/octet-stream")]
//     ApplicationOctetStream,
//     #[serde(rename = "application/vnd.android.package-archive")]
//     ApplicationVndAndroidPackageArchive,
//     #[serde(rename = "application/x-compressed")]
//     ApplicationXCompressed,
//     #[serde(rename = "application/x-xz")]
//     ApplicationXXz,
//     #[serde(rename = "application/x-zip-compressed")]
//     ApplicationXZipCompressed,
//     #[serde(rename = "application/zip")]
//     ApplicationZip,
//     #[serde(rename = "text/plain")]
//     TextPlain,
//     #[serde(rename = "text/plain; charset=utf-8")]
//     TextPlainCharsetUtf8,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// #[serde(rename_all = "snake_case")]
// pub enum State {
//     Uploaded,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct Author {
//     pub login: String,
//     pub id: i64,
//     pub node_id: String,
//     pub avatar_url: String,
//     pub gravatar_id: String,
//     pub url: String,
//     pub html_url: String,
//     pub followers_url: String,
//     pub following_url: String,
//     pub gists_url: String,
//     pub starred_url: String,
//     pub subscriptions_url: String,
//     pub organizations_url: String,
//     pub repos_url: String,
//     pub events_url: String,
//     pub received_events_url: String,
//     #[serde(rename = "type")]
//     pub author_type: Type,
//     pub site_admin: bool,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub enum Type {
//     User,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct Reactions {
//     pub url: String,
//     pub total_count: i64,
//     #[serde(rename = "+1")]
//     pub vote_up: i64,
//     #[serde(rename = "-1")]
//     pub vote_down: i64,
//     pub laugh: i64,
//     pub hooray: i64,
//     pub confused: i64,
//     pub heart: i64,
//     pub rocket: i64,
//     pub eyes: i64,
// }

const BASE_URL: &str = "https://api.github.com";

pub async fn get_releases(
    owner: &str,
    repo: &str,
    client: &reqwest::Client,
) -> Result<Vec<Release>, reqwest::Error> {
    let mut headers = HeaderMap::with_capacity(3);
    headers.append("User-Agent", "request".parse().unwrap());
    headers.append("Accept", "application/vnd.github+json".parse().unwrap());
    headers.append("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());

    let url = format!("{BASE_URL}/repos/{owner}/{repo}/releases");

    log::trace!("Fetching releases from {url}");

    let request = client.get(url).headers(headers);
    let releases: Vec<Release> = request.send().await?.json().await?;

    return Ok(releases);
}

pub async fn find_release(
    owner: &str,
    repo: &str,
    version_exact: &Option<semver::Version>,
    version_like: &Option<semver::VersionReq>,
    client: &reqwest::Client,
) -> Result<Release, String> {
    let releases = match get_releases(owner, repo, client).await {
        Err(e) => return Err(e.to_string()),
        Ok(r) => r,
    };

    let release: Option<Release>;

    // If the caller has specified an exact version, we need to look through all
    // releases, parse the semver from the tag, and check if it's equal to the
    // specified exact version
    if let Some(version_exact) = version_exact {
        release = find_exact_release_version(&releases, version_exact);
    } else if let Some(version_like) = version_like {
        release = find_latest_release(&releases, &Some(version_like));
    } else {
        release = find_latest_release(&releases, &None);
    }

    return match release {
        None => Err("No matching release found".to_owned()),
        Some(release) => {
            log::info!("Found release with version {}", release.tag_name);
            return Ok(release);
        }
    };
}

pub async fn find_release_with_asset(
    owner: &str,
    repo: &str,
    version_exact: &Option<semver::Version>,
    version_like: &Option<semver::VersionReq>,
    asset_name_like: Vec<String>,
    client: &reqwest::Client,
) -> Result<Release, String> {
    let mut release = find_release(owner, repo, version_exact, version_like, client).await?;

    log::trace!("Finding release asset");

    let name_checks = asset_name_like
        .into_iter()
        .map(|cur| Regex::new(&cur).unwrap())
        .into_iter();

    let assets: Vec<Asset> = release
        .assets
        .into_iter()
        .filter(|a| name_checks.clone().all(|n| n.is_match(&a.name)))
        .collect();

    if assets.is_empty() {
        return Err(format!(
            "No assets found for release {} repo {repo}",
            release.tag_name
        )
        .to_owned());
    }

    let asset = assets.first().unwrap().clone();

    log::info!("Found release asset {}", asset.name);

    release.assets = Vec::new();
    release.assets.push(asset);

    return Ok(release);
}

fn find_exact_release_version(
    releases: &Vec<Release>,
    version_exact: &semver::Version,
) -> Option<Release> {
    log::trace!("Finding release matching exact version {version_exact}");
    for release in releases {
        let release_version =
            match parse_semver_version(&release.tag_name, &Some(vec!["stable".to_owned()])) {
                Err(_) => {
                    log::warn!(
                        "Release tag {} does not indicate a valid version. Skipping",
                        &release.tag_name
                    );
                    continue;
                }
                Ok(v) => v,
            };
        if release_version == *version_exact {
            return Some(release.clone());
        }
    }
    return None;
}

struct Candidate {
    release: Release,
    version: semver::Version,
}

fn find_latest_release(
    releases: &[Release],
    version_like: &Option<&semver::VersionReq>,
) -> Option<Release> {
    match version_like {
        Some(v) => log::trace!("Finding release matching version {v}"),
        None => log::trace!("Finding latest version"),
    }

    let mut candidate: Option<Candidate> = None;
    let ignore_pre_releases = Some(vec!["stable".to_owned()]);
    for release in releases {
        let release_version = match parse_semver_version(&release.tag_name, &ignore_pre_releases) {
            Err(_) => {
                log::warn!(
                    "Release tag {} does not indicate a valid version. Skipping",
                    &release.tag_name
                );
                continue;
            }
            Ok(v) => v,
        };

        if version_like.is_none() || version_like.unwrap().matches(&release_version) {
            if let Some(c) = candidate.borrow() {
                if release_version > c.version {
                    candidate = Some(Candidate {
                        release: release.clone(),
                        version: release_version,
                    });
                }
            } else {
                candidate = Some(Candidate {
                    release: release.clone(),
                    version: release_version,
                });
            }
        }
    }
    return match candidate {
        None => None,
        Some(candidate) => Some(candidate.release),
    };
}
