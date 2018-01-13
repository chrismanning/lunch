use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;

use lunch::errors::*;

use super::locale::Locale;
use super::parse::parse_desktop_groups;
use super::entry::*;

#[derive(Debug, Eq, PartialEq)]
pub struct DesktopFile {
    pub desktop_entry: DesktopEntry,
    pub actions: Vec<DesktopAction>,
}

impl DesktopFile {
    pub fn read<R: BufRead>(input: R, locale: &Locale) -> Result<DesktopFile> {
        let input = read_whole(input)?;
        let mut groups = parse_desktop_groups(&input, locale)?;
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
                "OnlyShowIn" => builder.only_show_in(
                    value
                        .split(';')
                        .filter(|val| !val.is_empty())
                        .map(|keyword| keyword.to_owned())
                        .collect(),
                ),
                "NotShowIn" => builder.not_show_in(
                    value
                        .split(';')
                        .filter(|val| !val.is_empty())
                        .map(|keyword| keyword.to_owned())
                        .collect(),
                ),
                "TryExec" => builder.try_exec(value),
                "Exec" => builder.exec(value),
                "Path" => builder.path(PathBuf::from(value)),
                "Actions" => builder.actions(
                    value
                        .split(';')
                        .filter(|val| !val.is_empty())
                        .map(|keyword| keyword.to_owned())
                        .collect(),
                ),
                "Categories" => builder.categories(
                    value
                        .split(';')
                        .filter(|val| !val.is_empty())
                        .map(|keyword| keyword.to_owned())
                        .collect(),
                ),
                "Keywords" => builder.keywords(
                    value
                        .split(';')
                        .filter(|val| !val.is_empty())
                        .map(|keyword| keyword.to_owned())
                        .collect(),
                ),
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

fn read_whole<R: BufRead>(mut reader: R) -> Result<String> {
    let mut string = String::new();
    reader.read_to_string(&mut string)?;
    Ok(string)
}

#[cfg(test)]
mod read_tests {
    use super::*;
    use spectral::prelude::*;
    use std::io::BufReader;

    #[test]
    fn test_all() {
        let input = "[Desktop Entry]
        Name=Some Desktop Application
        GenericName=App
        Type=Application
        Exec=exec
        Comment=comment
        Icon=icon
        Actions=test;
        Categories=Utility;;
        Keywords=word
        Hidden=true
        Path=/
        NoDisplay=false
        OnlyShowIn=A
        NotShowIn=B

        [Desktop Action test]
        Name=Test
        Exec=exec
        ";
        let locale = "C".parse().unwrap();

        let desktop_file = DesktopFile::read(BufReader::new(input.as_bytes()), &locale);
        assert_that(&desktop_file).is_ok().is_equal_to(DesktopFile {
            desktop_entry: DesktopEntry {
                entry_type: "Application".to_owned(),
                name: "Some Desktop Application".to_owned(),
                generic_name: Some("App".to_owned()),
                no_display: false,
                comment: Some("comment".to_owned()),
                icon: Some("icon".to_owned()),
                hidden: true,
                only_show_in: vec!["A".to_owned()],
                not_show_in: vec!["B".to_owned()],
                try_exec: None,
                exec: Some("exec".to_owned()),
                path: Some(PathBuf::from("/")),
                actions: vec!["test".to_owned()],
                mime_type: vec![],
                categories: vec!["Utility".to_owned()],
                keywords: vec!["word".to_owned()],
            },
            actions: vec![
                DesktopAction {
                    name: "Test".to_owned(),
                    exec: "exec".to_owned(),
                    icon: None,
                },
            ],
        });
    }

    #[test]
    fn test_bad_bool() {
        let input = "[Desktop Entry]
        Name=Some Desktop Application
        GenericName=App
        Type=Application
        Exec=exec
        Comment=comment
        Icon=icon
        Actions=test;
        Categories=Utility;;
        Keywords=word
        Hidden=trie

        [Desktop Action test]
        Name=Test
        Exec=exec
        ";
        let locale = "C".parse().unwrap();
        let desktop_file = DesktopFile::read(BufReader::new(input.as_bytes()), &locale);

        assert_that(&desktop_file).is_err();
    }
}
