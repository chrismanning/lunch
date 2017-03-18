use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ffi::OsStr;
use std::str::FromStr;
use std::result::Result as StdResult;
use std::borrow::Borrow;

use walkdir::{DirEntry, WalkDir, WalkDirIterator};

use locale::{Locale, MatchLevel};

pub mod errors;
pub mod locale;

use errors::*;

pub struct DesktopEntry {
    pub name: String,
}

pub fn find_exact_match(name: &str, locale: Option<Locale>) -> Option<DesktopEntry> {
    // TODO make Result so I can use ?
    WalkDir::new("/home/chris/.local/share/applications").into_iter().chain(WalkDir::new("/usr/share/applications"))
        //        .filter_map(|entry| entry.ok())
        .filter_map(|dir_entry| {
            match dir_entry {
                Ok(entry) => {
                    trace!("found entry {:?}", entry);
                    Some(entry)
                }
                Err(err) => {
                    warn!("Error opening file: {}", err);
                    None
                }
            }
        })
        .filter(|entry| match entry.path().extension() {
            Some(os_str) => os_str.to_str() == Some("desktop"),
            None => false
        })
        .map(|entry| File::open(entry.path()))
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
        .map(|file| parse(BufReader::new(file)))
        // TODO don't just open the first
        .next()
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
    TryExec(String),
    Exec(String),
    Path(String),
    Terminal(bool),
    Categories(Vec<String>),
    Keywords(Vec<String>),
}

impl EntryKey {
    fn parse_entry(entry: &str) -> Option<(&str, &str)> {
        entry.find("=")
            .map(|i| entry.split_at(i))
            .map(|(name, value)| (name.trim(), value[1..value.len()].trim()))
    }
}

impl FromStr for EntryKey {
    type Err = Error;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        // TODO parse all types
        if let Some((name, value)) = Self::parse_entry(s) {
            let i = name.find("[").unwrap_or_else(|| name.len());
            match name[0..i].trim().to_lowercase().as_ref() {
                "type" => Ok(EntryKey::Type(value.to_string())),
                "name" => Ok(EntryKey::Name(LocaleString::new(name, value))),
                "genericname" => Ok(EntryKey::GenericName(LocaleString::new(name, value))),
                "nodisplay" => Ok(EntryKey::NoDisplay(value.parse::<bool>()?)),
                _ => Err(ErrorKind::UnknownEntryKey.into())
            }
        } else {
            Err(ErrorKind::UnknownEntryKey.into())
        }
    }
}

fn parse<R: BufRead>(input: R) -> DesktopEntry {
    let mut entries: Vec<EntryKey> = input.lines()
        .filter_map(|line| line.ok())
        .skip_while(|line| !line.starts_with("[Desktop Entry]"))
        .skip(1)
        .take_while(|line| !line.starts_with("["))
        .map(|line| line.parse::<EntryKey>())
        .filter_map(|line| line.ok())
        .collect();
    let mut desktop_entry: DesktopEntry;
    for entry in entries.iter() {
        match entry {
            &EntryKey::Name(ref local_name) => {
                desktop_entry.name = local_name.value[..].to_string();
                ()
            }
            _ => {
                ()
            }
        }
    }
    //    desktop_entry
    //    desktop_entry.name
    DesktopEntry {
        name: entries.iter()
            .filter_map(|entry| if let &EntryKey::Name(ref name) = entry { Some(name.value[..].to_string()) } else { None })
            .next().unwrap()
    }
}
