use std::path::{Path, PathBuf};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::rc::Rc;
use std::convert::TryFrom;

use super::desktopfile::DesktopFile;
use super::entry::*;
use lunch::errors::*;
use lunch::exec::{Exec, FieldCode};
use lunch::{Io, Launch, Lunchable, Options, Search};
use lunch::search::SearchTerms;

#[derive(Debug)]
pub struct Application {
    pub app_part: Rc<ApplicationPart>,
    pub action_parts: Vec<Rc<ActionPart>>,
}

impl TryFrom<DesktopFile> for Application {
    type Error = Error;

    fn try_from(desktop_file: DesktopFile) -> Result<Self> {
        debug!(
            "Processing desktop entry '{}'",
            desktop_file.desktop_entry.name
        );
        let exec = desktop_file.desktop_entry.exec.unwrap_or("".to_owned());
        if exec.trim().is_empty() {
            return Err(ErrorKind::InvalidCommandLine(exec).into());
        }

        let app_data = Rc::new(ApplicationPart {
            name: desktop_file.desktop_entry.name,
            icon: desktop_file.desktop_entry.icon,
            comment: desktop_file.desktop_entry.comment,
            keywords: desktop_file.desktop_entry.keywords,
            field_code: FieldCode::extract_field_code(&exec),
            exec: exec.parse()?,
            try_exec: desktop_file.desktop_entry.try_exec.map(From::from),
            path: desktop_file.desktop_entry.path.map(From::from),
        });
        let actions = desktop_file
            .actions
            .into_iter()
            .map(|desktop_action| ActionPart::from_desktop_action(desktop_action, app_data.clone()))
            .collect::<Result<_>>()?;
        Ok(Application { app_part: app_data, action_parts: actions })
    }
}

impl Application {
    pub fn can_exec(&self) -> bool {
        self.app_part.can_exec()
    }

    pub fn to_lunchables(self) -> Vec<Rc<Lunchable>> {
        let (app, actions) = (self.app_part, self.action_parts);
        let mut actions = actions.clone()
            .into_iter()
            .map(|action| action as Rc<Lunchable>)
            .collect();
        let mut lunchables: Vec<Rc<Lunchable>> = vec![app];
        lunchables.append(&mut actions);
        lunchables
    }
}

#[derive(Debug)]
pub struct ApplicationPart {
    pub name: String,
    pub icon: Option<String>,
    pub comment: Option<String>,
    pub keywords: Vec<String>,
    pub exec: Exec,
    pub field_code: Option<FieldCode>,
    pub try_exec: Option<PathBuf>,
    pub path: Option<PathBuf>,
}

impl ApplicationPart {
    pub fn can_exec(&self) -> bool {
        self.try_exec
            .as_ref()
            .map(|try_exec| can_exec(try_exec.as_path()))
            .unwrap_or_else(|| true)
    }
}

fn can_exec(try_exec: &Path) -> bool {
    if try_exec.is_absolute() {
        try_exec.exists()
    } else if let Some(paths) = ::std::env::var_os("PATH") {
        for path in ::std::env::split_paths(&paths) {
            debug!("Looking for {} in {}", try_exec.display(), path.display());
            if path.is_absolute() && path.exists() {
                let path = path.join(try_exec);
                if path.exists() {
                    return true;
                }
            }
        }
        false
    } else {
        false
    }
}

#[cfg(test)]
mod can_exec_tests {
    use super::*;

    #[test]
    fn test_relative() {
        assert!(can_exec(Path::new("echo")));
    }

    #[test]
    fn test_nonexistant() {
        assert!(!can_exec(Path::new("mdi309r29rj298f93d")));
    }

    #[test]
    fn test_absolute() {
        use tempdir::TempDir;

        let tmp_dir = TempDir::new("can_exec").unwrap();
        let path = tmp_dir.path().join("echo");

        assert!(!can_exec(&path));
    }
}

impl Display for ApplicationPart {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.name)
    }
}

impl Launch for ApplicationPart {
    fn launch(&self, args: Vec<String>) -> Error {
        if !self.can_exec() {
            return ErrorKind::ApplicationNotFound.into();
        }
        info!("Launching '{}'...", self);
        if let Some(ref path) = self.try_exec {
            let path = Path::new(path);
            if !path.exists() {
                return ErrorKind::ApplicationNotFound.into();
            }
        }

        if let Some(field_code) = self.field_code {
            let cmd_lines = field_code.expand_exec(&self.exec, args);
            let children = cmd_lines
                .into_iter()
                .map(|cmd_line| {
                    self.spawn(
                        cmd_line,
                        self.path.as_ref().map(|path| path.as_path()),
                        &Options { io: Io::Suppress },
                    )
                })
                .collect::<Result<Vec<_>>>();
            match children {
                Ok(_) => {
                    ::std::process::exit(0);
                }
                Err(err) => err.into(),
            }
        } else {
            let cmd_line = self.exec.get_command_line(vec![]);
            let opt = Options { io: Io::Inherit };
            self.exec(
                cmd_line,
                self.path.as_ref().map(|path| path.as_path()),
                &opt,
            )
        }
    }
}

impl Search for ApplicationPart {
    fn search_terms(&self) -> SearchTerms {
        let mut terms = vec![self.name.clone()];
        if let Some(ref comment) = self.comment {
            terms.push(comment.clone())
        }
        use std::borrow::{Borrow, Cow};
        SearchTerms {
            terms: terms.into_iter().map(Cow::Owned).collect(),
            keywords: self.keywords
                .iter()
                .map(Borrow::borrow)
                .map(Cow::Borrowed)
                .collect(),
            related: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActionPart {
    name: String,
    icon: Option<String>,
    exec: Exec,
    application: Rc<ApplicationPart>,
}

impl ActionPart {
    fn from_desktop_action(
        desktop_action: DesktopAction,
        application: Rc<ApplicationPart>,
    ) -> Result<Rc<ActionPart>> {
        Ok(Rc::new(ActionPart {
            name: desktop_action.name,
            exec: desktop_action.exec.parse()?,
            icon: desktop_action.icon,
            application,
        }))
    }
}

impl Launch for ActionPart {
    fn launch(&self, args: Vec<String>) -> Error {
        unimplemented!()
    }
}

impl Display for ActionPart {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{} - {}", self.name, self.application.name)
    }
}

impl Search for ActionPart {
    fn search_terms(&self) -> SearchTerms {
//        SearchTerms {
//        }
        unimplemented!()
    }
}
