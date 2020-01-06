use crate::error::Error::{InvalidIssueId, InvalidTimeLogHours};
use crate::result::Result;

pub fn parse_hours(input: &str) -> Result<f32> {
    let suffixes: &[_] = &[' ', 'h', 'H'];
    let trimmed_input = input.trim_start().trim_end_matches(suffixes);
    trimmed_input
        .parse()
        .map_err(|_| InvalidTimeLogHours(input.to_owned()))
}

pub fn parse_issue(input: &str) -> Result<i32> {
    let prefixes: &[_] = &[' ', '#'];
    let trimmed_input = input.trim_start_matches(prefixes).trim_end();
    trimmed_input
        .parse()
        .map_err(|_| InvalidIssueId(input.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hours_integer_number() {
        assert!((parse_hours("5").unwrap() - 5.0).abs() < 0.001);
    }

    #[test]
    fn parse_hours_integer_number_and_space() {
        assert!((parse_hours("  5 ").unwrap() - 5.0).abs() < 0.001);
    }

    #[test]
    fn parse_hours_fraction_number() {
        assert!((parse_hours("5.5").unwrap() - 5.5).abs() < 0.001);
    }

    #[test]
    fn parse_hours_number_with_h() {
        assert!((parse_hours("6h").unwrap() - 6.0).abs() < 0.001);
    }

    #[test]
    fn parse_hours_integer_with_h_and_space() {
        assert!((parse_hours(" 6  h ").unwrap() - 6.0).abs() < 0.001);
    }

    #[test]
    fn parse_hours_faction_with_h_and_space() {
        assert!((parse_hours(" 6.7 h ").unwrap() - 6.7).abs() < 0.001);
    }

    #[test]
    fn parse_hours_integer_with_capital_h() {
        assert!((parse_hours("7H").unwrap() - 7.0).abs() < 0.001);
    }

    #[test]
    fn parse_hours_fraction_with_capital_h() {
        assert!((parse_hours("7.8 H").unwrap() - 7.8).abs() < 0.001);
    }

    #[test]
    fn parse_hours_integer_with_capital_h_and_space() {
        assert!((parse_hours(" 7 H  ").unwrap() - 7.0).abs() < 0.001);
    }

    #[test]
    fn parse_hours_empty() {
        assert!(parse_hours("").is_err());
    }

    #[test]
    fn parse_hours_just_suffix() {
        assert!(parse_hours("h").is_err());
    }

    #[test]
    fn parse_issue_id() {
        assert_eq!(parse_issue("12345").unwrap(), 12345);
    }

    #[test]
    fn parse_issueid_and_space() {
        assert_eq!(parse_issue("  12345 ").unwrap(), 12345);
    }

    #[test]
    fn parse_issue_id_with_hash_prefix() {
        assert_eq!(parse_issue("#12345").unwrap(), 12345);
    }

    #[test]
    fn parse_issue_id_with_hash_prefix_and_space() {
        assert_eq!(parse_issue(" #  12345 ").unwrap(), 12345);
    }
}
