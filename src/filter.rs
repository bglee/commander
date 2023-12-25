use nucleo_matcher::{
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
    Config, Matcher,
};

pub fn fuzzy_filter<'a>(term: &'a str, list: &Vec<&'a str>) -> Vec<&'a str> {
    let mut matcher = Matcher::new(Config::DEFAULT);
    let matched = Atom::new(
        term,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Substring,
        true,
    )
    .match_list(list, &mut matcher);
    return matched.iter().map(|f| f.0.to_owned()).collect();
}
