use std::process::{Child, Command, Stdio};
use std::path::Path;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use super::errors::*;
use super::exec::CmdLine;
use lunch::{Io, Options};

pub trait Launch {
    fn launch(&self, args: Vec<String>) -> Error;

    fn spawn(&self, cmd_line: CmdLine, work_dir: Option<&Path>, opt: &Options) -> Result<Child> {
        debug!("spawning {:?}", cmd_line);
        let mut cmd = init_cmd(cmd_line, work_dir, opt);
        cmd.spawn().chain_err(|| "Error spawning process")
    }

    #[cfg(unix)]
    fn exec(&self, cmd_line: CmdLine, work_dir: Option<&Path>, opt: &Options) -> Error {
        debug!("execing {:?}", cmd_line);
        let mut cmd = init_cmd(cmd_line, work_dir, opt);
        cmd.exec().into()
    }
}

fn init_cmd(cmd_line: CmdLine, work_dir: Option<&Path>, opt: &Options) -> Command {
    let mut cmd = Command::new(cmd_line.cmd);
    cmd.args(cmd_line.args);
    if let Some(ref path) = work_dir {
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
    cmd
}
