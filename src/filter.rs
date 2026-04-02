use nucleo_matcher::{
    Config, Matcher,Utf32Str
};

pub fn fuzzy_filter<'a>(term: &str, list: &[&'a str]) -> Vec<&'a str> {
    let mut matcher = Matcher::new(Config::DEFAULT);
    let mut needle_buf = Vec::new();
    let mut haystack_buf =  Vec::new();

    let needle = Utf32Str::new(term, &mut needle_buf);

    let mut ordered_scored: Vec<(&str, u32)> = Vec::new();

    for &item in list {
       let haystack = Utf32Str::new(item, &mut haystack_buf);
       if let Some(score) = matcher.fuzzy_match(haystack, needle){
         ordered_scored.push((item, score.into()));
       }
    }

    ordered_scored.sort_by(|a,b| b.1.cmp(&a.1));

    return ordered_scored.iter().map(|f| f.0).collect();
}
