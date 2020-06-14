use std::str::FromStr;
use chrono::FixedOffset;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplicitTimezone(FixedOffset);

impl ExplicitTimezone
{
    pub fn new(offset: FixedOffset) -> Self
    {
        ExplicitTimezone(offset)
    }

    pub fn adjust(&self, val: &picvudb::data::Date) -> picvudb::data::Date
    {
        let adjust_chrono = |v: chrono::DateTime<chrono::FixedOffset>| -> chrono::DateTime<chrono::FixedOffset>
        {
            v.with_timezone(&self.0)
        };
        
        picvudb::data::Date::from_chrono_fixed(&adjust_chrono(val.to_chrono_fixed_offset()))
    }

    pub fn adjust_opt(&self, val: &Option<picvudb::data::Date>) -> Option<picvudb::data::Date>
    {
        val.clone().map(|v| self.adjust(&v))
    }

    pub fn from_local_assuming_tz(&self, local: &chrono::NaiveDateTime) -> picvudb::data::Date
    {
        let utc = *local + chrono::Duration::seconds(self.0.utc_minus_local().into());
        let assumed = chrono::DateTime::<chrono::FixedOffset>::from_utc(utc, self.0);
        picvudb::data::Date::from_chrono_fixed(&assumed)
    }
}

impl From<FixedOffset> for ExplicitTimezone
{
    fn from(offset: FixedOffset) -> Self
    {
        ExplicitTimezone::new(offset)
    }
}

impl ToString for ExplicitTimezone
{
    fn to_string(&self) -> String
    {
        self.0.to_string()
    }
}

impl FromStr for ExplicitTimezone
{
    type Err = ExplicitTimezoneParseError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let parts = s.split(':').collect::<Vec<_>>();

        if parts.len() == 2
        {
            let hours: i32 = parts[0].parse().map_err(|_| ExplicitTimezoneParseError)?;
            let mins: i32 = parts[1].parse().map_err(|_| ExplicitTimezoneParseError)?;

            if hours >= -23 && hours <= 23
            {
                if mins >= 0 && mins <= 59
                {
                    let mins = (hours * 60) + mins;

                    return Ok(FixedOffset::east_opt(mins * 60).ok_or(ExplicitTimezoneParseError)?.into());
                }
            }
        }

        Err(ExplicitTimezoneParseError)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplicitTimezoneParseError;

#[cfg(test)]
mod tests
{
    use chrono::FixedOffset;
    use super::ExplicitTimezone;

    #[test]
    fn test_explicit_timezone()
    {
        assert_eq!("+10:00".to_owned(), ExplicitTimezone::new(FixedOffset::east(36000)).to_string());
        assert_eq!("+10:00".parse::<ExplicitTimezone>(), Ok(ExplicitTimezone::new(FixedOffset::east(36000))));
    }
}