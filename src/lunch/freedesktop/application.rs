use std::path::{Path, PathBuf};
use std::process::{Command, Child, Stdio};
use std::convert::{From, TryFrom};
use std::fmt::{Result as FmtResult, Display, Formatter};

use freedesktop::exec::{Exec, CmdLine, FieldCode};
use lunch::{Options, Io, Launch};
use lunch::errors::*;

pub struct Application {
    name: String,
    icon: Option<String>,
    exec: Exec,
    field_code: Option<FieldCode>,
    try_exec: Option<PathBuf>,
    path: Option<PathBuf>,
    actions: Vec<Action>,
}

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

    fn spawn<'a>(&self, cmd_line: CmdLine<'a>, opt: &Options) -> Result<Child> {
        debug!("spawning {:?}", cmd_line);
        let mut cmd = Command::new(cmd_line.cmd);
        cmd.args(cmd_line.args);
        if let Some(ref path) = self.path {
            if path.exists() {
                cmd.current_dir(path);
            }
        }
        match opt.io {
            Io::Suppress => {
                cmd.stdout(Stdio::null()).stderr(Stdio::null()).stdin(
                    Stdio::null(),
                );
            }
            Io::Inherit => {
                cmd.stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .stdin(Stdio::inherit());
            }
        }
        cmd.spawn().chain_err(|| "Error spawning process")
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
                        self.spawn(cmd_line, &Options { io: Io::Suppress })
                    })
                    .collect()
            } else {
                let cmd_line = self.exec.get_command_line(vec![]);
                let opt = Options { io: Io::Inherit };
                self.spawn(cmd_line, &opt).map(|child| vec![child])
            };
            match children {
                Ok(_) => {
                    ::std::process::exit(0);
                }
                Err(err) => {
                    err.into()
                }
            }
        } else {
            ErrorKind::ApplicationNotFound.into()
        }
    }
}
