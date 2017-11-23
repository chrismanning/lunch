use std::path::{Path, PathBuf};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::convert::TryFrom;

use super::desktopfile::DesktopFile;
use super::entry::*;
use lunch::errors::*;
use lunch::exec::{Exec, FieldCode};
use lunch::{Io, Launch, Options, Search};
use lunch::search::SearchTerms;

#[derive(Debug)]
pub struct Application {
    pub name: String,
    pub icon: Option<String>,
    pub comment: Option<String>,
    pub keywords: Vec<String>,
    pub exec: Exec,
    pub field_code: Option<FieldCode>,
    pub try_exec: Option<PathBuf>,
    pub path: Option<PathBuf>,
    pub actions: Vec<Action>,
}

#[derive(Debug)]
pub struct Action {
    name: String,
    icon: Option<String>,
    exec: Exec,
}

impl Application {
    fn can_exec(&self) -> bool {
        if let Some(ref try_exec) = self.try_exec {
            try_exec.exists()
        } else {
            true
        }
    }
}

impl Display for Application {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.name)
    }
}

impl Launch for Application {
    fn launch(&self, args: Vec<String>) -> Error {
        if self.can_exec() {
            info!("Launching '{}'...", self);
            if let Some(ref path) = self.try_exec {
                let path = Path::new(path);
                if !path.exists() {
                    return ErrorKind::ApplicationNotFound.into();
                }
            }

            let children = if let Some(field_code) = self.field_code {
                let cmd_lines = field_code.expand_exec(&self.exec, args);
                cmd_lines
                    .into_iter()
                    .map(|cmd_line| {
                        self.spawn(
                            cmd_line,
                            self.path.as_ref().map(|path| path.as_path()),
                            &Options { io: Io::Suppress },
                        )
                    })
                    .collect()
            } else {
                let cmd_line = self.exec.get_command_line(vec![]);
                let opt = Options { io: Io::Inherit };
                self.spawn(
                    cmd_line,
                    self.path.as_ref().map(|path| path.as_path()),
                    &opt,
                ).map(|child| vec![child])
            };
            match children {
                Ok(_) => {
                    ::std::process::exit(0);
                }
                Err(err) => err.into(),
            }
        } else {
            ErrorKind::ApplicationNotFound.into()
        }
    }
}

impl Search for Application {
    fn search_terms(&self) -> SearchTerms {
        let mut terms = vec![self.name.clone()];
        if let Some(ref comment) = self.comment {
            terms.push(comment.clone())
        }
        SearchTerms {
            terms,
            keywords: self.keywords.clone(),
        }
    }
}

impl TryFrom<DesktopAction> for Action {
    type Error = Error;

    fn try_from(desktop_action: DesktopAction) -> Result<Action> {
        Ok(Action {
            name: desktop_action.name,
            exec: desktop_action.exec.parse()?,
            icon: desktop_action.icon,
        })
    }
}

impl TryFrom<DesktopFile> for Application {
    type Error = Error;

    fn try_from(desktop_file: DesktopFile) -> Result<Application> {
        let exec = desktop_file.desktop_entry.exec;
        Ok(Application {
            name: desktop_file.desktop_entry.name,
            icon: desktop_file.desktop_entry.icon,
            comment: desktop_file.desktop_entry.comment,
            keywords: desktop_file.desktop_entry.keywords,
            exec: exec.clone()
                .ok_or(ErrorKind::InvalidCommandLine("".into()).into())
                .and_then(|s| s.parse())?,
            field_code: exec.and_then(|exec| FieldCode::extract_field_code(&exec)),
            try_exec: desktop_file.desktop_entry.try_exec.map(From::from),
            path: desktop_file.desktop_entry.path.map(From::from),
            actions: desktop_file.actions
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<_>>()?,
        })
    }
}
