use std::str::FromStr;
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime};

use crate::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateRangeExtent
{
    Year(i32),
    YearMonth(i32, u32),
    YearMonthDay(i32, u32, u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateRange
{
    pub(crate) start: DateRangeExtent,
    pub(crate) end: DateRangeExtent,
}

impl DateRange
{
    pub fn first_date(&self) -> NaiveDate
    {
        self.start.first_date_opt().unwrap()
    }

    pub fn last_date(&self) -> NaiveDate
    {
        self.end.last_date_opt().unwrap()
    }
}

impl DateRangeExtent
{
    pub(crate) fn first_date_opt(&self) -> Option<NaiveDate>
    {
        match self
        {
            Self::Year(year) => NaiveDate::from_ymd_opt(*year, 1, 1),
            Self::YearMonth(year, month) => NaiveDate::from_ymd_opt(*year, *month, 1),
            Self::YearMonthDay(year, month, day) => NaiveDate::from_ymd_opt(*year, *month, *day),
        }
    }

    pub(crate) fn last_date_opt(&self) -> Option<NaiveDate>
    {
        match self
        {
            Self::Year(year) =>
            {
                // Last day of the year is always Dec 31

                NaiveDate::from_ymd_opt(*year, 12, 31)
            },
            Self::YearMonth(year, month) =>
            {
                // Predecessor of the first day of the next month

                let plus_1 = if *month == 12
                {
                    NaiveDate::from_ymd_opt(*year + 1, 1, 1)
                }
                else
                {
                    NaiveDate::from_ymd_opt(*year, *month + 1, 1)
                };

                if let Some(plus_1) = plus_1
                {
                    plus_1.pred_opt()
                }
                else
                {
                    None
                }
            },
            Self::YearMonthDay(year, month, day) =>
            {
                // Last day is just the same day - it's only one day
                
                NaiveDate::from_ymd_opt(*year, *month, *day)
            },
        }
    }

    pub(crate) fn first_timestamp_after_local_adjust(&self) -> Option<NaiveDateTime>
    {
        // Returns a "local" timestamp for the start of the period.
        // This is the exact timestamp, in seconds since midnight
        // 1 Jan 1970 *local time*, *after* the UTC timestamp has been
        // adjusted for the timezone.

        if let Some(date) = self.first_date_opt()
        {
            date.and_hms_opt(0, 0, 0)
        }
        else
        {
            None
        }
    }

    pub(crate) fn first_timestamp_utc_false_positive(&self) -> Option<NaiveDateTime>
    {
        // Returns UTC timestamp that can be used to quickly filter
        // to near the start of the range. However, it may include a few extra
        // results depending on the time-zone of the localized time.
        // These can be manually filtered out.
        //
        // To do this, we get the final timestamp after adjustment
        // and subtract 14 hours (the biggest timezone adjustment possible).

        if let Some(ts) = self.first_timestamp_after_local_adjust()
        {
            ts.checked_sub_signed(Duration::hours(14))
        }
        else
        {
            None
        }
    }

    pub(crate) fn last_timestamp_after_local_adjust(&self) -> Option<NaiveDateTime>
    {
        // Returns a "local" timestamp for the end of the period.
        // This is the exact timestamp, in seconds since midnight
        // 1 Jan 1970 *local time*, *after* the UTC timestamp has been
        // adjusted for the timezone.

        if let Some(date) = self.last_date_opt()
        {
            date.and_hms_opt(23, 59, 59)
        }
        else
        {
            None
        }
    }

    pub(crate) fn last_timestamp_utc_false_positive(&self) -> Option<NaiveDateTime>
    {
        // Returns UTC timestamp that can be used to quickly filter
        // to near the end of the range. However, it may include a few extra
        // results depending on the time-zone of the localized time.
        // These can be manually filtered out.
        //
        // To do this, we get the final timestamp after adjustment
        // and add 14 hours (the biggest timezone adjustment possible).

        if let Some(ts) = self.last_timestamp_after_local_adjust()
        {
            ts.checked_add_signed(Duration::hours(14))
        }
        else
        {
            None
        }
    }
}

impl FromStr for DateRangeExtent
{
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        // First, see if we can convert the string
        // into a valid year/month/year.

        let extent = || -> Option<DateRangeExtent>
        {
            if let Ok(date) = s.parse::<NaiveDate>()
            {
                // It's a full date - just use this

                return Some(DateRangeExtent::YearMonthDay(date.year(), date.month(), date.day()))
            }
            
            if let Ok(year) = s.parse::<i32>()
            {
                if NaiveDate::from_ymd_opt(year, 1, 1).is_some()
                    && (year >= 1900)
                    && (year < 3000)
                {
                    // It's just a year

                    return Some(DateRangeExtent::Year(year))
                }
            }

            if let Ok(date) = NaiveDate::parse_from_str(&format!("{}/1", s), "%Y/%m/%d")
            {
                // It's a year and month, separated by slashes

                return Some(DateRangeExtent::YearMonth(date.year(), date.month()));
            }

            if let Ok(date) = NaiveDate::parse_from_str(&format!("{}-1", s), "%Y-%m-%d")
            {
                // It's a year and month, separated by dashes

                return Some(DateRangeExtent::YearMonth(date.year(), date.month()));
            }

            if let Ok(date) = NaiveDate::parse_from_str(s, "%Y/%m/%d")
            {
                // It's a year month and date, separated by slashes

                return Some(DateRangeExtent::YearMonthDay(date.year(), date.month(), date.day()));
            }

            None
        }();

        // Now, check that all of the operations on this date/time
        // return valid values

        let extent = if let Some(extent) = extent
        {
            if extent.first_date_opt().is_some()
                && extent.first_timestamp_after_local_adjust().is_some()
                && extent.first_timestamp_utc_false_positive().is_some()
                && extent.last_date_opt().is_some()
                && extent.last_timestamp_after_local_adjust().is_some()
                && extent.last_timestamp_utc_false_positive().is_some()
            {
                Some(extent)
            }
            else
            {
                None
            }
        }
        else
        {
            None
        };

        // Finally, convert the Option to a Result

        extent.ok_or(ParseError::new(format!("Invalid DateRangeExtent {:?}", s)))
    }
}

impl FromStr for DateRange
{
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        // First, see if it's just one date

        if let Ok(dr) = s.trim().parse::<DateRangeExtent>()
        {
            return Ok(DateRange{ start: dr.clone(), end: dr.clone() });
        }

        // Next, see if it's two dates. Split by the string " to " and
        // by dashes, creating a list of pairs

        let mut pairs = Vec::new();

        {
            let split = s.split(" to ").collect::<Vec<_>>();
            if split.len() == 2
            {
                pairs.push((split[0].to_owned(), split[1].to_owned()));
            }
        }

        {
            let chars = s.chars().collect::<Vec<char>>();
            for i in 0..chars.len()
            {
                if chars[i] == '-'
                {
                    pairs.push((chars[0..i].iter().collect::<String>(), chars[(i+1)..].iter().collect::<String>()));
                }
            }
        }

        for (a, b) in pairs
        {
            if let (Ok(start), Ok(end)) = (a.trim().parse::<DateRangeExtent>(), b.trim().parse::<DateRangeExtent>())
            {
                if start.first_date_opt().unwrap() <= end.last_date_opt().unwrap()
                {
                    return Ok(DateRange{start, end});
                }
            }
        }

        Err(ParseError::new(format!("Invalid DateRange {:?}", s)))
    }
}

impl ToString for DateRange
{
    fn to_string(&self) -> String
    {
        format!("{} to {}",
            self.start.first_date_opt().unwrap().format("%Y-%m-%d"),
            self.end.last_date_opt().unwrap().format("%Y-%m-%d"))
    }
}

#[cfg(test)]
mod tests
{
    use super::DateRange;

    fn check_date_range(from_str: &str, to_str: &str, start: &str, end: &str)
    {
        let dr = from_str.parse::<DateRange>();
        assert!(dr.is_ok());
        let dr = dr.unwrap();

        assert_eq!(&dr.to_string(), to_str);

        let start = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d").unwrap();
        let end = chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d").unwrap();

        assert_eq!(dr.first_date(), start);
        assert_eq!(dr.last_date(), end);
    }

    #[test]
    fn test_date_range()
    {
        check_date_range("2020", "2020-01-01 to 2020-12-31", "2020-01-01", "2020-12-31");
        check_date_range("2020 to 2021", "2020-01-01 to 2021-12-31", "2020-01-01", "2021-12-31");
        check_date_range("2020-2021", "2020-01-01 to 2021-12-31", "2020-01-01", "2021-12-31");

        check_date_range("2020/03", "2020-03-01 to 2020-03-31", "2020-03-01", "2020-03-31");

        check_date_range("2020-02-12 to 2020-03-27", "2020-02-12 to 2020-03-27", "2020-02-12", "2020-03-27");
        check_date_range("2020-02-12-2020-03-27", "2020-02-12 to 2020-03-27", "2020-02-12", "2020-03-27");
        check_date_range("2020/02/12-2020-03-27", "2020-02-12 to 2020-03-27", "2020-02-12", "2020-03-27");

        assert!("2020-2019".parse::<DateRange>().is_err());
        assert!("2020-02-12 to 2020-01-27".parse::<DateRange>().is_err());
    }
}