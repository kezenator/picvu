pub fn bytes_to_str(bytes: u64) -> String
{
    if bytes < 4 * 1024
    {
        format!("{} bytes", bytes)
    }
    else if bytes < 1500 * 1024
    {
        format!("{:.1} kB", (bytes as f64) / 1024.0)
    }
    else if bytes < 1500 * 1024 * 1024
    {
        format!("{:.1} MB", (bytes as f64) / 1024.0 / 1024.0)
    }
    else
    {
        format!("{:.1} GB", ((bytes / 1024 / 1024) as f64) / 1024.0)
    }
}

pub fn date_to_str(date: &picvudb::data::Date, _now: &picvudb::data::Date) -> String
{
    date.to_rfc3339()
}