use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;
use std::convert::TryInto;

use lunch::errors::*;
use lunch::env::Lunchable;

use super::locale::Locale;
use super::parse::parse_desktop_groups;
use super::entry::*;
use super::application::Application;

#[derive(Debug)]
pub struct DesktopFile {
    pub desktop_entry: DesktopEntry,
    pub actions: Vec<DesktopAction>,
}

impl DesktopFile {
    pub fn read<R: BufRead>(mut input: R, locale: &Locale) -> Result<DesktopFile> {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        let mut groups = parse_desktop_groups(&buf, locale)?;
        let desktop_entry = Self::convert_desktop_entry(groups
            .remove("Desktop Entry")
            .ok_or(ErrorKind::ApplicationNotFound)?)?;
        let actions = groups
            .into_iter()
            .filter(|&(ref key, _)| key.starts_with("Desktop Action "))
            .map(|(key, value)| {
                (
                    key.get("Desktop Action ".len()..).unwrap().to_owned(),
                    value,
                )
            })
            .filter(|&(ref key, _)| desktop_entry.actions.contains(&key))
            .map(|(_, value)| Self::convert_desktop_action(value))
            .collect::<Result<Vec<DesktopAction>>>()?;
        Ok(DesktopFile {
            desktop_entry,
            actions,
        })
    }

    fn convert_desktop_entry(desktop_entry_group: HashMap<String, String>) -> Result<DesktopEntry> {
        let mut builder = DesktopEntryBuilder::default();
        for (key, value) in desktop_entry_group {
            match key.as_ref() {
                "Type" => builder.entry_type(value),
                "Name" => builder.name(value),
                "GenericName" => builder.generic_name(value),
                "NoDisplay" => builder.no_display(value.parse()?),
                "Comment" => builder.comment(value),
                "Icon" => builder.icon(value),
                "Hidden" => builder.hidden(value.parse()?),
                "OnlyShowIn" => builder
                    .only_show_in(value.split(';').map(|keyword| keyword.to_owned()).collect()),
                "NotShowIn" => builder
                    .not_show_in(value.split(';').map(|keyword| keyword.to_owned()).collect()),
                "TryExec" => builder.try_exec(value),
                "Exec" => builder.exec(value),
                "Path" => builder.path(PathBuf::from(value)),
                "Actions" => {
                    builder.actions(value.split(';').map(|keyword| keyword.to_owned()).collect())
                }
                "Categories" => {
                    builder.categories(value.split(';').map(|keyword| keyword.to_owned()).collect())
                }
                "Keywords" => {
                    builder.keywords(value.split(';').map(|keyword| keyword.to_owned()).collect())
                }
                _ => &mut builder,
            };
        }

        builder.build().map_err(|s| s.into())
    }

    fn convert_desktop_action(
        desktop_action_group: HashMap<String, String>,
    ) -> Result<DesktopAction> {
        unimplemented!()
    }
}

impl TryInto<Box<Lunchable>> for DesktopFile {
    type Error = Error;

    fn try_into(self) -> Result<Box<Lunchable>> {
        let app: Application = self.try_into()?;
        Ok(Box::new(app))
    }
}

impl TryInto<Application> for DesktopFile {
    type Error = Error;

    fn try_into(self) -> Result<Application> {
        // TODO convert all the things
        Ok(Application {
            name: self.desktop_entry.name,
            icon: self.desktop_entry.icon,
            comment: self.desktop_entry.comment,
            keywords: self.desktop_entry.keywords,
            exec: self.desktop_entry
                .exec
                .ok_or(ErrorKind::InvalidCommandLine("".into()).into())
                .and_then(|s| s.parse())?,
            field_code: None,
            try_exec: self.desktop_entry.try_exec.map(From::from),
            path: self.desktop_entry.path.map(From::from),
            actions: Vec::new(),
        })
    }
}
