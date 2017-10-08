use std::path::PathBuf;

use lunch::errors::*;
use lunch::*;

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
    pub actions: Vec<String>,
    #[builder(default = "vec![]")]
    pub mime_type: Vec<String>,
    #[builder(default = "vec![]")]
    pub categories: Vec<String>,
    #[builder(default = "vec![]")]
    pub keywords: Vec<String>,
}

pub struct DesktopAction {
    name: String,
    exec: String,
    icon: Option<String>,
    // TODO scan for (DesktopEntry, Vec<DesktopAction>)
    // TODO convert to Application (move ApplicationEntry impl to Application)
}
