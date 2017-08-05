use std::collections::HashMap;

use locale::Locale;
use errors::*;
use desktop::iteratorext::IteratorExt;

type Group = HashMap<String, String>;

pub fn parse_desktop_entry_group<LineIter>(input: LineIter, locale: &Locale) -> Result<Group>
where
    LineIter: Iterator<Item = Result<String>>,
{
    let localised_group = parse_group(input, "Desktop Entry");
    localised_group.map(|localised_group| {
        resolve_localised_group(localised_group, locale)
    })
}

fn resolve_localised_group(localised_group: LocalisedGroup, locale: &Locale) -> Group {
    localised_group
        .into_iter()
        .map(|(key, localised_value)| {
            (
                key,
                localised_value.get(locale).or_else(|| {
                    localised_value.get(&Locale::default())
                }),
            )
        })
        .filter_map(|(key, value)| value.map(|value| (key, value)))
        .collect()
}

type LocalisedGroup = HashMap<String, LocalisedValue>;

struct LocalisedValue(Vec<(Locale, String)>);

impl LocalisedValue {
    fn get(&self, locale: &Locale) -> Option<String> {
        let &LocalisedValue(ref localised_value) = self;
        localised_value
            .iter()
            .map(|&(ref locale_key, ref value)| {
                (locale_key.match_level(locale), value)
            })
            .max_by_key(|&(match_level, _)| match_level)
            .map(|(_, value)| value.to_owned())
    }
}

fn parse_group<LineIter>(input: LineIter, section_name: &str) -> Result<LocalisedGroup>
where
    LineIter: Iterator<Item = Result<String>>,
{
    let lines: Vec<(String, String)> = input
        .skip_while_result(matches_group_header_not_named(section_name))
        .skip(1)
        .filter_result(|line| {
            !line.trim().is_empty() && !line.trim().starts_with('#')
        })
        .take_while_result(|line| !line.trim().starts_with('['))
        .filter_map(|line| match line {
            Ok(line) => split_to_owned('=', &line).map(Ok),
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<_>>()?;
    let mut group = LocalisedGroup::default();
    for (key, value) in lines {
        let (key, locale) = parse_key(&key);
        let &mut LocalisedValue(ref mut localised_value) =
            group.entry(key.to_owned()).or_insert_with(
                || LocalisedValue(vec![]),
            );
        localised_value.push((locale, value));
    }
    Ok(group)
}

fn parse_key(line: &str) -> (&str, Locale) {
    line.find('[')
        .and_then(|i| line[i + 1..line.len()].find(']').map(|j| (i, j)))
        .map(|(i, j)| (&line[0..i], &line[i + 1..j + i + 1]))
        .and_then(|(key, locale)| {
            locale.parse::<Locale>().ok().map(|locale| (key, locale))
        })
        .unwrap_or_else(|| (line, Locale::default()))
}

#[cfg(test)]
mod test {
    #[cfg(test)]
    mod parse_key {
        use desktop::parse::parse_key;
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
    }
}

fn matches_group_header_not_named(group_name: &str) -> impl FnMut(&String) -> bool {
    let mut header = "[".to_owned();
    header.push_str(group_name.trim());
    header.push(']');

    move |line| !line.trim().starts_with(&header)
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
