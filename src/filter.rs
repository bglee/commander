use nucleo_matcher::{
    pattern::{CaseMatching, Pattern},
    Config, Matcher,
};

pub fn fuzzy_filter<'a>(term: &'a str, list: &Vec<&'a str>) -> Vec<&'a str> {
    let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
    let matched = Pattern::parse(term, CaseMatching::Ignore).match_list(list, &mut matcher);
    return matched.iter().map(|f| f.0.to_owned()).collect();
}
