use std::path::PathBuf;

#[derive(Debug, Default, Builder, Eq, PartialEq)]
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
    pub icon: Option<String>,
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

#[derive(Debug, Default, Builder, Eq, PartialEq)]
pub struct DesktopAction {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into))]
    pub exec: String,
    #[builder(setter(into), default = "None")]
    pub icon: Option<String>,
}
