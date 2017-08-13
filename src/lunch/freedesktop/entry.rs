use std::path::{Path, PathBuf};
use std::io::{BufRead, BufReader};
use std::process::Command;
use std::os::unix::process::CommandExt;
use std::ffi::OsStr;

use lunch::errors::*;
use lunch::Exec;

use super::locale::Locale;
use super::parse::parse_group;

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
    pub fn read_desktop_entry<R: BufRead>(input: R, locale: &Locale) -> Result<DesktopEntry> {
        let group = parse_group(
            input.lines().map(
                |res| res.chain_err(|| "Error reading file"),
            ),
            "Desktop Entry",
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
}

impl Exec for DesktopEntry {
    fn exec<I, S>(&self, args: I) -> Error where
        I: IntoIterator<Item=S>,
        S: AsRef<OsStr> {
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
        cmd.args(args);

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

struct ApplicationEntry {

}

//impl Ex

struct ActionEntry {

}
