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

pub struct DesktopFiles {
    desktop_files: Vec<PathBuf>,
}

impl DesktopFiles {
    fn new(desktop_files: Vec<PathBuf>) -> Self {
        DesktopFiles {
            desktop_files: desktop_files
        }
    }

    pub fn find_exact_match(&self, name: &str, locale: Option<Locale>) -> Result<DesktopEntry> {
        self.parse_files().into_iter()
            .filter(|entry| entry.entry_type == "Application")
            .filter(|entry| !entry.no_display)
            .skip_while(|entry| entry.name != name)
            .filter(|entry| !entry.hidden)
            .next().ok_or(ErrorKind::NoMatchFound.into())
    }

    pub fn parse_files(&self) -> Vec<DesktopEntry> {
        self.desktop_files.iter()
            .map(|buf| File::open(buf.as_path()))
            .filter_map(|file| {
                match file {
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
            .filter_map(|entry| {
                match entry {
                    Ok(e) => {
                        debug!("Found desktop entry file {:?}", e);
                        Some(e)
                    }
                    Err(err) => {
                        warn!("Error reading desktop file: {}", err);
                        None
                    }
                }
            })
            .collect()
    }
}

#[derive(Debug, Default, Builder)]
pub struct DesktopEntry {
    #[builder(setter(into))]
    pub entry_type: String,
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into), default="None")]
    pub generic_name: Option<String>,
    #[builder(default="false")]
    pub no_display: bool,
    #[builder(setter(into), default="None")]
    pub comment: Option<String>,
    #[builder(setter(into), default="None")]
    pub icon: Option<PathBuf>,
    #[builder(default="false")]
    pub hidden: bool,
    #[builder(default="vec![]")]
    pub only_show_in: Vec<String>,
    #[builder(default="vec![]")]
    pub not_show_in: Vec<String>,
    #[builder(setter(into), default="None")]
    pub try_exec: Option<String>,
    #[builder(setter(into), default="None")]
    pub exec: Option<String>,
    #[builder(setter(into), default="None")]
    pub path: Option<PathBuf>,
    #[builder(default="vec![]")]
    pub keywords: Vec<String>,
    #[builder(default="vec![]")]
    pub categories: Vec<String>,
}

impl DesktopEntry {
    pub fn launch(&self) -> Result<()> {
        info!("Launching '{}'", self.name);
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

pub fn find_all_desktop_files() -> Result<DesktopFiles> {
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
    Ok(DesktopFiles::new(desktop_files))
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
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("#"))
        .skip_while(|line| !line.trim().starts_with(r"[Desktop Entry]"))
        .skip(1)
        .take_while(|line| !line.trim().starts_with("["))
    ;
    // TODO make Vec<(Locale, DesktopEntry)>, get current locale, return closest match

    let mut builder = DesktopEntryBuilder::default();
    for line in lines {
        match split_entry(&line) {
            Some((key, value)) => {
                let i = key.find("[").unwrap_or_else(|| key.len());
                match key[0..i].trim().as_ref() {
                    "Type" => builder.entry_type(value),
                    "Name" => builder.name(value),
                    "GenericName" => builder.generic_name(value.to_string()),
                    "NoDisplay" => builder.no_display(value.parse()?),
                    "Comment" => builder.comment(value.to_string()),
                    _ => &builder
                };
                ()
            },
            None => ()
        }
    }

    builder.build()
        .map_err(|s| s.into())
}
