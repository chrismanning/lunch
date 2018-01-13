use std::collections::HashMap;
use std::iter::Peekable;

use peeking_take_while::PeekableExt;

use lunch::errors::*;
use super::locale::Locale;

type Group = HashMap<String, String>;
type Groups = HashMap<String, Group>;

pub fn parse_desktop_groups(src: &str, locale: &Locale) -> Result<Groups> {
    let mut groups = Groups::new();
    let mut lines = src.lines().peekable();
    while lines.peek().is_some() {
        if let Some((header, localised_group)) = parse_localised_desktop_group(&mut lines) {
            let group = localised_group.resolve_to_locale(locale);
            groups.insert(header, group);
        }
    }
    if groups.is_empty() {
        Err(ErrorKind::NoGroupsFound.into())
    } else {
        Ok(groups)
    }
}

#[cfg(test)]
mod parse_desktop_groups_tests {
    use super::*;

    #[test]
    fn empty_group_err() {
        let input = "";
        let groups = parse_desktop_groups(input, &Locale::default());
        assert!(groups.is_err());
    }

    #[test]
    fn parse_desktop_groups_default_locale() {
        let input = "[group header]
        # Comment
        Key1=Value1
        Key1[en]=Value2
        Key2[C]=Value3

        [Desktop Group]
        # Top comment
        Key=Value
        # Middle comment
        Key=Overwritten Value
        # Bottom comment
        ";
        let groups = parse_desktop_groups(input, &Locale::default());
        assert_eq!(
            groups.unwrap(),
            hashmap!{
                "Desktop Group".to_owned() => hashmap!{
                    "Key".to_owned() => "Overwritten Value".to_owned(),
                }
            }
        );
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct LocalisedGroup {
    group: HashMap<String, LocalisedValue>,
}

impl LocalisedGroup {
    fn resolve_to_locale(self, locale: &Locale) -> Group {
        self.group
            .into_iter()
            .map(|(key, mut localised_value)| {
                (
                    key,
                    localised_value
                        .remove(locale)
                        .or_else(|| localised_value.remove(&Locale::default())),
                )
            })
            .filter_map(|(key, value)| value.map(|value| (key, value)))
            .collect()
    }
}

#[cfg(test)]
mod localised_group_tests {
    use super::*;

    #[test]
    fn resolve_to_locale() {
        let localised_group = LocalisedGroup {
            group: hashmap!{
                "Key1".to_owned() => LocalisedValue {
                    localised_value: vec!{
                        (Locale::default(), "def".to_owned()),
                        ("en".parse::<Locale>().unwrap(), "en".to_owned()),
                    }
                },
                "Key2".to_owned() => LocalisedValue {
                    localised_value: vec!{
                        ("C".parse::<Locale>().unwrap(), "C".to_owned()),
                    }
                }
            },
        };
        assert_eq!(
            localised_group.resolve_to_locale(&"C".parse().unwrap()),
            hashmap!{
                "Key1".to_owned() => "def".to_owned(),
                "Key2".to_owned() => "C".to_owned(),
            }
        );
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct LocalisedValue {
    localised_value: Vec<(Locale, String)>,
}

impl LocalisedValue {
    fn insert(&mut self, locale: Locale, val: String) {
        if let Some(idx) = self.get_idx(&locale) {
            self.localised_value.push((locale, val));
            self.localised_value.swap_remove(idx);
        } else {
            self.localised_value.push((locale, val));
        }
    }

    fn remove(&mut self, locale: &Locale) -> Option<String> {
        let idx = self.get_idx(locale);
        idx.map(|idx| self.localised_value.remove(idx))
            .map(|(_, value)| value)
    }

    fn get_idx(&self, locale: &Locale) -> Option<usize> {
        self.localised_value
            .iter()
            .enumerate()
            .map(|(idx, &(ref key, _))| (idx, key))
            .max_by_key(|&(_, locale_key)| locale.match_level(locale_key))
            .and_then(|(idx, locale_key)| locale.match_level(locale_key).and(Some(idx)))
    }
}

#[cfg(test)]
mod localised_value_tests {
    use super::*;

    #[test]
    fn get_exact() {
        let mut localised_value = LocalisedValue {
            localised_value: vec![
                ("en".parse().unwrap(), "en".to_owned()),
                ("en_GB".parse().unwrap(), "en_GB".to_owned()),
            ],
        };
        let value = localised_value.remove(&"en_GB".parse().unwrap()).unwrap();
        assert_eq!(value, "en_GB");
    }

    #[test]
    fn get_same_lang() {
        let mut localised_value = LocalisedValue {
            localised_value: vec![("en".parse().unwrap(), "en".to_owned())],
        };
        let value = localised_value.remove(&"en_GB".parse().unwrap()).unwrap();
        assert_eq!(value, "en");
    }

    #[test]
    fn get_only_lang() {
        let mut localised_value = LocalisedValue {
            localised_value: vec![
                ("en".parse().unwrap(), "en".to_owned()),
                ("en_GB".parse().unwrap(), "en_GB".to_owned()),
            ],
        };
        let value = localised_value.remove(&"en".parse().unwrap()).unwrap();
        assert_eq!(value, "en");
    }

    #[test]
    fn get_too_specific() {
        let mut localised_value = LocalisedValue {
            localised_value: vec![("en".parse().unwrap(), "en".to_owned())],
        };
        let value = localised_value.remove(&"en_GB".parse().unwrap()).unwrap();
        assert_eq!(value, "en");
    }

    #[test]
    fn get_precedence() {
        let mut localised_value = LocalisedValue {
            localised_value: vec![
                (Locale::default(), "def".to_owned()),
                ("sr_YU".parse().unwrap(), "sr_YU".to_owned()),
                ("sr@Latn".parse().unwrap(), "sr@Latn".to_owned()),
                ("sr".parse().unwrap(), "sr".to_owned()),
            ],
        };
        let value = localised_value
            .remove(&"sr_YU@Latn".parse().unwrap())
            .unwrap();
        assert_eq!(value, "sr_YU");
    }
}

fn parse_localised_desktop_group<'a, LineIter>(
    lines: &mut Peekable<LineIter>,
) -> Option<(String, LocalisedGroup)>
where
    LineIter: Iterator<Item = &'a str>,
{
    let header = if let Some(header) = find_desktop_header(lines) {
        header
    } else {
        error!("Could not parse header");
        return None;
    };

    let mut localised_group = LocalisedGroup::default();

    lines
        .peeking_take_while(|line| !line.trim().starts_with('['))
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| split_first('=', line))
        .filter(|&(_, value)| !value.is_empty())
        .for_each(|(key, value)| {
            let (key, locale) = parse_key(key);
            let localised_value = localised_group
                .group
                .entry(key.to_owned())
                .or_insert_with(|| LocalisedValue::default());
            localised_value.insert(locale, value.to_owned());
        });
    Some((header, localised_group))
}

fn find_desktop_header<'a, LineIter>(lines: &mut LineIter) -> Option<String>
where
    LineIter: Iterator<Item = &'a str>,
{
    while let Some(line) = lines.next() {
        if let Some(header) = parse_header(line) {
            if header.starts_with("Desktop ") {
                return Some(header);
            }
        }
    }
    None
}

#[cfg(test)]
mod parse_localised_desktop_group_tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn no_input() {
        let mut lines = "".lines().peekable();
        lines.next();
        let localised_group = parse_localised_desktop_group(&mut lines);
        assert_that(&localised_group).is_none();
    }

    #[test]
    fn no_group() {
        let localised_group = parse_localised_desktop_group(&mut "".lines().peekable());
        assert_that(&localised_group).is_none();
    }

    #[test]
    fn header_only() {
        let input = "[Desktop group header]";
        let localised_group = parse_localised_desktop_group(&mut input.lines().peekable());
        assert_that(&localised_group)
            .is_some()
            .is_equal_to(("Desktop group header".to_owned(), LocalisedGroup::default()));
    }

    #[test]
    fn single_group() {
        let input = "[Desktop group header]
        # Comment
        Key1=Value1
        Key1[en]=Value2
        Key2[C]=Value3";
        let localised_group = parse_localised_desktop_group(&mut input.lines().peekable());
        assert_that(&localised_group)
            .is_some()
            .map(|group| &group.0)
            .is_equal_to("Desktop group header".to_owned());
        assert_that(&localised_group)
            .is_some()
            .map(|group| &group.1.group)
            .is_equal_to(hashmap! {
                "Key1".to_owned() => LocalisedValue {
                    localised_value: vec![
                        (Locale::default(), "Value1".to_owned()),
                        ("en".parse::<Locale>().unwrap(), "Value2".to_owned()),
                    ]
                },
                "Key2".to_owned() => LocalisedValue {
                    localised_value: vec![
                        ("C".parse::<Locale>().unwrap(), "Value3".to_owned()),
                    ]
                }
            });
    }

    #[test]
    fn multiple_groups() {
        let input = "[Desktop Entry]
        # Comment
        Key1=Value1
        Key1[en]=Value2
        Key2[C]=Value3

        [Desktop Action group]
        # Top comment
        Key=Value
        # Middle comment
        Key=Overwritten Value
        # Bottom comment
        ";
        let mut lines = input.lines().peekable();

        let localised_group = parse_localised_desktop_group(&mut lines);

        assert_that(&localised_group)
            .is_some()
            .map(|group| &group.0)
            .is_equal_to("Desktop Entry".to_owned());
        assert_that(&localised_group)
            .is_some()
            .map(|group| &group.1.group)
            .is_equal_to(hashmap! {
                "Key1".to_owned() => LocalisedValue {
                    localised_value: vec![
                        (Locale::default(), "Value1".to_owned()),
                        ("en".parse().unwrap(), "Value2".to_owned()),
                    ]
                },
                "Key2".to_owned() => LocalisedValue {
                    localised_value: vec![
                        ("C".parse().unwrap(), "Value3".to_owned()),
                    ]
                },
            });

        let localised_group = parse_localised_desktop_group(&mut lines);

        assert_that(&localised_group)
            .is_some()
            .map(|group| &group.0)
            .is_equal_to("Desktop Action group".to_owned());
        assert_that(&localised_group)
            .is_some()
            .map(|group| &group.1.group)
            .is_equal_to(hashmap! {
                "Key".to_owned() => LocalisedValue {
                    localised_value: vec![
                        (Locale::default(), "Overwritten Value".to_owned()),
                    ]
                },
            });
    }
}

fn parse_header(line: &str) -> Option<String> {
    let mut chars = line.trim().chars();
    if chars.next() == Some('[') {
        Some(chars.take_while(|c| *c != ']').collect())
    } else {
        None
    }
}

#[cfg(test)]
mod parse_header_tests {
    use super::*;

    #[test]
    fn header() {
        assert_eq!(
            parse_header(&"[group header]".to_owned()),
            Some("group header".to_owned())
        );
    }

    #[test]
    fn not_header() {
        assert_eq!(parse_header(&"group header]".to_owned()), None);
    }
}

fn parse_key(line: &str) -> (&str, Locale) {
    line.rfind(']')
        .and_then(|j| line[0..j].rfind('[').map(|i| (i, j)))
        .and_then(|(i, j)| {
            if j - i > 1 {
                let (key, locale) = (&line[0..i], &line[i + 1..j]);
                locale.parse::<Locale>().ok().map(|locale| (key, locale))
            } else {
                Some((&line[0..i], Locale::default()))
            }
        })
        .unwrap_or_else(|| (line, Locale::default()))
}

#[cfg(test)]
mod parse_key_tests {
    use super::*;

    #[test]
    fn no_locale() {
        let (key, locale) = parse_key("Key");
        assert_eq!(key, "Key");
        assert_eq!(locale, Locale::default());
    }

    #[test]
    fn locale() {
        let (key, locale) = parse_key("Key[lang]");
        assert_eq!(key, "Key");
        assert_eq!(locale, "lang".parse().unwrap());
    }

    #[test]
    fn empty_locale() {
        let (key, locale) = parse_key("Key[]");
        assert_eq!(key, "Key");
        assert_eq!(locale, Locale::default());
    }
}

fn split_first(delim: char, s: &str) -> Option<(&str, &str)> {
    s.find(delim)
        .map(|i| s.split_at(i))
        .map(|(name, value)| (name.trim(), value[1..value.len()].trim()))
}

#[cfg(test)]
mod split_tests {
    use super::split_first;

    #[test]
    fn split_match() {
        assert_eq!(split_first('b', "abc"), Some(("a", "c")))
    }

    #[test]
    fn no_match() {
        assert_eq!(split_first('-', "abc"), None)
    }
}
