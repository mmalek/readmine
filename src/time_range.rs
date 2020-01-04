use crate::error::Error;
use crate::result::Result;
use chrono::{Datelike, Duration, Local, NaiveDate};

#[derive(Debug, PartialEq)]
pub struct TimeRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

const RANGE_SEPARATOR: &str = "..";

impl TimeRange {
    pub fn parse(input: &str) -> Result<TimeRange> {
        let range = TimePointRange::parse(input)?;
        let from = range.from.to_lower_bound()?;
        let to = range.to.to_upper_bound()?;
        Ok(TimeRange { from, to })
    }
}

#[derive(Clone, Debug, PartialEq)]
enum TimePoint {
    Date(NaiveDate),
    Week(i32),
    Month(i32),
}

impl TimePoint {
    fn to_lower_bound(&self) -> Result<NaiveDate> {
        self.to_lower_bound_with_date(Local::today().naive_local())
    }

    fn to_lower_bound_with_date(&self, today: NaiveDate) -> Result<NaiveDate> {
        match *self {
            TimePoint::Date(date) => Ok(date),
            TimePoint::Week(offset) => Ok(today
                - Duration::days(today.weekday().num_days_from_monday() as i64)
                + Duration::weeks(offset as i64)),
            TimePoint::Month(offset) => {
                let year_offset = offset / 12;
                let month_offset = offset % 12;
                let date = today
                    .with_year(today.year() + year_offset)
                    .ok_or(Error::InvalidMonthOffset(offset))?;
                let date = date
                    .with_month((date.month() as i32 + month_offset) as u32)
                    .ok_or(Error::InvalidMonthOffset(offset))?;
                Ok(date
                    .with_day(1)
                    .expect("Cannot use '1' as day of month (?)"))
            }
        }
    }

    fn to_upper_bound(&self) -> Result<NaiveDate> {
        self.to_upper_bound_with_date(Local::today().naive_local())
    }

    fn to_upper_bound_with_date(&self, today: NaiveDate) -> Result<NaiveDate> {
        match *self {
            TimePoint::Date(date) => Ok(date),
            TimePoint::Week(offset) => Ok(today
                + Duration::days(6 - today.weekday().num_days_from_monday() as i64)
                + Duration::weeks(offset as i64)),
            TimePoint::Month(offset) => {
                let year_offset = offset / 12;
                let month_offset = offset % 12;
                let date = today
                    .with_year(today.year() + year_offset)
                    .ok_or(Error::InvalidMonthOffset(offset))?;
                let date = date
                    .with_month((date.month() as i32 + month_offset) as u32)
                    .ok_or(Error::InvalidMonthOffset(offset))?;
                let date = date
                    .with_day(1)
                    .expect("Cannot use '1' as day of month (?)");
                if date.month() < 12 {
                    date.with_month(date.month() + 1)
                        .map(|date| date - Duration::days(1))
                        .ok_or(Error::InvalidMonthOffset(offset))
                } else {
                    Ok(date
                        .with_month(12)
                        .expect("Cannot use '12' as a month number (?)")
                        .with_day(31)
                        .expect("Cannot use '31' as day number of December (?)"))
                }
            }
        }
    }
}

fn parse_week_month_offset(input: &str) -> Option<i32> {
    if input.is_empty() {
        Some(0)
    } else if input.chars().nth(0) == Some('+') && input.len() > 1 {
        input[1..].parse().ok()
    } else if input.chars().nth(0) == Some('-') && input.len() > 1 {
        input[1..].parse().ok().map(|offset: i32| -offset)
    } else {
        None
    }
}

const MONTH_PLACEHOLDER: &str = "month";
const WEEK_PLACEHOLDER: &str = "week";

fn parse_time_point(input: &str) -> Result<TimePoint> {
    if input.starts_with(MONTH_PLACEHOLDER) {
        parse_week_month_offset(&input[MONTH_PLACEHOLDER.len()..])
            .ok_or_else(|| Error::InvalidTimeRangeFormat(input.to_owned()))
            .map(TimePoint::Month)
    } else if input.starts_with(WEEK_PLACEHOLDER) {
        parse_week_month_offset(&input[WEEK_PLACEHOLDER.len()..])
            .ok_or_else(|| Error::InvalidTimeRangeFormat(input.to_owned()))
            .map(TimePoint::Week)
    } else {
        NaiveDate::parse_from_str(input, "%Y-%m-%d")
            .map_err(|_| Error::InvalidTimeRangeFormat(input.to_owned()))
            .map(TimePoint::Date)
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TimePointRange {
    from: TimePoint,
    to: TimePoint,
}

impl TimePointRange {
    fn parse(input: &str) -> Result<TimePointRange> {
        let sep_pos = input.find(RANGE_SEPARATOR);
        let from_end = sep_pos.unwrap_or_else(|| input.len());

        let from = parse_time_point(&input[..from_end])?;

        let to = if let Some(sep_pos) = sep_pos {
            let to_pos = sep_pos + RANGE_SEPARATOR.len();
            parse_time_point(&input[to_pos..])?
        } else {
            from.clone()
        };

        Ok(TimePointRange { from, to })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_time_point_current_month() {
        assert_eq!(parse_time_point("month").unwrap(), TimePoint::Month(0));
    }

    #[test]
    fn parse_time_point_next_month() {
        assert_eq!(parse_time_point("month+1").unwrap(), TimePoint::Month(1));
    }

    #[test]
    fn parse_time_point_last_month() {
        assert_eq!(parse_time_point("month-1").unwrap(), TimePoint::Month(-1));
    }

    #[test]
    fn parse_time_point_current_week() {
        assert_eq!(parse_time_point("week").unwrap(), TimePoint::Week(0));
    }

    #[test]
    fn parse_time_point_iso_date() {
        assert_eq!(
            parse_time_point("2019-01-23").unwrap(),
            TimePoint::Date(NaiveDate::from_ymd(2019, 01, 23))
        );
    }

    #[test]
    fn empty_string_range() {
        assert!(TimeRange::parse("").is_err());
    }

    #[test]
    fn week_range() {
        assert_eq!(
            TimePointRange::parse("week-1..week+1").unwrap(),
            TimePointRange {
                from: TimePoint::Week(-1),
                to: TimePoint::Week(1),
            }
        );
    }

    #[test]
    fn iso_range() {
        assert_eq!(
            TimePointRange::parse("2019-01-23..2019-05-09").unwrap(),
            TimePointRange {
                from: TimePoint::Date(NaiveDate::from_ymd(2019, 01, 23)),
                to: TimePoint::Date(NaiveDate::from_ymd(2019, 05, 09)),
            }
        );
    }

    #[test]
    fn mixed_range() {
        assert_eq!(
            TimePointRange::parse("2019-01-23..month+3").unwrap(),
            TimePointRange {
                from: TimePoint::Date(NaiveDate::from_ymd(2019, 01, 23)),
                to: TimePoint::Month(3),
            }
        );
    }

    #[test]
    fn date_to_lower_bound() {
        let input_date = NaiveDate::from_ymd(2019, 01, 23);
        let today = NaiveDate::from_ymd(2019, 8, 24);
        assert_eq!(
            TimePoint::Date(input_date)
                .to_lower_bound_with_date(today)
                .unwrap(),
            NaiveDate::from_ymd(2019, 1, 23)
        );
    }

    #[test]
    fn date_to_upper_bound() {
        let input_date = NaiveDate::from_ymd(2019, 01, 23);
        let today = NaiveDate::from_ymd(2019, 8, 24);
        assert_eq!(
            TimePoint::Date(input_date)
                .to_upper_bound_with_date(today)
                .unwrap(),
            NaiveDate::from_ymd(2019, 1, 23)
        );
    }

    #[test]
    fn current_week_to_lower_bound() {
        assert_eq!(
            TimePoint::Week(0)
                .to_lower_bound_with_date(NaiveDate::from_ymd(2019, 8, 24))
                .unwrap(),
            NaiveDate::from_ymd(2019, 8, 19)
        );
    }

    #[test]
    fn current_week_monday_to_lower_bound() {
        assert_eq!(
            TimePoint::Week(0)
                .to_lower_bound_with_date(NaiveDate::from_ymd(2019, 8, 19))
                .unwrap(),
            NaiveDate::from_ymd(2019, 8, 19)
        );
    }

    #[test]
    fn current_week_sunday_to_lower_bound() {
        assert_eq!(
            TimePoint::Week(0)
                .to_lower_bound_with_date(NaiveDate::from_ymd(2019, 8, 25))
                .unwrap(),
            NaiveDate::from_ymd(2019, 8, 19)
        );
    }

    #[test]
    fn current_week_to_upper_bound() {
        assert_eq!(
            TimePoint::Week(0)
                .to_upper_bound_with_date(NaiveDate::from_ymd(2019, 8, 24))
                .unwrap(),
            NaiveDate::from_ymd(2019, 8, 25)
        );
    }

    #[test]
    fn current_week_monday_to_upper_bound() {
        assert_eq!(
            TimePoint::Week(0)
                .to_upper_bound_with_date(NaiveDate::from_ymd(2019, 8, 19))
                .unwrap(),
            NaiveDate::from_ymd(2019, 8, 25)
        );
    }

    #[test]
    fn current_week_sunday_to_upper_bound() {
        assert_eq!(
            TimePoint::Week(0)
                .to_upper_bound_with_date(NaiveDate::from_ymd(2019, 8, 25))
                .unwrap(),
            NaiveDate::from_ymd(2019, 8, 25)
        );
    }

    #[test]
    fn last_week_to_lower_bound() {
        assert_eq!(
            TimePoint::Week(-1)
                .to_lower_bound_with_date(NaiveDate::from_ymd(2019, 8, 24))
                .unwrap(),
            NaiveDate::from_ymd(2019, 8, 12)
        );
    }

    #[test]
    fn last_week_to_upper_bound() {
        assert_eq!(
            TimePoint::Week(-1)
                .to_upper_bound_with_date(NaiveDate::from_ymd(2019, 8, 24))
                .unwrap(),
            NaiveDate::from_ymd(2019, 8, 18)
        );
    }

    #[test]
    fn next_week_to_lower_bound() {
        assert_eq!(
            TimePoint::Week(1)
                .to_lower_bound_with_date(NaiveDate::from_ymd(2019, 8, 24))
                .unwrap(),
            NaiveDate::from_ymd(2019, 8, 26)
        );
    }

    #[test]
    fn next_week_to_upper_bound() {
        assert_eq!(
            TimePoint::Week(1)
                .to_upper_bound_with_date(NaiveDate::from_ymd(2019, 8, 24))
                .unwrap(),
            NaiveDate::from_ymd(2019, 9, 1)
        );
    }

    #[test]
    fn current_month_to_lower_bound() {
        assert_eq!(
            TimePoint::Month(0)
                .to_lower_bound_with_date(NaiveDate::from_ymd(2019, 8, 24))
                .unwrap(),
            NaiveDate::from_ymd(2019, 8, 1)
        );
    }

    #[test]
    fn current_month_day_1_to_lower_bound() {
        assert_eq!(
            TimePoint::Month(0)
                .to_lower_bound_with_date(NaiveDate::from_ymd(2019, 8, 1))
                .unwrap(),
            NaiveDate::from_ymd(2019, 8, 1)
        );
    }

    #[test]
    fn current_month_day_31_to_lower_bound() {
        assert_eq!(
            TimePoint::Month(0)
                .to_lower_bound_with_date(NaiveDate::from_ymd(2019, 8, 31))
                .unwrap(),
            NaiveDate::from_ymd(2019, 8, 1)
        );
    }

    #[test]
    fn current_month_to_upper_bound() {
        assert_eq!(
            TimePoint::Month(0)
                .to_upper_bound_with_date(NaiveDate::from_ymd(2019, 8, 24))
                .unwrap(),
            NaiveDate::from_ymd(2019, 8, 31)
        );
    }

    #[test]
    fn current_month_day_1_to_upper_bound() {
        assert_eq!(
            TimePoint::Month(0)
                .to_upper_bound_with_date(NaiveDate::from_ymd(2019, 8, 1))
                .unwrap(),
            NaiveDate::from_ymd(2019, 8, 31)
        );
    }

    #[test]
    fn current_month_day_31_to_upper_bound() {
        assert_eq!(
            TimePoint::Month(0)
                .to_upper_bound_with_date(NaiveDate::from_ymd(2019, 8, 31))
                .unwrap(),
            NaiveDate::from_ymd(2019, 8, 31)
        );
    }
}
