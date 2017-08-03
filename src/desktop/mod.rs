use std::fs::File;
use std::result::Result as StdResult;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::os::unix::process::CommandExt;
use std::io::{BufRead, BufReader};

use xdg::BaseDirectories as XdgDirs;

pub mod errors;
pub mod locale;
mod parse;

use locale::Locale;
use self::parse::parse_desktop_entry_group;
use errors::*;

pub struct DesktopFiles {
    desktop_files: Vec<PathBuf>,
}

impl DesktopFiles {
    fn new(desktop_files: Vec<PathBuf>) -> Self {
        DesktopFiles { desktop_files: desktop_files }
    }

    pub fn find_exact_match(&self, name: &str) -> Result<DesktopEntry> {
        self.parse_files()
            .into_iter()
            .filter(|entry| entry.entry_type == "Application")
            .filter(|entry| !entry.no_display)
            .skip_while(|entry| entry.name != name)
            .find(|entry| !entry.hidden)
            .ok_or_else(|| ErrorKind::NoMatchFound.into())
    }

    pub fn parse_files(&self) -> Vec<DesktopEntry> {
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
                read_desktop_entry(
                    BufReader::new(file),
                    &get_locale_from_env().unwrap_or_default(),
                )
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

fn get_locale_from_env() -> Option<Locale> {
    unimplemented!();
}

#[derive(Debug, Default, Builder)]
pub struct DesktopEntry {
    #[builder(setter(into))]
    pub entry_type: String,
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into), default = "None")]
    pub generic_name: Option<String>,
    #[builder(default = "false")]
    pub no_display: bool,
    #[builder(setter(into), default = "None")]
    pub comment: Option<String>,
    #[builder(setter(into), default = "None")]
    pub icon: Option<PathBuf>,
    #[builder(default = "false")]
    pub hidden: bool,
    #[builder(default = "vec![]")]
    pub only_show_in: Vec<String>,
    #[builder(default = "vec![]")]
    pub not_show_in: Vec<String>,
    #[builder(setter(into), default = "None")]
    pub try_exec: Option<String>,
    #[builder(setter(into), default = "None")]
    pub exec: Option<String>,
    #[builder(setter(into), default = "None")]
    pub path: Option<PathBuf>,
    #[builder(default = "vec![]")]
    pub keywords: Vec<String>,
    #[builder(default = "vec![]")]
    pub categories: Vec<String>,
}

impl DesktopEntry {
    pub fn launch(&self) -> Error {
        info!("Launching '{}'...", self.name);
        if let Some(ref path) = self.try_exec {
            let path = Path::new(path);
            if !path.exists() {
                return ErrorKind::ApplicationNotFound.into();
            }
        }
        let mut cmd = if let Some(ref exec) = self.exec {
            Command::new(exec)
        } else {
            return ErrorKind::MissingRequiredEntryKey.into();
        };
        // TODO launch() args

        if let Some(ref path) = self.path {
            cmd.current_dir(path);
        }
        use std::io::ErrorKind::NotFound;
        match cmd.exec().kind() {
            NotFound => ErrorKind::ApplicationNotFound.into(),
            _ => ErrorKind::UnknownError.into(),
        }
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

fn read_desktop_entry<R: BufRead>(input: R, locale: &Locale) -> Result<DesktopEntry> {
    let group = parse_desktop_entry_group(
        input.lines().map(
            |res| res.chain_err(|| "Error reading file"),
        ),
        locale,
    )?;

    let mut builder = DesktopEntryBuilder::default();
    for (key, value) in group {
        match key.as_ref() {
            "Type" => builder.entry_type(value),
            "Name" => builder.name(value),
            "GenericName" => builder.generic_name(value.to_string()),
            "NoDisplay" => builder.no_display(value.parse()?),
            "Comment" => builder.comment(value.to_string()),
            _ => &builder,
        };
    }

    builder.build().map_err(|s| s.into())
}
