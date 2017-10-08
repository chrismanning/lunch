use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;

use lunch::errors::*;
use freedesktop::locale::Locale;
use freedesktop::parse::parse_groups;
use freedesktop::entry::*;

pub struct DesktopFile {
    desktop_entry: DesktopEntry,
    actions: Vec<DesktopAction>,
}

impl DesktopFile {

    pub fn read<R: BufRead>(input: R, locale: &Locale) -> Result<DesktopFile> {
        let mut groups = parse_groups(
            input.lines().map(
                |res| res.chain_err(|| "Error reading file"),
            ),
            |header| header.starts_with("Desktop "),
            locale,
        )?;
        let desktop_entry = Self::convert_desktop_entry(groups.remove("Desktop Entry")
            .ok_or(ErrorKind::ApplicationNotFound)?)?;
        let actions = groups.into_iter()
            .filter(|&(ref key, _)| key.starts_with("Desktop Action "))
            .map(|(key, value)| (key.get("Desktop Action ".len()..).unwrap().to_owned(), value))
            .filter(|&(ref key, _)| desktop_entry.actions.contains(&key))
            .map(|(key, value)| Self::convert_desktop_action(value))
            .collect::<Result<Vec<DesktopAction>>>()?;
        Ok(DesktopFile {
            desktop_entry: desktop_entry,
            actions: actions,
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
                "Icon" => builder.icon(PathBuf::from(value)),
                "Hidden" => builder.hidden(value.parse()?),
                "OnlyShowIn" => builder.only_show_in(value.split(';')
                    .map(|keyword| keyword.to_owned()).collect()),
                "NotShowIn" => builder.not_show_in(value.split(';')
                    .map(|keyword| keyword.to_owned()).collect()),
                "TryExec" => builder.try_exec(value),
                "Exec" => builder.exec(value),
                "Path" => builder.path(PathBuf::from(value)),
                "Actions" => builder.actions(value.split(';')
                    .map(|keyword| keyword.to_owned()).collect()),
                "Categories" => builder.categories(value.split(';')
                    .map(|keyword| keyword.to_owned()).collect()),
                "Keywords" => builder.keywords(value.split(';')
                    .map(|keyword| keyword.to_owned()).collect()),
                _ => &builder,
            };
        }

        builder.build().map_err(|s| s.into())
    }

    fn convert_desktop_action(desktop_action_group: HashMap<String, String>) -> Result<DesktopAction> {
        unimplemented!()
    }
}
