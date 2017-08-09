use std::collections::HashMap;

use locale::Locale;
use errors::*;
use desktop::iteratorext::IteratorExt;

type Group = HashMap<String, String>;

pub fn parse_group<LineIter>(input: LineIter, group_name: &str, locale: &Locale) -> Result<Group>
where
    LineIter: Iterator<Item = Result<String>>,
{
    let localised_group = parse_localised_group(input, group_name);
    localised_group.map(|localised_group| localised_group.resolve_to_locale(locale))
}

#[cfg(test)]
mod parse_group_tests {
    use super::*;
    use desktop::locale::Locale;

    #[test]
    #[should_panic]
    fn error() {
        let lines: Vec<Result<String>> = vec![Ok("[Group]".to_owned()), Err("error".into())];
        let group = parse_group(lines.into_iter(), "Group", &Locale::default());
        group.unwrap();
    }

    #[test]
    fn parse_group_default_locale() {
        let input = "[group header]
        # Comment
        Key1=Value1
        Key1[en]=Value2
        Key2[C]=Value3

        [Another Group]
        # Top comment
        Key=Value
        # Middle comment
        Key=Overwritten Value
        # Bottom comment
        ";
        let lines = input.lines().map(|line| Ok(line.to_owned()));
        let group = parse_group(lines, "Another Group", &Locale::default());
        assert_eq!(group.unwrap(), hashmap!{
            "Key".to_owned() => "Overwritten Value".to_owned(),
        });
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
                    localised_value.remove(locale).or_else(|| {
                        localised_value.remove(&Locale::default())
                    }),
                )
            })
            .filter_map(|(key, value)| value.map(|value| (key, value)))
            .collect()
    }
}

#[cfg(test)]
mod localised_group_tests {
    use desktop::locale::*;
    use super::{LocalisedGroup, LocalisedValue};

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
        idx.map(|idx| self.localised_value.remove(idx)).map(|(_,
          value)| {
            value
        })
    }

    fn get_idx(&self, locale: &Locale) -> Option<usize> {
        self.localised_value
            .iter()
            .enumerate()
            .map(|(idx, &(ref key, _))| (idx, key))
            .max_by_key(|&(_, locale_key)| locale.match_level(locale_key))
            .and_then(|(idx, locale_key)| {
                locale.match_level(locale_key).and(Some(idx))
            })
    }
}

#[cfg(test)]
mod localised_value_tests {
    use desktop::locale::Locale;
    use super::LocalisedValue;

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
        let mut localised_value =
            LocalisedValue { localised_value: vec![("en".parse().unwrap(), "en".to_owned())] };
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
        let mut localised_value =
            LocalisedValue { localised_value: vec![("en".parse().unwrap(), "en".to_owned())] };
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
        let value = localised_value.remove(&"sr_YU@Latn".parse().unwrap()).unwrap();
        assert_eq!(value, "sr_YU");
    }
}

fn parse_localised_group<LineIter>(input: LineIter, group_name: &str) -> Result<LocalisedGroup>
where
    LineIter: Iterator<Item = Result<String>>,
{
    let header_pred = {
        let mut header = "[".to_owned();
        header.push_str(group_name.trim());
        header.push(']');

        move |line: &String| !line.trim().starts_with(&header)
    };
    let lines: Vec<(String, String)> = input
        .map_result(|line| line.trim().to_owned())
        .skip_while_result(header_pred)
        .skip(1)
        .filter_result(|line| !line.is_empty() && !line.starts_with('#'))
        .take_while_result(|line| !line.starts_with('['))
        .filter_map(|line| match line {
            Ok(line) => split_to_owned('=', &line).map(Ok),
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<_>>()?;
    let mut localised_group = LocalisedGroup::default();
    for (key, value) in lines {
        let (key, locale) = parse_key(&key);
        let mut localised_value = localised_group.group.entry(key.to_owned()).or_insert_with(
            || {
                LocalisedValue::default()
            },
        );
        localised_value.insert(locale, value);
    }
    Ok(localised_group)
}

#[cfg(test)]
mod parse_localised_group_tests {
    use super::*;

    #[test]
    fn header_only() {
        let input = "[group header]";
        let lines = input.lines().map(|line| Ok(line.to_owned()));
        let localised_group = parse_localised_group(lines, "group header");
        assert_eq!(localised_group.unwrap(), LocalisedGroup::default());
    }

    #[test]
    fn single_group() {
        let input = "[group header]
        # Comment
        Key1=Value1
        Key1[en]=Value2
        Key2[C]=Value3";
        let localised_group = parse_localised_group(
            input.lines().map(|line| Ok(line.to_owned())),
            "group header",
        );
        assert_eq!(
            localised_group.unwrap().group,
            hashmap! {
                "Key1".to_owned() => LocalisedValue {
                    localised_value: vec!{
                        (Locale::default(), "Value1".to_owned()),
                        ("en".parse::<Locale>().unwrap(), "Value2".to_owned()),
                    }
                },
                "Key2".to_owned() => LocalisedValue {
                    localised_value: vec!{
                        ("C".parse::<Locale>().unwrap(), "Value3".to_owned()),
                    }
                }
            }
        );
    }

    #[test]
    fn multiple_groups() {
        let input = "[group header]
        # Comment
        Key1=Value1
        Key1[en]=Value2
        Key2[C]=Value3

        [Another Group]
        # Top comment
        Key=Value
        # Middle comment
        Key=Overwritten Value
        # Bottom comment
        ";
        let lines = input.lines().map(|line| Ok(line.to_owned()));
        let localised_group = parse_localised_group(lines, "Another Group");
        assert_eq!(
            localised_group.unwrap().group,
            hashmap! {
                "Key".to_owned() => LocalisedValue {
                    localised_value: vec![
                        (Locale::default(), "Overwritten Value".to_owned()),
                    ]
                },
            }
        );
    }
}

fn parse_key(line: &str) -> (&str, Locale) {
    line.rfind(']')
        .and_then(|j| line[0..j].rfind('[').map(|i| (i, j)))
        .and_then(|(i, j)| if j - i > 1 {
            let (key, locale) = (&line[0..i], &line[i + 1..j]);
            locale.parse::<Locale>().ok().map(|locale| (key, locale))
        } else {
            Some((&line[0..i], Locale::default()))
        })
        .unwrap_or_else(|| (line, Locale::default()))
}

#[cfg(test)]
mod parse_key_tests {
    use super::parse_key;
    use desktop::locale::Locale;

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

fn split(delim: char, s: &str) -> Option<(&str, &str)> {
    s.find(delim).map(|i| s.split_at(i)).map(|(name, value)| {
        (name.trim(), value[1..value.len()].trim())
    })
}

fn split_to_owned(delim: char, s: &str) -> Option<(String, String)> {
    if let Some((left, right)) = split(delim, s) {
        Some((left.to_owned(), right.to_owned()))
    } else {
        None
    }
}

#[cfg(test)]
mod split_tests {
    use super::split;

    #[test]
    fn split_match() {
        assert_eq!(split('b', "abc"), Some(("a", "c")))
    }

    #[test]
    fn no_match() {
        assert_eq!(split('-', "abc"), None)
    }
}
