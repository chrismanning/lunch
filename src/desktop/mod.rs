use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ffi::OsStr;
use std::str::FromStr;
use std::result::Result as StdResult;
use std::borrow::Borrow;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::os::unix::process::CommandExt;

use walkdir::{DirEntry, WalkDir, WalkDirIterator};
use xdg::BaseDirectories as XdgDirs;

pub mod errors;
pub mod locale;

use locale::{Locale, MatchLevel};
use errors::*;

pub struct Applications {
    desktop_files: Vec<PathBuf>,
}

impl Applications {
    fn new(desktop_files: Vec<PathBuf>) -> Self {
        Applications {
            desktop_files: desktop_files
        }
    }

    pub fn find_exact_match(&self, name: &str, locale: Option<Locale>) -> Result<DesktopEntry> {
        self.desktop_files.iter()
            .map(|buf| File::open(buf.as_path()))
            .filter_map(|entry| {
                match entry {
                    Ok(e) => {
                        debug!("Opened file {:?}", e);
                        Some(e)
                    }
                    Err(err) => {
                        warn!("Error opening file: {}", err);
                        None
                    }
                }
            })
            .map(|file| read_desktop_entry(BufReader::new(file)))
            // TODO don't just open the first
            .next().ok_or(ErrorKind::NoMatchFound.into())
            .and_then(|x| x)

    }
}

#[derive(Debug, Default, Builder)]
pub struct DesktopEntry {
    pub application: String,
    pub name: String,
    pub generic_name: Option<String>,
    pub no_display: bool,
    pub comment: String,
    pub icon: PathBuf,
    pub hidden: bool,
    pub only_show_in: Option<Vec<String>>,
    pub not_show_in: Option<Vec<String>>,
    pub try_exec: Option<String>,
    pub exec: Option<String>,
    pub path: Option<PathBuf>,
    pub terminal: bool,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
}

impl DesktopEntry {
    pub fn launch(&self) -> Result<()> {
        let installed = match self.try_exec {
            Some(ref path) => {
                let path = Path::new(path);
                path.exists()
            },
            None => false
        };
        // TODO launch()
        Ok(())
    }
}

pub fn find_all_desktop_files() -> Result<Applications> {
    let xdg = XdgDirs::new()?;
    let data_files = xdg.list_data_files_once("applications");
    let desktop_files = data_files.into_iter()
        .filter(|path| match path.extension() {
            Some(os_str) => os_str.to_str() == Some("desktop"),
            None => false
        })
        .map(|path| {
            debug!("Found desktop file '{}'", path.as_path().display());
            path
        })
        .collect();
    Ok(Applications::new(desktop_files))
}

struct LocaleString {
    value: String,
    locale: Option<Locale>,
}

impl LocaleString {
    pub fn new(name: &str, value: &str) -> LocaleString {
        let name = name.find("[")
            .map(|i| &name[i + 1..name.len()])
            .and_then(|locale| locale.rfind("]")
                .map(|j| &locale[0..j]));
        LocaleString {
            value: value.to_string(),
            locale: name.and_then(|s| s.parse::<Locale>().ok()),
        }
    }
}

fn split_entry(entry: &str) -> Option<(&str, &str)> {
    entry.find("=")
        .map(|i| entry.split_at(i))
        .map(|(name, value)| (name.trim(), value[1..value.len()].trim()))
}

fn read_desktop_entry<R: BufRead>(input: R) -> Result<DesktopEntry> {
    let lines = input.lines()
        .filter_map(|line| line.ok())
        .filter(|line| line.trim().is_empty() || line.trim().starts_with("#"))
        .skip_while(|line| !line.trim().starts_with("[Desktop Entry]"))
        .skip(1)
        .take_while(|line| !line.trim().starts_with("["));
    // TODO make Vec<(Locale, DesktopEntry)>, get current locale, return closest match

    // TODO parse directly into Option bindings like below
    let mut typ = None;
    let mut name = None;
    for line in lines {
        split_entry(&line)
            .map(|(key, value)| {
                let i = key.find("[").unwrap_or_else(|| key.len());
                match key[0..i].trim().as_ref() {
                    "Type" => typ = Some(value.to_string()),
                    "Name" => name = Some(LocaleString::new(key, value)),
                    //                "GenericName" => Ok(EntryKey::GenericName(LocaleString::new(name, value))),
                    //                "NoDisplay" => Ok(EntryKey::NoDisplay(value.parse::<bool>()?)),
                    _ => {}// Err(ErrorKind::UnknownEntryKey.into())
                }
            });
        ()
    }

    if typ.as_ref().map(|s| s.as_str()) != Some("Application") {
        return Err(ErrorKind::TypeNotApplication.into());
    }

//    Ok(DesktopEntry {
//        name: name.ok_or(ErrorKind::MissingRequiredEntryKey)?,
//    })
    Err(ErrorKind::NoMatchFound.into())
}
