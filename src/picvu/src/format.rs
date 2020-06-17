pub fn bytes_to_string(bytes: u64) -> String
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

pub fn meters_to_string(meters: &f64) -> String
{
    if *meters < 1000.0
    {
        format!("{:.0} m", meters)
    }
    else
    {
        format!("{:.1} km", meters / 1000.0)
    }
}

pub fn bytes_to_group_header(bytes: u64) -> String
{
    if bytes >= 10 * 1024 * 1024 * 1024
    {
        "More than 10GB"
    }
    else if bytes >= 1024 * 1024 * 1024
    {
        "More than 1GB"
    }
    else if bytes >= 100 * 1024 * 1024
    {
        "More than 100MB"
    }
    else if bytes >= 10 * 1024 * 1024
    {
        "More than 10MB"
    }
    else if bytes >= 1024 * 1024
    {
        "More than 1MB"
    }
    else if bytes >= 100 * 1024
    {
        "More than 100kB"
    }
    else if bytes >= 10 * 1024
    {
        "More than 10kB"
    }
    else if bytes >= 1024
    {
        "More than 1kB"
    }
    else
    {
        "Less than 1KB"
    }.to_owned()
}

pub fn date_to_str(date: &picvudb::data::Date, _now: &picvudb::data::Date) -> String
{
    match date
    {
        picvudb::data::Date::Utc(utc) =>
        {
            let local = utc.with_timezone(&chrono::Local);
            format!("{} (Local)", local.to_rfc3339())
        },
        picvudb::data::Date::FixedOffset(fixed) =>
        {
            fixed.to_rfc3339()
        },
    }
}

pub fn date_to_date_only_string(date: &picvudb::data::Date) -> String
{
    let rfc3339_str = match date
    {
        picvudb::data::Date::Utc(utc) =>
        {
            let local = utc.with_timezone(&chrono::Local);
            local.to_rfc3339()
        },
        picvudb::data::Date::FixedOffset(fixed) =>
        {
            fixed.to_rfc3339()
        },
    };

    rfc3339_str[0..10].to_owned()
}

pub fn query_to_string(query: &picvudb::data::get::GetObjectsQuery) -> String
{
    match query
    {
        picvudb::data::get::GetObjectsQuery::ByObjectId(id) => format!("Object {}", id.to_string()),
        picvudb::data::get::GetObjectsQuery::ByActivityDesc => "Calendar".to_owned(),
        picvudb::data::get::GetObjectsQuery::ByModifiedDesc => "Recently Modified".to_owned(),
        picvudb::data::get::GetObjectsQuery::ByAttachmentSizeDesc => "Largest Attachments".to_owned(),
        picvudb::data::get::GetObjectsQuery::NearLocationByActivityDesc{ location, radius_meters } => format!("Within {} of {}", meters_to_string(radius_meters), location.to_string()),
        picvudb::data::get::GetObjectsQuery::TitleNotesSearchByActivityDesc{ search } => format!("Search {:?}", search),
        picvudb::data::get::GetObjectsQuery::TagByActivityDesc{ tag_id } => format!("Tag {}", tag_id.to_string()),
    }
}

pub fn insert_zero_width_spaces(value: String) -> String
{
    value.replace("_", "_\u{200B}")
}