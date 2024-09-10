use std::{
    env,
    fs::{self, remove_file, DirEntry},
    path::{Path, PathBuf},
};

use async_zip::tokio::read::seek::ZipFileReader;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{header, Url};
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    io::{AsyncWriteExt, BufReader},
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::{
    common::{Architecture, Flavour, Platform},
    github::godot_repo::{self as gd, parse_version_name, GodotVersionNameParts},
};

pub fn set_active_godot_version(version_name: &str) -> Result<(), String> {
    log::trace!("Setting active Godot version to {version_name}");

    let link_path = get_godot_link_path()?;

    remove_link(&link_path)?;

    let version_dir_path = get_versions_dir()?;
    let target_version_dir = version_dir_path.join(version_name);

    if !target_version_dir.is_dir() {
        return Err(format!(
            "Version directory {} does not exist",
            target_version_dir.display()
        ));
    }

    let target_exe_path = get_godot_exe_path(&target_version_dir)?;

    log::trace!("Linking godot command to {}", target_exe_path.display());

    create_link(&link_path, &target_exe_path)?;

    log::info!("Set {version_name} active");

    return Ok(());
}

pub fn already_installed(version_name: &str) -> bool {
    let mut dir_path = match get_versions_dir() {
        Err(_) => return false,
        Ok(d) => d,
    };

    dir_path.push(version_name);

    log::trace!("Checking if {} is installed", dir_path.display());

    if !dir_path.is_dir() {
        log::trace!("{} doesn't exist", dir_path.display());
        return false;
    }

    return match get_godot_exe_path(&dir_path) {
        Err(_) => {
            log::trace!("{} not yet installed", dir_path.display());
            false
        }
        Ok(_) => {
            log::trace!("{} already installed", dir_path.display());
            return true;
        }
    };
}

pub fn get_versions_dir() -> Result<PathBuf, String> {
    let mut dir = get_base_dir()?;
    dir.push("versions");
    return match fs::create_dir_all(&dir) {
        Err(e) => Err(format!(
            "versions directory does not exist and failed to create it\n{}\n{}",
            dir.display(),
            e
        )),
        Ok(_) => Ok(dir),
    };
}

pub fn get_base_dir() -> Result<PathBuf, String> {
    match env::current_exe() {
        Ok(dir) => Ok(dir.parent().unwrap().to_owned()),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn download_godot_version(
    version_name: &str,
    client: &reqwest::Client,
    url: &str,
) -> Result<PathBuf, String> {
    log::info!("Getting {url}");

    let version_zip_name = [version_name, "zip"].join(".");
    let versions_dir_path = get_versions_dir()?;

    let mut version_dir_path = versions_dir_path.clone();
    version_dir_path.push(&version_name);

    let mut version_zip_path = version_dir_path.clone();
    version_zip_path.push(&version_zip_name);

    if version_zip_path.is_file() {
        log::trace!(
            "Removing old zip file from previous installation attempt, {}",
            version_zip_path.display()
        );
        fs::remove_file(&version_zip_path).or(Err(format!(
            "Error removing old zip file from previous installation attempt\n{}",
            version_zip_path.display()
        )))?;
    }

    let url = Url::parse(url).or(Err("Invalid URL"))?;

    let file = download_file(client, url, &version_zip_path).await?;

    unzip_file(file, &version_dir_path).await?;

    log::trace!("Deleting zip file {}", &version_zip_path.display());
    if let Err(e) = remove_file(&version_zip_path) {
        return Err(e.to_string());
    }

    return Ok(version_dir_path);
}

/**
 * Checks if an exact version was specified, and if so, checks if it's already
 * installed on the system. If it's found to already be installed, that version
 * is set as the active version.
 *
 * Returns true if the specified version is now set to active
 * Returns false if no exact version specified or the version isn't currently installed
 * Returns an error if something goes wrong when trying to active the specified version
 */
pub fn activate_by_parts_if_installed(
    version: &Option<String>,
    platform: &Platform,
    architecture: &Architecture,
    flavour: &Flavour,
) -> Result<bool, String> {
    if let Some(v) = version {
        let version_name = gd::generate_version_name(v, platform, architecture, flavour)?;
        return activate_by_name_if_installed(&version_name);
    }
    return Ok(false);
}

pub fn activate_by_name_if_installed(name: &str) -> Result<bool, String> {
    if already_installed(name) {
        log::trace!("Version {name} already installed, setting active");
        match set_active_godot_version(name) {
            Err(e) => {
                return Err(format!(
                    "Error when trying to activate currently installed version {name}\n{e}"
                ))
            }
            Ok(_) => return Ok(true),
        }
    } else {
        log::trace!("Version {name} not yet installed");
    }
    return Ok(false);
}

pub fn get_installed_versions() -> Result<Vec<GodotVersionInfo>, String> {
    log::trace!("Checking installed versions");
    let versions_dir = get_versions_dir()?;
    log::trace!("Reading versions directory {}", versions_dir.display());

    let entries = match fs::read_dir(&versions_dir) {
        Err(e) => return Err(format!("Error reading versions directory\n{e}")),
        Ok(entries) => entries,
    };

    return Ok(entries
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("Godot_")
        })
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                Some(GodotVersionInfo {
                    name_parts: parse_version_name(&e.file_name().to_str().unwrap()).unwrap(),
                    path: e.path(),
                })
            })
        })
        .collect::<Vec<GodotVersionInfo>>());
}

#[derive(Clone)]
pub struct GodotVersionInfo {
    pub path: PathBuf,
    pub name_parts: GodotVersionNameParts,
}

pub fn get_current_version() -> Result<GodotVersionInfo, String> {
    let path = get_godot_link_path()?;

    if path.exists() {
        log::trace!("Found godot link path {}", path.display());
    } else {
        return Err(format!(
            "Cant determine current version, godot link not found at {}",
            path.display()
        )
        .to_owned());
    }

    let target = get_link_target(&path)?;

    let version_dir = get_version_dir_from_exe_path(&target)?;

    log::trace!("Found version directory {}", version_dir.display());

    let version_name = version_dir.file_name().unwrap();

    let current_info = GodotVersionInfo {
        path: target,
        name_parts: gd::parse_version_name(version_name.to_str().unwrap())?,
    };

    log::trace!(
        "Found active version of Godot to be {}, (platform = {}, architecture = {}, flavour = {})",
        current_info.name_parts.version.to_string(),
        current_info.name_parts.platform,
        current_info.name_parts.architecture,
        current_info.name_parts.flavour
    );

    return Ok(current_info);
}

#[cfg(windows)]
fn create_link(link_path: &PathBuf, target_path: &PathBuf) -> Result<(), String> {
    let sl = mslnk::ShellLink::new(target_path).or(Err("Failed to create ShellLink"))?;
    sl.create_lnk(link_path)
        .or(Err("Failed to create Godot shortcut"))?;

    return Ok(());
}

#[cfg(unix)]
fn create_link(link_path: &PathBuf, target_path: &PathBuf) -> Result<(), String> {
    if let Err(err) = symlink::symlink_file(target_path, link_path) {
        return Err(format!(
            "Failed to create Godot symlink,\n{}",
            err.to_string()
        ));
    }
    return Ok(());
}

#[cfg(windows)]
fn remove_link(link_path: &PathBuf) -> Result<(), String> {
    if link_path.is_file() {
        log::trace!("Removing old godot link {}", link_path.display());
        fs::remove_file(link_path).or(Err("Error deleting Windows Godot shortcut"))?;
    }
    return Ok(());
}

#[cfg(unix)]
fn remove_link(link_path: &PathBuf) -> Result<(), String> {
    match fs::symlink_metadata(&link_path) {
        Ok(_) => {
            log::trace!("Removing old godot link {}", link_path.display());
            if let Err(e) = fs::remove_file(&link_path) {
                return Err(e.to_string().to_owned());
            }
        }
        Err(_) => {}
    }
    return Ok(());
}

#[cfg(unix)]
fn get_link_target(link_path: &PathBuf) -> Result<PathBuf, String> {
    return match fs::read_link(link_path) {
        Err(e) => Err(format!(
            "Cant determine current version, godot link may be broken\n{e}"
        )),
        Ok(target) => {
            log::trace!("Godot link points to {}", target.display());
            return Ok(target);
        }
    };
}

#[cfg(windows)]
fn get_link_target(link_path: &PathBuf) -> Result<PathBuf, String> {
    let target = lnk::ShellLink::open(link_path).unwrap();
    log::trace!("Found lnk data {:#?}", target);

    let working_dir = target
        .working_dir()
        .as_ref()
        .expect("Godot shortcut has no working directory");

    let relative_path = target
        .relative_path()
        .as_ref()
        .expect("Godot shortcut has no relative path");

    let target_path = PathBuf::from_iter([working_dir, relative_path]);

    return Ok(target_path);
}

#[cfg(any(windows, target_os = "linux"))]
fn get_version_dir_from_exe_path(exe_path: &PathBuf) -> Result<PathBuf, String> {
    return Ok(exe_path.parent().unwrap().to_path_buf());
}

#[cfg(target_os = "macos")]
fn get_version_dir_from_exe_path(exe_path: &PathBuf) -> Result<PathBuf, String> {
    // exe_path will be something like Godot_v1.2.3/Godot/Contents/MacOS/Godot
    // We need to backtrack 4 parents to get the version dir
    return Ok(exe_path.ancestors().nth(4).unwrap().to_path_buf());
}

fn get_godot_link_path() -> Result<PathBuf, String> {
    let link_name = match env::consts::OS {
        "windows" => "godot.lnk",
        _ => "godot",
    };

    let base_dir = get_base_dir()?;

    let link_path: PathBuf = base_dir.join(link_name);

    return Ok(link_path);
}

/// On linux, the Godot executable is expected to be the only file within the version directory.
/// If exactly one file is found, it's path will be returned. Otherwise an error will be returned.
#[cfg(target_os = "linux")]
fn get_godot_exe_path(dir_path: &PathBuf) -> Result<PathBuf, String> {
    let files = get_files(dir_path)?;

    if files.len() == 1 {
        let exe_path = files.first().unwrap().path();
        if !exe_path.ends_with(".zip") {
            return Ok(exe_path);
        }
    }
    return Err("Cant find Godot executable".to_owned());
}

/// On Windows, the Godot executable is expected to match the directory name.
/// If this is found, it's path will be returned. Otherwise an error will be returned.
#[cfg(windows)]
fn get_godot_exe_path(dir_path: &PathBuf) -> Result<PathBuf, String> {
    let dir_name = dir_path.file_name().unwrap();
    for file in get_files(dir_path)? {
        if file.file_name() == dir_name || file.path().file_stem().unwrap() == dir_name {
            let exe_path = file.path();
            if !exe_path.ends_with(".zip") {
                return Ok(exe_path);
            }
        }
    }
    return Err("Cant find Godot executable".to_owned());
}

/// On MacOS, the executable is expected to be in a consistent, exact location.
/// If this is found, it will be returned, otherwise an error will be returned.
#[cfg(target_os = "macos")]
fn get_godot_exe_path(dir_path: &PathBuf) -> Result<PathBuf, String> {
    // there should be one folder within dir_path.
    // It name depends on the flavour - we dont care which it is, we just need to find it
    let entries = fs::read_dir(dir_path)
        .unwrap()
        .filter(|e| e.as_ref().unwrap().path().is_dir())
        .flatten()
        .collect::<Vec<DirEntry>>();

    let err = Err("Cant find Godot executable".to_owned());
    if entries.len() != 1 {
        return err;
    }

    let path = dir_path
        .join(entries[0].file_name())
        .join("Contents/MacOS/Godot");

    return match path.is_file() {
        true => Ok(path),
        false => err,
    };
}

fn get_files(dir_path: &PathBuf) -> Result<Vec<DirEntry>, String> {
    return Ok(match fs::read_dir(dir_path) {
        Err(err) => return Err(err.to_string().to_owned()),
        Ok(e) => e,
    }
    .flatten()
    .filter(|e| e.metadata().unwrap().is_file())
    .collect::<Vec<DirEntry>>());
}

async fn download_file(
    client: &reqwest::Client,
    url: Url,
    out_file_path: &PathBuf,
) -> Result<File, String> {
    let download_size = {
        let resp = match client.head(url.as_str()).send().await {
            Err(e) => return Err(e.to_string()),
            Ok(d) => d,
        };
        if resp.status().is_success() {
            resp.headers() // Gives us the HeaderMap
                .get(header::CONTENT_LENGTH) // Gives us an Option containing the HeaderValue
                .and_then(|ct_len| ct_len.to_str().ok()) // Unwraps the Option as &str
                .and_then(|ct_len| ct_len.parse().ok()) // Parses the Option as u64
                .unwrap_or(0) // Fallback to 0
        } else {
            // We return an Error if something goes wrong here
            return Err(
                format!("Couldn't download URL: {}. Error: {:?}", url, resp.status(),).into(),
            );
        }
    };

    let request = client.get(url);

    let progress_bar = ProgressBar::new(download_size);

    // Set Style to the ProgressBar
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {bytes}/{total_bytes} - {msg}")
            .or(Err("Invalid progress bar".to_string()))?
            .progress_chars("#>-"),
    );

    progress_bar.set_message("Download progress");

    let parent_path = out_file_path.parent().unwrap();

    log::trace!("Creating {}", &parent_path.display());
    if let Err(e) = tokio::fs::create_dir_all(&parent_path).await {
        return Err(format!(
            "Failed to create directory {}\n{}",
            parent_path.display(),
            e,
        ));
    }

    log::trace!("Creating {}", &out_file_path.display());

    let mut file = match tokio::fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(&out_file_path)
        .await
    {
        Err(e) => {
            return Err(format!(
                "Failed to create file '{}'\n{e}",
                out_file_path.display()
            ))
        }
        Ok(f) => f,
    };

    log::trace!("Downloading zip file contents");
    // Do the actual request to download the file
    let mut download = match request.send().await {
        Err(e) => return Err(e.to_string()),
        Ok(d) => d,
    };

    while let Some(chunk) = download.chunk().await.or(Err("Error downloading chunk"))? {
        progress_bar.inc(chunk.len() as u64); // Increase ProgressBar by chunk size
        file.write(&chunk)
            .await
            .or(Err("Error writing chunk to file"))?; // Write chunk to output file
    }

    progress_bar.finish();

    file.flush().await.or(Err("Error flushing file"))?;

    return Ok(file);
}

async fn unzip_file(file: tokio::fs::File, out_dir: &Path) -> Result<(), String> {
    log::trace!("Creating zip reader");
    let archive = BufReader::new(file).compat();
    let mut reader = ZipFileReader::new(archive)
        .await
        .or(Err("Error creating zip reader"))?;

    let out_dir_name = out_dir.file_name().unwrap().to_str().unwrap().to_owned() + "/";
    let entry_count = reader.file().entries().len();
    let progress_bar = ProgressBar::new(entry_count as u64);

    log::trace!("Creating zip progress bar");
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len} - {msg}")
            .or(Err("Error creating progress bar for zip progress"))?
            .progress_chars("#>-"),
    );

    progress_bar.set_message("Extracting contents from zip");

    for index in 0..entry_count {
        log::trace!("Extracting entry {index} of {entry_count}");
        let entry = reader.file().entries().get(index).unwrap();

        // reduce nesting where the zip contains a folder matching
        // the parent directory name. Remove the current directory name
        // from the start of the entry path if it exists.
        let mut path = out_dir.to_path_buf();
        match entry
            .filename()
            .as_str()
            .unwrap()
            .trim_start_matches(out_dir_name.as_str())
        {
            "" => (),
            v => path.push(v),
        };

        let entry_is_dir = entry.dir().unwrap();

        log::trace!("Creating reader for zip entry {index}");
        let mut entry_reader = reader
            .reader_without_entry(index)
            .await
            .or(Err("Failed to read ZipEntry"))?;

        if entry_is_dir {
            if !path.exists() {
                log::trace!(
                    "Creating directory {} for zip entry {index}",
                    path.display()
                );
                create_dir_all(&path)
                    .await
                    .or(Err("Failed to create extracted directory"))?;
            }
        } else {
            let parent = path
                .parent()
                .expect("A file entry should have parent directories");

            if !parent.is_dir() {
                log::trace!(
                    "Creating directory {} for zip entry {index}",
                    parent.display()
                );
                create_dir_all(parent)
                    .await
                    .or(Err("Failed to create parent directories"))?;
            }

            log::trace!("Creating writer for zip entry {index}");
            let mut file_opts = OpenOptions::new();
            file_opts.write(true).create_new(true);

            #[cfg(unix)]
            file_opts.mode(755);

            let writer = file_opts
                .open(&path)
                .await
                .expect("Failed to create extracted file");

            log::trace!("Extracting zip entry {index} to {}", path.display());

            futures_lite::io::copy(&mut entry_reader, &mut writer.compat_write())
                .await
                .or(Err("Failed to copy to extracted file"))?;

            progress_bar.inc(1);
        }
    }

    progress_bar.finish();

    return Ok(());
}
