use std::result::Result as StdResult;
use std::str::FromStr;

use lunch::errors::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Exec {
    exec: String,
    args: Vec<Arg>,
}

impl Exec {
    pub fn get_command_line(&self, user_args: Vec<String>) -> CmdLine {
        assert!(
            self.args
                .iter()
                .filter(|arg| match **arg {
                    Arg::FieldCode => true,
                    _ => false,
                })
                .count() <= 1
        );
        CmdLine {
            cmd: self.exec.clone(),
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
        let tokens: Vec<_> = split_command_line(s)?;

        if let [ref cmd, ref args..] = tokens[..] {
            Ok(Exec {
                exec: cmd.to_owned(),
                args: args.iter()
                    .map(|arg| arg.parse())
                    .collect::<Result<Vec<Arg>>>()?,
            })
        } else {
            Err(ErrorKind::InvalidCommandLine(s.to_owned()).into())
        }
    }
}

fn split_command_line(cmd_line: &str) -> Result<Vec<String>> {
    let mut slices = Vec::new();
    let mut chars = cmd_line.chars();
    let mut token: String = String::new();
    let mut quote = false;
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(c) = chars.next() {
                    match c {
                        'n' => token.push('\n'),
                        't' => token.push('\t'),
                        _ => token.push(c),
                    }
                }
            }
            ' ' => {
                if quote {
                    token.push(c);
                } else {
                    slices.push(token.clone());
                    token.clear();
                }
            }
            '"' => {
                quote = !quote;
            }
            _ => {
                token.push(c);
            }
        }
    }
    if !token.is_empty() {
        slices.push(token);
    }
    Ok(slices)
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Arg {
    FieldCode,
    StaticArg(String),
}

impl FromStr for Arg {
    type Err = Error;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        match s {
            "%f" | "%F" | "%u" | "%U" => Ok(Arg::FieldCode),
            _ => Ok(Arg::StaticArg(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod exec_tests {
    use super::*;
    use spectral::prelude::*;

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
        assert_that!(cmd_line.args).is_equal_to(vec![] as Vec<String>);
        assert_that!(cmd_line.cmd).is_equal_to("echo".to_owned());
    }

    #[test]
    fn exec_all_static_args() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::StaticArg("-n".into()), Arg::StaticArg("-e".into())],
        };
        let cmd_line = exec.get_command_line(vec![]);
        assert_that!(cmd_line.args).is_equal_to(vec!["-n".to_owned(), "-e".to_owned()]);
        assert_that!(cmd_line.cmd).is_equal_to("echo".to_owned());
    }

    #[test]
    fn exec_field_code_args() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::FieldCode],
        };
        let cmd_line = exec.get_command_line(vec!["-n".to_owned(), "-e".to_owned()]);
        assert_that!(cmd_line.args).is_equal_to(vec!["-n".to_owned(), "-e".to_owned()]);
        assert_that!(cmd_line.cmd).is_equal_to("echo".to_owned());
    }

    #[test]
    fn exec_mixed_args() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![
                Arg::StaticArg("-n".into()),
                Arg::FieldCode,
                Arg::StaticArg("-e".into()),
            ],
        };
        let cmd_line = exec.get_command_line(vec!["-E".to_owned()]);
        assert_that!(cmd_line.args).is_equal_to(vec![
            "-n".to_owned(),
            "-E".to_owned(),
            "-e".to_owned(),
        ]);
        assert_that!(cmd_line.cmd).is_equal_to("echo".to_owned());
    }

    #[test]
    fn exec_parse_cmd_only() {
        let exec: Exec = "echo".parse().unwrap();
        assert_that!(&exec.exec).is_equal_to("echo".to_owned());
        assert_that!(exec.args).is_equal_to(vec![]);
    }

    #[test]
    fn exec_parse_args() {
        let exec: Exec = "echo -n -e".parse().unwrap();
        assert_that!(&exec.exec).is_equal_to("echo".to_owned());
        assert_that!(exec.args).is_equal_to(vec![
            Arg::StaticArg("-n".to_owned()),
            Arg::StaticArg("-e".to_owned()),
        ]);
    }

    #[test]
    fn exec_parse_field_code() {
        let exec: Exec = "echo -n %f -e".parse().unwrap();
        assert_that!(&exec.exec).is_equal_to("echo".to_owned());
        assert_that!(exec.args).is_equal_to(vec![
            Arg::StaticArg("-n".to_owned()),
            Arg::FieldCode,
            Arg::StaticArg("-e".to_owned()),
        ]);
    }

    #[test]
    fn exec_parse_space() {
        let exec: Exec = r"/opt/Echo\ 2/echo -n %f -e".parse().unwrap();
        assert_that!(&exec.exec).is_equal_to(r"/opt/Echo 2/echo".to_owned());
        assert_that!(exec.args).is_equal_to(vec![
            Arg::StaticArg("-n".to_owned()),
            Arg::FieldCode,
            Arg::StaticArg("-e".to_owned()),
        ]);
    }

    #[test]
    fn exec_quoted_arg() {
        let exec: Exec = r##"/opt/Echo\ 2/echo -n %f -e "arg with spaces" -v"##
            .parse()
            .unwrap();
        assert_that!(&exec.exec).is_equal_to("/opt/Echo 2/echo".to_owned());
        assert_that!(exec.args).is_equal_to(vec![
            Arg::StaticArg("-n".to_owned()),
            Arg::FieldCode,
            Arg::StaticArg("-e".to_owned()),
            Arg::StaticArg("arg with spaces".to_owned()),
            Arg::StaticArg("-v".to_owned()),
        ]);
    }
}

#[derive(Debug)]
pub struct CmdLine {
    pub cmd: String,
    pub args: Vec<String>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FieldCode {
    SingleFile,
    MultipleFiles,
    SingleUrl,
    MultipleUrls,
}

impl FieldCode {
    pub fn extract_field_code(exec: &str) -> Option<FieldCode> {
        let field_code = if exec.contains("%f") {
            FieldCode::SingleFile
        } else if exec.contains("%F") {
            FieldCode::MultipleFiles
        } else if exec.contains("%u") {
            FieldCode::SingleUrl
        } else if exec.contains("%U") {
            FieldCode::MultipleUrls
        } else {
            return None;
        };
        Some(field_code)
    }

    pub fn expand_exec(&self, exec: &Exec, args: Vec<String>) -> Vec<CmdLine> {
        use self::FieldCode::*;
        if args.is_empty() {
            vec![exec.get_command_line(args)]
        } else {
            match *self {
                SingleFile | SingleUrl => args.into_iter()
                    .map(|arg| exec.get_command_line(vec![arg]))
                    .collect(),
                MultipleFiles | MultipleUrls => vec![exec.get_command_line(args)],
            }
        }
    }
}

#[cfg(test)]
mod field_code_tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn test_extract() {
        assert_that!(FieldCode::extract_field_code("/bin/echo %f"))
            .is_some()
            .is_equal_to(FieldCode::SingleFile);
        assert_that!(FieldCode::extract_field_code("/bin/echo %F"))
            .is_some()
            .is_equal_to(FieldCode::MultipleFiles);
        assert_that!(FieldCode::extract_field_code("/bin/echo %u"))
            .is_some()
            .is_equal_to(FieldCode::SingleUrl);
        assert_that!(FieldCode::extract_field_code("/bin/echo %U"))
            .is_some()
            .is_equal_to(FieldCode::MultipleUrls);

        assert_that!(FieldCode::extract_field_code("/bin/echo %G")).is_none();
        assert_that!(FieldCode::extract_field_code("/bin/echo")).is_none();
    }

    #[test]
    fn field_code_single_file_no_args() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::FieldCode],
        };
        let cmd_lines = FieldCode::SingleFile.expand_exec(&exec, vec![]);
        assert_that!(cmd_lines.len()).is_equal_to(1);
        let cmd_line = &cmd_lines[0];
        assert_that!(cmd_line.args).is_equal_to(vec![] as Vec<String>);
        assert_that!(cmd_line.cmd).is_equal_to("echo".to_owned());
    }

    #[test]
    fn field_code_single_file_single_arg() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::FieldCode],
        };
        let cmd_lines = FieldCode::SingleFile.expand_exec(&exec, vec!["arg".to_owned()]);
        assert_that!(cmd_lines.len()).is_equal_to(1);
        let cmd_line = &cmd_lines[0];
        assert_that!(cmd_line.args).is_equal_to(vec!["arg".to_owned()]);
        assert_that!(cmd_line.cmd).is_equal_to("echo".to_owned());
    }

    #[test]
    fn field_code_single_file_multiple_args() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::FieldCode],
        };
        let cmd_lines =
            FieldCode::SingleFile.expand_exec(&exec, vec!["arg1".to_owned(), "arg2".to_owned()]);
        assert_that!(cmd_lines.len()).is_equal_to(2);
        let cmd_line = &cmd_lines[0];
        assert_that!(cmd_line.args).is_equal_to(vec!["arg1".to_owned()]);
        assert_that!(cmd_line.cmd).is_equal_to("echo".to_owned());
        let cmd_line = &cmd_lines[1];
        assert_that!(cmd_line.args).is_equal_to(vec!["arg2".to_owned()]);
        assert_that!(cmd_line.cmd).is_equal_to("echo".to_owned());
    }

    #[test]
    fn field_code_multiple_files_single_arg() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::FieldCode],
        };
        let cmd_lines = FieldCode::MultipleFiles.expand_exec(&exec, vec!["arg".to_owned()]);
        assert_that!(cmd_lines.len()).is_equal_to(1);
        let cmd_line = &cmd_lines[0];
        assert_that!(cmd_line.args).is_equal_to(vec!["arg".to_owned()]);
        assert_that!(cmd_line.cmd).is_equal_to("echo".to_owned());
    }

    #[test]
    fn field_code_multiple_files_multiple_args() {
        let exec = Exec {
            exec: "echo".to_owned(),
            args: vec![Arg::FieldCode],
        };
        let cmd_lines =
            FieldCode::MultipleFiles.expand_exec(&exec, vec!["arg1".to_owned(), "arg2".to_owned()]);
        assert_that!(cmd_lines.len()).is_equal_to(1);
        let cmd_line = &cmd_lines[0];
        assert_that!(cmd_line.args).is_equal_to(vec!["arg1".to_owned(), "arg2".to_owned()]);
        assert_that!(cmd_line.cmd).is_equal_to("echo".to_owned());
    }
}
