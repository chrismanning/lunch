use std::path::{Path, PathBuf};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs;
use std::rc::Rc;
use std::convert::TryFrom;
use std::os::unix::fs::MetadataExt;
use std::ffi::OsString;

use users::*;
use users::os::unix::GroupExt;

use super::desktopfile::DesktopFile;
use super::entry::*;
use lunch::errors::*;
use lunch::exec::{Exec, FieldCode};
use lunch::{Io, Launch, Lunchable, Options, Search};
use lunch::search::SearchTerms;

#[derive(Debug)]
pub struct Application {
    app_part: Rc<ApplicationPart>,
    action_parts: Vec<Rc<ActionPart>>,
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
        Ok(Application {
            app_part: app_data,
            action_parts: actions,
        })
    }
}

impl Application {
    pub fn can_exec(&self) -> bool {
        self.app_part.can_exec()
    }

    pub fn to_lunchables(self) -> Vec<Rc<Lunchable>> {
        let (app, actions) = (self.app_part, self.action_parts);
        let mut actions = actions
            .clone()
            .into_iter()
            .map(|action| action as Rc<Lunchable>)
            .collect();
        let mut lunchables: Vec<Rc<Lunchable>> = vec![app];
        lunchables.append(&mut actions);
        lunchables
    }
}

#[derive(Debug)]
struct ApplicationPart {
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
            .map(|try_exec| can_exec(try_exec.as_path(), ::std::env::var_os("PATH")))
            .unwrap_or_else(|| true)
    }
}

fn can_exec(try_exec: &Path, env_path: Option<OsString>) -> bool {
    if try_exec.is_absolute() {
        try_exec.exists() && is_executable(try_exec)
    } else if let Some(paths) = env_path {
        for path in ::std::env::split_paths(&paths) {
            debug!(
                "Looking for '{}' in '{}'",
                try_exec.display(),
                path.display()
            );
            if path.is_absolute() && path.exists() {
                let path = path.join(try_exec);
                if path.exists() {
                    return is_executable(&path);
                }
            }
        }
        false
    } else {
        false
    }
}

fn is_executable(path: &Path) -> bool {
    debug!("Path '{}' exists", path.display());
    match fs::metadata(&path) {
        Ok(metadata) => {
            debug!("Path '{}' executable: {}", path.display(), metadata.exec());
            metadata.exec()
        }
        Err(_err) => {
            debug!(
                "Could not determine if path '{}' is executable",
                path.display()
            );
            false
        }
    }
}

#[cfg(test)]
mod can_exec_tests {
    use super::*;
    use spectral::prelude::*;
    use std::fs::{set_permissions, File};
    use std::os::unix::fs::PermissionsExt;
    use tempdir::TempDir;

    #[test]
    fn test_nonexistant() {
        let tmp_dir = TempDir::new("can_exec").unwrap();
        assert_that!(can_exec(
            Path::new("test_nonexistant"),
            Some(tmp_dir.path().as_os_str().to_owned())
        )).is_equal_to(false);
    }

    #[test]
    fn test_relative() {
        let tmp_dir = TempDir::new("can_exec").unwrap();
        let path = tmp_dir.path().join("test_relative");
        let _file = File::create(&path).unwrap();
        set_permissions(&path, PermissionsExt::from_mode(0o777)).unwrap();

        assert_that!(can_exec(
            Path::new("test_relative"),
            Some(tmp_dir.path().as_os_str().to_owned())
        )).is_equal_to(true);
    }

    #[test]
    fn test_relative_not_exec() {
        let tmp_dir = TempDir::new("can_exec").unwrap();
        let path = tmp_dir.path().join("test_relative_not_exec");
        let _file = File::create(&path).unwrap();
        set_permissions(&path, PermissionsExt::from_mode(0o666)).unwrap();

        assert_that!(can_exec(
            Path::new("test_relative_not_exec"),
            Some(tmp_dir.path().as_os_str().to_owned())
        )).is_equal_to(false);
    }

    #[test]
    fn test_absolute() {
        let tmp_dir = TempDir::new("can_exec").unwrap();
        let path = tmp_dir.path().join("test_absolute");
        let _file = File::create(&path).unwrap();
        set_permissions(&path, PermissionsExt::from_mode(0o777)).unwrap();

        assert_that!(can_exec(&path, None)).is_equal_to(true);
    }

    #[test]
    fn test_absolute_not_exec() {
        let tmp_dir = TempDir::new("can_exec").unwrap();
        let path = tmp_dir.path().join("test_absolute_not_exec");
        let _file = File::create(&path).unwrap();
        set_permissions(&path, PermissionsExt::from_mode(0o666)).unwrap();

        assert_that!(can_exec(&path, None)).is_equal_to(false);
    }
}

trait MetadataExecExt {
    fn exec(&self) -> bool;
    fn exec_owner(&self) -> bool;
    fn exec_group(&self) -> bool;
    fn exec_others(&self) -> bool;
}

impl MetadataExecExt for fs::Metadata {
    fn exec(&self) -> bool {
        let current_uid = get_effective_uid();
        if current_uid == self.uid() && self.exec_owner() {
            return true;
        }
        if self.exec_owner() {
            if let Some(group) = get_group_by_gid(self.gid()) {
                if let Some(user) = get_user_by_uid(current_uid) {
                    if group
                        .members()
                        .iter()
                        .any(|member| user.name() == member.as_str())
                    {
                        return true;
                    }
                }
            }
        }
        self.exec_others()
    }

    fn exec_owner(&self) -> bool {
        let mode: u32 = self.mode();
        mode & 0o100 != 0
    }

    fn exec_group(&self) -> bool {
        let mode: u32 = self.mode();
        mode & 0o010 != 0
    }

    fn exec_others(&self) -> bool {
        let mode: u32 = self.mode();
        mode & 0o001 != 0
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
struct ActionPart {
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
        if !self.application.can_exec() {
            return ErrorKind::ApplicationNotFound.into();
        }
        info!("Launching '{}'...", self);

        let cmd_line = self.exec.get_command_line(vec![]);
        let opt = Options { io: Io::Inherit };
        self.exec(cmd_line, None, &opt)
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
