use std::str::FromStr;
use lunch::StdResult;
use lunch::errors::*;

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Locale {
    lang: String,
    country: Option<String>,
    encoding: Option<String>,
    modifier: Option<String>,
}

impl FromStr for Locale {
    type Err = Error;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        let s = s.trim();

        let (modifier, len) = find_after(s, '@');
        let (encoding, len) = find_after(&s[0..len], '.');
        let (country, len) = find_after(&s[0..len], '_');
        let lang = filter_empty(&s[0..len])
            .ok_or_else::<Self::Err, _>(|| ErrorKind::InvalidLocale(s.to_owned()).into())?;

        Ok(Locale {
            lang: lang.to_string(),
            country: country.map(|s| s.to_string()),
            encoding: encoding.map(|s| s.to_string()),
            modifier: modifier.map(|s| s.to_string()),
        })
    }
}

fn find_after(s: &str, after_pat: char) -> (Option<&str>, usize) {
    let pos = s.rfind(after_pat);
    let m = pos.map(|pos| s[pos + 1..s.len()].trim());
    (m.and_then(filter_empty), pos.unwrap_or_else(|| s.len()))
}

fn filter_empty(s: &str) -> Option<&str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
pub enum MatchLevel {
    Lang,
    LangModifier,
    LangCountry,
    LangCountryModifier,
}

impl Locale {
    pub fn from_env() -> Result<Locale> {
        use std::env::var;
        for var_name in &["LC_ALL", "LC_MESSAGES", "LANG"] {
            match var(var_name) {
                Ok(locale) => {
                    debug!("Found locale '{}' in ${}", locale, var_name);
                    return locale.parse();
                }
                Err(err) => {
                    debug!("Error reading env var ${}: {}", var_name, err);
                }
            }
        }
        Ok(Locale::default())
    }

    pub fn match_level(&self, b: &Self) -> Option<MatchLevel> {
        use self::MatchLevel::*;
        if (!self.modifier.is_some() && b.modifier.is_some())
            || (!self.country.is_some() && b.country.is_some())
        {
            None
        } else if self.lang == b.lang && self.country.is_some() && self.country == b.country
            && self.modifier.is_some() && self.modifier == b.modifier
        {
            Some(LangCountryModifier)
        } else if self.lang == b.lang && self.country.is_some() && self.country == b.country
            && b.modifier.is_none()
        {
            Some(LangCountry)
        } else if self.lang == b.lang && self.modifier.is_some() && self.modifier == b.modifier
            && b.country.is_none()
        {
            Some(LangModifier)
        } else if self.lang == b.lang {
            Some(Lang)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod from_str_tests {
    use super::*;

    #[test]
    fn from_str() {
        let s = "en_GB.UTF-8@mod";
        let locale: Locale = s.parse().unwrap();
        assert_eq!(locale.lang, "en");
        assert_eq!(locale.country, Some("GB".to_string()));
        assert_eq!(locale.encoding, Some("UTF-8".to_string()));
        assert_eq!(locale.modifier, Some("mod".to_string()));
    }

    #[test]
    fn from_str_no_modifier() {
        let s = "en_GB.UTF-8";
        let locale: Locale = s.parse().unwrap();
        assert_eq!(locale.lang, "en");
        assert_eq!(locale.country, Some("GB".to_string()));
        assert_eq!(locale.encoding, Some("UTF-8".to_string()));
        assert_eq!(locale.modifier, None);
    }

    #[test]
    fn from_str_modifier_no_country() {
        let s = "en.UTF-8@mod";
        let locale: Locale = s.parse().unwrap();
        assert_eq!(locale.lang, "en");
        assert_eq!(locale.country, None);
        assert_eq!(locale.encoding, Some("UTF-8".to_string()));
        assert_eq!(locale.modifier, Some("mod".to_string()));
    }

    #[test]
    fn from_str_modifier_no_country_no_encoding() {
        let s = "en@mod";
        let locale: Locale = s.parse().unwrap();
        assert_eq!(locale.lang, "en");
        assert_eq!(locale.country, None);
        assert_eq!(locale.encoding, None);
        assert_eq!(locale.modifier, Some("mod".to_string()));
    }

    #[test]
    fn from_str_no_country() {
        let s = "en.UTF-8";
        let locale: Locale = s.parse().unwrap();
        assert_eq!(locale.lang, "en");
        assert_eq!(locale.country, None);
        assert_eq!(locale.encoding, Some("UTF-8".to_string()));
        assert_eq!(locale.modifier, None);
    }

    #[test]
    #[should_panic]
    fn from_str_no_lang() {
        let s = "_GB.UTF-8";
        s.parse::<Locale>().unwrap();
    }
}

#[cfg(test)]
mod match_level_tests {
    use super::*;

    #[test]
    fn match_level_lang_country_mod() {
        let a: Locale = "en_GB@mod".parse().unwrap();
        {
            let b: Locale = "en_GB@mod".parse().unwrap();

            let res = a.match_level(&b);
            assert_eq!(res, Some(MatchLevel::LangCountryModifier));
        }
        {
            let b: Locale = "en_GB".parse().unwrap();

            let res = a.match_level(&b);
            assert_eq!(res, Some(MatchLevel::LangCountry));
        }
        {
            let b: Locale = "en@mod".parse().unwrap();

            let res = a.match_level(&b);
            assert_eq!(res, Some(MatchLevel::LangModifier));
        }
        {
            let b: Locale = "en".parse().unwrap();

            let res = a.match_level(&b);
            assert_eq!(res, Some(MatchLevel::Lang));
        }
    }

    #[test]
    fn match_level_lang_country() {
        let a: Locale = "en_GB.UTF-8".parse().unwrap();
        {
            let b: Locale = "en_GB".parse().unwrap();

            let res = a.match_level(&b);
            assert_eq!(res, Some(MatchLevel::LangCountry));
        }
        {
            let b: Locale = "en".parse().unwrap();

            let res = a.match_level(&b);
            assert_eq!(res, Some(MatchLevel::Lang));
        }
    }

    #[test]
    fn match_level_lang_mod() {
        let a: Locale = "en@mod".parse().unwrap();
        {
            let b: Locale = "en@mod".parse().unwrap();

            let res = a.match_level(&b);
            assert_eq!(res, Some(MatchLevel::LangModifier));
        }
        {
            let b: Locale = "en".parse().unwrap();

            let res = a.match_level(&b);
            assert_eq!(res, Some(MatchLevel::Lang));
        }
    }

    #[test]
    fn match_level_none_mod_reverse() {
        let a: Locale = "en".parse().unwrap();
        let b: Locale = "en_GB@mod".parse().unwrap();

        let res = a.match_level(&b);
        assert_eq!(res, None);
    }

    #[test]
    fn match_level_ord() {
        use std::cmp::Ordering;
        let ord = MatchLevel::LangCountryModifier.cmp(&MatchLevel::LangCountry);
        assert_eq!(Ordering::Greater, ord);
    }

    #[test]
    fn spec_example() {
        let a: Locale = "sr_YU@Latn".parse().unwrap();
        {
            let b: Locale = "sr_YU".parse().unwrap();
            assert_eq!(Some(MatchLevel::LangCountry), a.match_level(&b));
        }
        {
            let b: Locale = "sr@Latn".parse().unwrap();
            assert_eq!(Some(MatchLevel::LangModifier), a.match_level(&b));
        }
        {
            let b: Locale = "sr".parse().unwrap();
            assert_eq!(Some(MatchLevel::Lang), a.match_level(&b));
        }
    }
}
