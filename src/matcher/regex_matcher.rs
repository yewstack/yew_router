use super::Captures;
use super::Matcher;
use crate::matcher::MatcherProvider;
use regex::Regex;

impl MatcherProvider for Regex {
    fn match_route_string<'a, 'b: 'a>(&'b self, route_string: &'a str) -> Option<Captures<'a>> {
        if self.is_match(route_string) {
            let names: Vec<&str> = self
                .capture_names()
                .filter_map(std::convert::identity)
                .collect();
            let mut matches: Captures = Captures::new();
            self.captures_iter(route_string).for_each(|cap| {
                names.iter().for_each(|name| {
                    matches.insert(name, cap[*name].to_string());
                });
            });
            Some(matches)
        } else {
            None
        }
    }
}

impl From<Regex> for Matcher {
    fn from(value: Regex) -> Self {
        Matcher::RegexMatcher(value)
    }
}
