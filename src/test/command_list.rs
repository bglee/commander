#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_from_env() {
        let regex = CommandList::regex_from_env().unwrap();
        assert!(regex.is_match("test;match"));
        assert!(regex.is_match("another;match"));
        assert!(!regex.is_match("testnomatch"));
        assert!(!regex.is_match("anothernomatch"));
        assert!(regex.is_match(" 1703434517:0;cargo build"))
    }
}
