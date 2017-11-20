use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::BufReader;

use xdg::BaseDirectories as XdgDirs;

use lunch::errors::*;
use lunch::env::LunchEnv;

use super::locale::Locale;
use super::desktopfile::DesktopFile;

pub struct FreeDesktopEnv;

impl FreeDesktopEnv {
    pub fn init_lunch() -> Result<LunchEnv> {
        let desktop_files = find_all_desktop_files()?;
        let locale = Locale::from_env()?;
        let desktop_files = parse_files(desktop_files.into_iter(), &locale);
        // TODO filter on ShowIn, Hidden, etc.
        // TODO convert DesktopFile to Application (Lunchable)
        unimplemented!()
    }
}

fn find_all_desktop_files() -> Result<Vec<PathBuf>> {
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
    Ok(desktop_files)
}

fn parse_files<Iter, T>(desktop_files: Iter, locale: &Locale) -> Vec<DesktopFile>
where
    Iter: Iterator<Item = T>,
    T: AsRef<Path>,
{
    desktop_files
        .map(|buf| File::open(buf.as_ref()))
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
        .map(|file| DesktopFile::read(BufReader::new(file), locale))
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
