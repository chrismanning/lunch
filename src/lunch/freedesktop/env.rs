use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::BufReader;
use std::borrow::Cow;
use std::convert::TryFrom;

use xdg::BaseDirectories as XdgDirs;

use lunch::errors::*;
use lunch::env::LunchEnv;

use super::locale::Locale;
use super::desktopfile::DesktopFile;
use super::application::Application;

pub struct FreeDesktopEnv;

impl FreeDesktopEnv {
    pub fn init_lunch<'a>() -> Result<LunchEnv> {
        let desktop_files = find_all_desktop_files()?;
        let locale = Locale::from_env()?;
        let mut desktop_files = parse_files(desktop_files.into_iter(), &locale);
        desktop_files.sort_by_key(|desktop_file| desktop_file.desktop_entry.name.clone());
        desktop_files.dedup_by_key(|desktop_file| desktop_file.desktop_entry.name.clone());

        let current_desktop = current_desktop()?;
        let desktop_files: Vec<_> = desktop_files
            .into_iter()
            .filter(|desktop_file| !desktop_file.desktop_entry.no_display)
            .filter(|desktop_file| !desktop_file.desktop_entry.hidden)
            .filter(|desktop_file| {
                desktop_file.desktop_entry.only_show_in.is_empty() ||
                    desktop_file.desktop_entry.only_show_in.iter().any(
                        |desktop| {
                            desktop == &current_desktop
                        },
                    )
            })
            .filter(|desktop_file| {
                desktop_file.desktop_entry.not_show_in.iter().all(
                    |desktop| {
                        desktop != &current_desktop
                    },
                )
            })
            .collect();
        let applications: Vec<Application> = desktop_files
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<_>>()?;
        let lunchables = applications
            .into_iter()
            .flat_map(|application| application.to_lunchables().into_iter())
            .collect();
        Ok(LunchEnv {
            lunchables
        })
    }
}

pub fn current_desktop<'a>() -> Result<Cow<'a, str>> {
    let xdg_current_desktop = ::std::env::var("XDG_CURRENT_DESKTOP");
    xdg_current_desktop.map(Cow::Owned).chain_err(|| {
        ErrorKind::NotDesktopEnvironment
    })
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
            trace!("Found desktop file '{}'", path.as_path().display());
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
