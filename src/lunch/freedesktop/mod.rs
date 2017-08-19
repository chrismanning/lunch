use std::fs::File;
use std::path::PathBuf;
use std::io::BufReader;

use xdg::BaseDirectories as XdgDirs;

pub mod locale;
mod parse;
pub mod entry;

use lunch::errors::*;

use self::locale::Locale;
use self::entry::*;

pub struct DesktopFiles {
    desktop_files: Vec<PathBuf>,
}

impl DesktopFiles {
    fn new(desktop_files: Vec<PathBuf>) -> Self {
        DesktopFiles { desktop_files: desktop_files }
    }

    pub fn find_exact_match(&self, name: &str, locale: &Locale) -> Result<DesktopEntry> {
        self.parse_files(locale)
            .into_iter()
            .filter(|entry| entry.entry_type == "Application")
            .filter(|entry| !entry.no_display)
            .skip_while(|entry| entry.name != name)
            .find(|entry| !entry.hidden)
            .ok_or_else(|| ErrorKind::NoMatchFound(name.to_owned()).into())
    }

    pub fn parse_files(&self, locale: &Locale) -> Vec<DesktopEntry> {
        self.desktop_files
            .iter()
            .map(|buf| File::open(buf.as_path()))
            .filter_map(|file| match file {
                Ok(e) => {
                    debug!("Opened file {:?}", e);
                    Some(e)
                }
                Err(err) => {
                    warn!("Error opening file: {}", err);
                    None
                }
            })
            .map(|file| {
                DesktopEntry::read_desktop_entry(BufReader::new(file), locale)
            })
            .filter_map(|entry| match entry {
                Ok(e) => {
                    debug!("Found desktop entry file {:?}", e);
                    Some(e)
                }
                Err(err) => {
                    warn!("Error reading desktop file: {}", err);
                    None
                }
            })
            .collect()
    }
}

pub fn find_all_desktop_files() -> Result<DesktopFiles> {
    let xdg = XdgDirs::new()?;
    let data_files = xdg.list_data_files_once("applications");
    let desktop_files = data_files
        .into_iter()
        .filter(|path| match path.extension() {
            Some(os_str) => os_str.to_str() == Some("desktop"),
            None => false,
        })
        .map(|path| {
            debug!("Found desktop file '{}'", path.as_path().display());
            path
        })
        .collect();
    Ok(DesktopFiles::new(desktop_files))
}
