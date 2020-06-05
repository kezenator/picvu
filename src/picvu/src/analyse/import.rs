use std::path::Path;
use image::GenericImageView;

use crate::analyse;

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
    else if ext == "webp"
    {
        Some("image/webp".parse().unwrap())
    }
    else if ext == "mp4"
    {
        Some(format!("{}/{}", mime::VIDEO.as_str(), mime::MP4.as_str()).parse().unwrap())
    }
    else if ext == "mkv"
    {
        Some("video/x-matroska".parse().unwrap())
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
    opt_google_photos_takeout_metadata: Option<analyse::takeout::Metadata>,
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
    let mut orientation = None;
    let mut dimensions = None;
    let mut duration = None;

    let mut got_better_activity_time = false;

    // Try and guess the MIME type

    let mut mime = guess_mime_type_from_filename(file_name)
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

            attachment_created_time = picvudb::data::Date::from_chrono(&local_date_time);
            obj_created_time = Some(attachment_created_time.clone());
        }

        if let Ok(md_modified_timestamp) = metadata.modification_time.timestamp.parse::<i64>()
        {
            let local_date_time = chrono::DateTime::<chrono::Utc>::from_utc(
                    chrono::NaiveDateTime::from_timestamp(md_modified_timestamp, 0),
                    chrono::Utc)
                .with_timezone(&chrono::Local);

            attachment_modified_time = picvudb::data::Date::from_chrono(&local_date_time);
        }

        if let Ok(md_photo_taken_timestamp) = metadata.photo_taken_time.timestamp.parse::<i64>()
        {
            let local_date_time = chrono::DateTime::<chrono::Utc>::from_utc(
                    chrono::NaiveDateTime::from_timestamp(md_photo_taken_timestamp, 0),
                    chrono::Utc)
                .with_timezone(&chrono::Local);

            obj_activity_time = Some(picvudb::data::Date::from_chrono(&local_date_time));
            got_better_activity_time = true;
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
        match analyse::img::ImgAnalysis::decode(&bytes, &file_name)
        {
            Ok(Some((analysis, exif_warnings))) =>
            {
                for w in exif_warnings
                {
                    warnings.push(format!("{}: EXIF Warning: {}", file_name, w));
                }

                if analysis.orig_taken.is_some()
                {
                    obj_activity_time = analysis.orig_taken;
                    got_better_activity_time = true;
                }

                if analysis.location.is_some()
                {
                    location = analysis.location;
                }

                if analysis.orientation.is_some()
                {
                    orientation = analysis.orientation;
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

    // See if we can process image dimensions

    if mime.type_() == mime::IMAGE
    {
        let image = image::load_from_memory(&bytes);
        if let Ok(image) = image
        {
            dimensions = Some(picvudb::data::Dimensions::new(image.width(), image.height()));
        }
    }

    // See if the image also contains a MP4 moving image

    if mime == mime::IMAGE_JPEG
    {
        match analyse::img::parse_mvimg_split(&bytes, file_name)
        {
            analyse::img::MvImgSplit::Neither
                | analyse::img::MvImgSplit::JpegOnly =>
            {
                // It's just an image file - nothing more do to
            },
            analyse::img::MvImgSplit::Both{mp4_offset} =>
            {
                // It's a JPEG with an MP4 attached - analyse the
                // MP4 to collect a duration

                match analyse::video::analyse_video(&bytes[mp4_offset..], file_name, 128)
                {
                    Err(err) =>
                    {
                        warnings.push(format!("{}: MVIMG Video analysis error: {:?}", file_name, err));
                    },
                    Ok(info) =>
                    {
                        duration = info.duration;
                    }
                }        
            },
            analyse::img::MvImgSplit::Mp4Only =>
            {
                // Google photos sometimes generates ".jpg" files that
                // don't contain an image and are only a MP4 movie.
                // Just change the MIME type and continue processing it
                // as a movie (below).

                if let Ok(new_mime) = "video/mp4".parse::<mime::Mime>()
                {
                    mime = new_mime;
                }
            },
        }
    }

    // See if we can obtain any video analysis information

    if mime.type_() == mime::VIDEO
    {
        match analyse::video::analyse_video(&bytes, file_name, 128)
        {
            Err(err) =>
            {
                warnings.push(format!("{}: Video analysis error: {:?}", file_name, err));
            },
            Ok(info) =>
            {
                if !got_better_activity_time
                {
                    if let Some(date) = info.date
                    {
                        obj_activity_time = Some(date);
                        //got_better_activity_time = true;
                    }                    
                }

                if location.is_none()
                {
                    location = info.location;
                }

                if orientation.is_none()
                {
                    orientation = info.orientation;
                }

                if dimensions.is_none()
                {
                    dimensions = info.dimensions;
                }

                if duration.is_none()
                {
                    duration = info.duration;
                }
            },
        }
    }

    // Adjust the dimensions for any orienatation
    // that will be applied to the raw image

    if let Some(dim) = dimensions
    {
        dimensions = Some(dim.adjust_for_orientation(&orientation));
    }
    
    // Construct the Add Object request

    let attachment = picvudb::data::add::Attachment
    {
        filename: file_name.clone(),
        created: attachment_created_time,
        modified: attachment_modified_time,
        mime: mime.clone(),
        orientation: orientation,
        dimensions: dimensions,
        duration: duration,
        bytes: bytes,
    };

    let data = picvudb::data::add::ObjectData
    {
        title: Some(title),
        notes: notes,
        rating: None,
        censor: picvudb::data::Censor::FamilyFriendly,
        created_time: obj_created_time,
        activity_time: obj_activity_time,
        location: location,
        attachment: attachment,
    };

    let msg = picvudb::msgs::AddObjectRequest{ data };

    Ok(msg)
}