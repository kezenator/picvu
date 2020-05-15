use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Date
{
    pub(crate) timestamp: i64,
    pub(crate) timestring: String,
}

impl Date
{
    pub fn now() -> Self
    {
        let local = chrono::Local::now();
        let utc = local.with_timezone(&chrono::Utc);

        let timestamp = utc.timestamp();
        let timestring = local.to_rfc3339();

        Date { timestamp, timestring }
    }

    pub fn to_rfc3339(&self) -> String
    {
        self.timestring.clone()
    }
}
