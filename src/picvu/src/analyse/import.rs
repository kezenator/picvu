use crate::analyse::takeout;
use std::path::Path;

pub fn guess_mime_type_from_filename(filename: &String) -> Option<mime::Mime>
{
    let ext = Path::new(filename).extension().unwrap_or_default().to_str().unwrap_or_default().to_owned().to_ascii_lowercase();

    if (ext == "jpg") || (ext == "jpeg")
    {
        Some(mime::IMAGE_JPEG)
    }
    else if ext == "png"
    {
        Some(mime::IMAGE_PNG)
    }
    else if ext == "gif"
    {
        Some(mime::IMAGE_GIF)
    }
    else if ext == "mp4"
    {
        Some(format!("{}/{}", mime::VIDEO.as_str(), mime::MP4.as_str()).parse().unwrap())
    }
    else
    {
        None
    }
}

pub fn create_add_object_for_import(
    bytes: Vec<u8>,
    file_name: &String,
    opt_file_created_time: Option<picvudb::data::Date>,
    opt_file_modified_time: Option<picvudb::data::Date>,
    opt_google_photos_takeout_metadata: Option<takeout::Metadata>,
    warnings: &mut Vec<String>) -> Result<picvudb::msgs::AddObjectRequest, std::io::Error>
{
    let now = picvudb::data::Date::now();

    // Take default values from the information provided

    let mut title = file_name.clone();
    let mut notes = None;
    let mut obj_created_time = None;
    let mut obj_activity_time = None;
    let mut location = None;
    let mut attachment_created_time = opt_file_created_time.unwrap_or(now.clone());
    let mut attachment_modified_time = opt_file_modified_time.unwrap_or(now.clone());

    // Try and guess the MIME type

    let mime = guess_mime_type_from_filename(file_name)
        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Cannot guess MIME type for file {}", file_name)))?;


    // Process the Google Photos Takeout Metadata if provided

    if let Some(metadata) = opt_google_photos_takeout_metadata
    {
        if !metadata.title.is_empty()
        {
            title = metadata.title;
        }

        if !metadata.description.is_empty()
        {
            notes = Some(metadata.description);
        }

        // Google Photos Takeout always provides timestamps in UTC
        // with no timezone information. We can get better timezone information
        // from photos with EXIF data (see below).
        // If we only have these times available, then lets change
        // them into our local timezone.

        if let Ok(md_create_timestamp) = metadata.creation_time.timestamp.parse::<i64>()
        {
            let local_date_time = chrono::DateTime::<chrono::Utc>::from_utc(
                    chrono::NaiveDateTime::from_timestamp(md_create_timestamp, 0),
                    chrono::Utc)
                .with_timezone(&chrono::Local);

            attachment_created_time = picvudb::data::Date::from_chrono_datetime(local_date_time);
            obj_created_time = Some(attachment_created_time.clone());
        }

        if let Ok(md_modified_timestamp) = metadata.modification_time.timestamp.parse::<i64>()
        {
            let local_date_time = chrono::DateTime::<chrono::Utc>::from_utc(
                    chrono::NaiveDateTime::from_timestamp(md_modified_timestamp, 0),
                    chrono::Utc)
                .with_timezone(&chrono::Local);

            attachment_modified_time = picvudb::data::Date::from_chrono_datetime(local_date_time);
        }

        if let Ok(md_photo_taken_timestamp) = metadata.photo_taken_time.timestamp.parse::<i64>()
        {
            let local_date_time = chrono::DateTime::<chrono::Utc>::from_utc(
                    chrono::NaiveDateTime::from_timestamp(md_photo_taken_timestamp, 0),
                    chrono::Utc)
                .with_timezone(&chrono::Local);

            obj_activity_time = Some(picvudb::data::Date::from_chrono_datetime(local_date_time));
        }

        if let Some(md_location) = metadata.geo_data
        {
            // Google photos takeout always provides metadata,
            // but sets all values to zero if it doesn't actually have
            // any. (0, 0) is in the ocean off the coast of Africa - lets
            // assume that the photo wasn't actually taken there

            if md_location.latitude != 0.0
                || md_location.longitude != 0.0
                || md_location.altitude != 0.0
            {
                location = Some(picvudb::data::Location::new(
                    md_location.latitude,
                    md_location.longitude,
                    Some(md_location.altitude)));
            }
        }
    }

    // Process the image EXIF data if provided

    if mime.type_() == mime::IMAGE
    {
        match crate::analyse::img::ImgAnalysis::decode(&bytes, &file_name)
        {
            Ok(Some((analysis, exif_warnings))) =>
            {
                for w in exif_warnings
                {
                    warnings.push(format!("{}: EXIF Warning: {}", file_name, w));
                }

                if analysis.original_datetime.is_some()
                {
                    obj_activity_time = analysis.original_datetime;
                }

                if analysis.location.is_some()
                {
                    location = analysis.location;
                }
            },
            Ok(None) =>
            {
            },
            Err(err) =>
            {
                warnings.push(format!("{}: EXIF Error: {:?}", file_name, err));
            },
        }
    }
    
    // Construct the Add Object request

    let attachment = picvudb::data::add::Attachment
    {
        filename: file_name.clone(),
        created: attachment_created_time,
        modified: attachment_modified_time,
        mime: mime.clone(),
        bytes: bytes,
    };

    let additional = if mime.type_() == mime::IMAGE
    {
        picvudb::data::add::AdditionalData::Photo
        {
            attachment
        }
    }
    else if mime.type_() == mime::VIDEO
    {
        picvudb::data::add::AdditionalData::Video
        {
            attachment
        }
    }
    else
    {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unsupported MIME {}", mime))
            .into()
        );
    };

    let data = picvudb::data::add::ObjectData
    {
        title: Some(title),
        notes: notes,
        created_time: obj_created_time,
        activity_time: obj_activity_time,
        location: location,
        additional,                            
    };

    let msg = picvudb::msgs::AddObjectRequest{ data };

    Ok(msg)
}