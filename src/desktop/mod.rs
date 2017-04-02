use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ffi::OsStr;
use std::str::FromStr;
use std::result::Result as StdResult;
use std::borrow::Borrow;
use std::path::PathBuf;

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
    pub fn new(desktop_files: Vec<PathBuf>) -> Self {
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
}

pub fn find_all_desktop_files() -> Result<Vec<PathBuf>> {
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
    Ok(desktop_files)
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

enum EntryKey {
    Type(String),
    Version(String),
    Name(LocaleString),
    GenericName(LocaleString),
    NoDisplay(bool),
    Comment(LocaleString),
    Hidden(bool),
    OnlyShowIn(Vec<String>),
    NotShowIn(Vec<String>),
    TryExec(String),
    Exec(String),
    Path(String),
    Terminal(bool),
    Categories(Vec<String>),
    Keywords(Vec<String>),
}

impl FromStr for EntryKey {
    type Err = Error;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        // TODO parse all types
        if let Some((name, value)) = Self::parse_entry(s.trim()) {
        } else {
            Err(ErrorKind::UnknownEntryKey.into())
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
        split_entry(line)
            .map(|(key, value)| {
                let i = key.find("[").unwrap_or_else(|| key.len());
                match key[0..i].trim().as_ref() {
                    "Type" => typ = Some(value.to_string()),
                    "Name" => name = Some(LocaleString::new(key, value)),
                    //                "GenericName" => Ok(EntryKey::GenericName(LocaleString::new(name, value))),
                    //                "NoDisplay" => Ok(EntryKey::NoDisplay(value.parse::<bool>()?)),
                    _ => Err(ErrorKind::UnknownEntryKey.into())
                }
            })
    }
    for entry in entries.iter() {
        match entry {
            &EntryKey::Name(ref local_name) => {
                name = Some(local_name.value[..].to_string());
                ()
            }
            _ => {
                ()
            }
        }
    }
    //    desktop_entry
    //    desktop_entry.name
    Ok(DesktopEntry {
        name: name.ok_or(ErrorKind::MissingRequiredEntryKey)?
    })
}
