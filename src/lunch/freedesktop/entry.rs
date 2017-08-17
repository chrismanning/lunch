use std::path::{Path, PathBuf};
use std::io::BufRead;
use std::process::{Command, Child, Stdio};
use std::convert::{From, TryFrom};
use std::str::FromStr;
use std::fmt::{Display, Formatter};
use std::fmt::Result as FmtResult;

use lunch::errors::*;
use lunch::*;

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
    #[builder(default = "vec![]")]
    pub actions: Vec<String>,
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
                "GenericName" => builder.generic_name(value),
                "NoDisplay" => builder.no_display(value.parse()?),
                "Comment" => builder.comment(value),
                "Exec" => builder.exec(value),
                "TryExec" => builder.try_exec(value),
                _ => &builder,
            };
        }

        builder.build().map_err(|s| s.into())
    }

    pub fn get_executable(self) -> Box<Launch> {
        //        let app: ApplicationEntry = self.into();

        unimplemented!()
    }
}

#[derive(Debug, Clone)]
enum Arg {
    FieldCode,
    StaticArg(String),
}

#[derive(Debug, Clone)]
struct Exec {
    exec: String,
    args: Vec<Arg>,
}

impl Exec {
    fn get_command_line(&self, user_args: Vec<String>) -> CmdLine {
        assert!(self.args.iter()
            .filter(|arg| match **arg { Arg::FieldCode => true, _ => false })
            .count() <= 1);
        CmdLine {
            cmd: &self.exec,
            args: self.args
                .iter()
                .flat_map(|arg| match *arg {
                    Arg::StaticArg(ref arg) => vec![arg.clone()].into_iter(),
                    Arg::FieldCode => user_args.clone().into_iter(),
                })
                .collect(),
        }
    }
}

impl FromStr for Exec {
    type Err = Error;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        unimplemented!()
    }
}

#[derive(Debug)]
struct CmdLine<'a> {
    cmd: &'a str,
    args: Vec<String>,
}

#[derive(Debug, Copy, Clone)]
pub enum FieldCode {
    SingleFile,
    MultipleFiles,
    SingleUrl,
    MultipleUrls,
    Icon,
    Name,
    EntryUri,
}

impl FieldCode {
    fn expand_exec<'a>(&self, exec: &'a Exec, args: Vec<String>) -> Vec<CmdLine<'a>> {
        use self::FieldCode::*;
        if args.is_empty() {
            vec![exec.get_command_line(args)]
        } else {
            match *self {
                SingleFile | SingleUrl => {
                    args.into_iter()
                        .map(|arg| exec.get_command_line(vec![arg]))
                        .collect()
                }
                MultipleFiles | MultipleUrls => vec![exec.get_command_line(args)],
                _ => vec![exec.get_command_line(vec![])],
            }
        }
    }
}

#[derive(Debug)]
pub struct ApplicationEntry {
    name: String,
    exec: Exec,
    field_code: Option<FieldCode>,
    try_exec: Option<PathBuf>,
    path: Option<PathBuf>,
    keywords: Vec<String>,
    action: Vec<String>,
}

impl ApplicationEntry {
    fn can_exec(&self) -> bool {
        if let Some(ref try_exec) = self.try_exec {
            return Path::new(try_exec).exists();
        }
        true
    }
}

impl Display for ApplicationEntry {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.name)
    }
}

impl TryFrom<DesktopEntry> for ApplicationEntry {
    type Error = self::Error;

    fn try_from(value: DesktopEntry) -> Result<Self> {
        // TODO make sure to parse exec line - this may contain non-field-code args
        unimplemented!()
    }
}

impl Application for ApplicationEntry {}

impl ApplicationIndex for ApplicationEntry {}

impl Launch for ApplicationEntry {
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
                    return err.into();
                }
            }
        } else {
            ErrorKind::ApplicationNotFound.into()
        }
    }
}

impl ApplicationEntry {
    fn spawn<'a>(&self, cmd_line: CmdLine<'a>, opt: &Options) -> Result<Child> {
        debug!("spawning {:?}", cmd_line);
        let mut cmd = Command::new(cmd_line.cmd);
        cmd.args(cmd_line.args);
        if let Some(ref path) = self.path {
            cmd.current_dir(path);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn exec_multiple_field_codes() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::FieldCode, Arg::FieldCode],
        };
        exec.get_command_line(vec![]);
    }

    #[test]
    fn exec_no_args() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![],
        };
        let cmd_line = exec.get_command_line(vec![]);
        assert_eq!(cmd_line.args, vec![] as Vec<String>);
        assert_eq!(cmd_line.cmd, "echo");
    }

    #[test]
    fn exec_all_static_args() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::StaticArg("-n".into()), Arg::StaticArg("-e".into())],
        };
        let cmd_line = exec.get_command_line(vec![]);
        assert_eq!(cmd_line.args, vec!["-n".to_owned(), "-e".to_owned()]);
        assert_eq!(cmd_line.cmd, "echo");
    }

    #[test]
    fn exec_field_code_args() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::FieldCode],
        };
        let cmd_line = exec.get_command_line(vec!["-n".to_owned(), "-e".to_owned()]);
        assert_eq!(cmd_line.args, vec!["-n".to_owned(), "-e".to_owned()]);
        assert_eq!(cmd_line.cmd, "echo");
    }

    #[test]
    fn exec_mixed_args() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::StaticArg("-n".into()), Arg::FieldCode, Arg::StaticArg("-e".into())],
        };
        let cmd_line = exec.get_command_line(vec!["-E".to_owned()]);
        assert_eq!(cmd_line.args, vec!["-n".to_owned(), "-E".to_owned(), "-e".to_owned()]);
        assert_eq!(cmd_line.cmd, "echo");
    }

    #[test]
    fn field_code_single_file_no_args() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::FieldCode],
        };
        let cmd_lines = FieldCode::SingleFile.expand_exec(&exec, vec![]);
        assert_eq!(1, cmd_lines.len());
        let cmd_line = &cmd_lines[0];
        assert_eq!(cmd_line.args, vec![] as Vec<String>);
        assert_eq!(cmd_line.cmd, "echo");
    }

    #[test]
    fn field_code_single_file_single_arg() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::FieldCode],
        };
        let cmd_lines = FieldCode::SingleFile.expand_exec(&exec, vec!["arg".to_owned()]);
        assert_eq!(1, cmd_lines.len());
        let cmd_line = &cmd_lines[0];
        assert_eq!(cmd_line.args, vec!["arg".to_owned()]);
        assert_eq!(cmd_line.cmd, "echo");
    }

    #[test]
    fn field_code_single_file_multiple_args() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::FieldCode],
        };
        let cmd_lines = FieldCode::SingleFile.expand_exec(&exec,
                                                          vec!["arg1".to_owned(), "arg2".to_owned()]);
        assert_eq!(2, cmd_lines.len());
        let cmd_line = &cmd_lines[0];
        assert_eq!(cmd_line.args, vec!["arg1".to_owned()]);
        assert_eq!(cmd_line.cmd, "echo");
        let cmd_line = &cmd_lines[1];
        assert_eq!(cmd_line.args, vec!["arg2".to_owned()]);
        assert_eq!(cmd_line.cmd, "echo");
    }

    #[test]
    fn field_code_multiple_files_single_arg() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::FieldCode],
        };
        let cmd_lines = FieldCode::MultipleFiles.expand_exec(&exec, vec!["arg".to_owned()]);
        assert_eq!(1, cmd_lines.len());
        let cmd_line = &cmd_lines[0];
        assert_eq!(cmd_line.args, vec!["arg".to_owned()]);
        assert_eq!(cmd_line.cmd, "echo");
    }

    #[test]
    fn field_code_multiple_files_multiple_args() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::FieldCode],
        };
        let cmd_lines = FieldCode::MultipleFiles.expand_exec(&exec,
                                                          vec!["arg1".to_owned(), "arg2".to_owned()]);
        assert_eq!(1, cmd_lines.len());
        let cmd_line = &cmd_lines[0];
        assert_eq!(cmd_line.args, vec!["arg1".to_owned(), "arg2".to_owned()]);
        assert_eq!(cmd_line.cmd, "echo");
    }
}
