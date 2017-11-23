use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;

use lunch::errors::*;

use super::locale::Locale;
use super::parse::parse_desktop_groups;
use super::entry::*;

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
        let desktop_entry = Self::build_desktop_entry(groups
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
            .map(|(_, value)| Self::build_desktop_action(value))
            .collect::<Result<Vec<DesktopAction>>>()?;
        Ok(DesktopFile {
            desktop_entry,
            actions,
        })
    }

    fn build_desktop_entry(desktop_entry_group: HashMap<String, String>) -> Result<DesktopEntry> {
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

    fn build_desktop_action(
        desktop_action_group: HashMap<String, String>,
    ) -> Result<DesktopAction> {
        let mut builder = DesktopActionBuilder::default();
        for (key, value) in desktop_action_group {
            match key.as_ref() {
                "Name" => builder.name(value),
                "Icon" => builder.icon(value),
                "Exec" => builder.exec(value),
                _ => &mut builder,
            };
        }

        builder.build().map_err(|s| s.into())
    }
}
