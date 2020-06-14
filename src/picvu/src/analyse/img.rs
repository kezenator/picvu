use std::convert::TryInto;
use chrono::offset::TimeZone;

pub use picvudb::data::Orientation;

use crate::analyse::warning::{Warning, WarningKind};
use crate::analyse::google::GoogleCache;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MvImgSplit
{
    Neither,
    JpegOnly,
    Mp4Only,
    Both{mp4_offset: usize},
}

pub struct ImgAnalysisError
{
    pub msg: String,
}

impl ImgAnalysisError
{
    pub fn new<S: Into<String>>(msg: S) -> Self
    {
        ImgAnalysisError{ msg: msg.into() }
    }
}

fn orientation_from_rexif_str(s: &str) -> Result<Option<Orientation>, String>
{
    match s
    {
        "Straight" => Ok(Some(Orientation::Straight)),
        "Upside down" => Ok(Some(Orientation::UpsideDown)),
        "Rotated to left" => Ok(Some(Orientation::RotatedLeft)),
        "Rotated to right" => Ok(Some(Orientation::RotatedRight)),
        "Undefined" 
            | "Unknown (0)" => Ok(None),
        _ => Err(format!("Unknown Orientation {:?}", s)),
    }
}

#[derive(Debug, Clone)]
pub struct MakeModel
{
    pub make: String,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct CameraSettings
{
    pub exposure_time: String,
    pub aperture: String,
    pub focal_length: String,
    pub iso: String,
}

#[derive(Debug, Clone)]
pub struct ImgAnalysis
{
    pub mime: mime::Mime,
    pub orientation: Option<Orientation>,
    pub orig_taken_naive: Option<chrono::NaiveDateTime>,
    pub orig_digitized_naive: Option<chrono::NaiveDateTime>,
    pub gps_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub orig_taken: Option<picvudb::data::Date>,
    pub make_model: Option<MakeModel>,
    pub camera_settings: Option<CameraSettings>,
    pub location: Option<picvudb::data::Location>,
    pub location_dop: Option<f64>,
}

fn find_single(exif: &Vec<rexif::ExifEntry>, tag: rexif::ExifTag) -> Option<rexif::ExifEntry>
{
    for entry in exif
    {
        if entry.tag == tag
        {
            return Some(entry.clone());
        }
    }
    None
}

fn parse_urational(entry: &rexif::ExifEntry, factors: Vec<f64>, msg: String) -> Result<f64, String>
{
    if let rexif::TagValue::URational(vec) = &entry.value
    {
        if vec.len() == factors.len()
        {
            let mut result = 0.0;

            for i in 0..vec.len()
            {
                result += factors[i] * (vec[i].numerator as f64) / (vec[i].denominator as f64);
            }

            return Ok(result);
        }
    }

    return Err(msg);
}

impl ImgAnalysis
{
    pub fn decode(data: &Vec<u8>, file_name: &String, google_cache: Option<&GoogleCache>) -> Result<Option<(Self, Vec<Warning>)>, ImgAnalysisError>
    {
        let (exif_result, mut exif_warnings) = rexif::parse_buffer_quiet(&data);

        match exif_result
        {
            Ok(exif) =>
            {
                // Mime type

                let mime = exif.mime.parse::<mime::Mime>();
                if mime.is_err()
                {
                    return Err(ImgAnalysisError::new(format!("Mime parse error({:?}): {:?}", exif.mime, mime)));
                }
                let mime = mime.unwrap();

                // Turn the EXIF warnings into our warning type

                let mut exif_warnings = exif_warnings
                        .drain(..)
                        .map(|s| Warning::new(file_name, WarningKind::ImgExifDecode, s))
                        .collect::<Vec<Warning>>();

                // Orientation

                let mut orientation = None;
                if let Some(entry) = find_single(&exif.entries, rexif::ExifTag::Orientation)
                {
                    match orientation_from_rexif_str(&entry.value_more_readable)
                    {
                        Ok(o) =>
                        {
                            orientation = o;
                        },
                        Err(e) =>
                        {
                            exif_warnings.push(Warning::new(file_name, WarningKind::ImgExifAnalyse, e));
                        }
                    }
                }

                // Make/Model

                let mut make_model = None;
                {
                    if let Some(make_entry) = find_single(&exif.entries, rexif::ExifTag::Make)
                    {
                        if let Some(model_entry) = find_single(&exif.entries, rexif::ExifTag::Model)
                        {
                            make_model = Some(MakeModel{
                                make: make_entry.value_more_readable,
                                model: model_entry.value_more_readable,
                            });
                        }
                    }
                }

                // Location

                let mut location = None;
                let mut location_dop = None;

                match calc_location_and_dop(&exif.entries)
                {
                    Ok((l, ldop)) =>
                    {
                        location = l;
                        location_dop = ldop;
                    },
                    Err(e) =>
                    {
                        exif_warnings.push(Warning::new(file_name, WarningKind::ImgExifAnalyse, e));
                    }
                }

                // Original Date/Time, Naive

                let mut orig_taken_naive = None;
                {
                    if let Some(datetime_entry) = find_single(&exif.entries, rexif::ExifTag::DateTimeOriginal)
                    {
                        let datetime_string = datetime_entry.value_more_readable;

                        let datetime_naive = chrono::naive::NaiveDateTime::parse_from_str(
                            &datetime_string,
                            "%Y:%m:%d %H:%M:%S");

                        match datetime_naive
                        {
                            Ok(datetime_naive) =>
                            {
                                orig_taken_naive = Some(datetime_naive);
                            },
                            Err(e) =>
                            {
                                exif_warnings.push(Warning::new(
                                    file_name,
                                    WarningKind::ImgExifAnalyse,
                                    format!("Can't decode DateTimeOriginal \"{}\": {:?}", datetime_string, e)));
                            },
                        }
                    }
                }

                // Originally Digitized Date/Time, Naive

                let mut orig_digitized_naive = None;
                {
                    if let Some(datetime_entry) = find_single(&exif.entries, rexif::ExifTag::DateTimeDigitized)
                    {
                        let datetime_string = datetime_entry.value_more_readable;

                        let datetime_naive = chrono::naive::NaiveDateTime::parse_from_str(
                            &datetime_string,
                            "%Y:%m:%d %H:%M:%S");

                        match datetime_naive
                        {
                            Ok(datetime_naive) =>
                            {
                                orig_digitized_naive = Some(datetime_naive);
                            },
                            Err(e) =>
                            {
                                exif_warnings.push(Warning::new(
                                    file_name,
                                    WarningKind::ImgExifAnalyse,
                                    format!("Can't decode DateTimeDigitized \"{}\": {:?}", datetime_string, e)));
                            },
                        }
                    }
                }

                // GPS Timestamp
                let mut gps_timestamp = None;
                {
                    if let Some(gps_date_entry) = find_single(&exif.entries, rexif::ExifTag::GPSDateStamp)
                    {
                        if let Some(gps_time_entry) = find_single(&exif.entries, rexif::ExifTag::GPSTimeStamp)
                        {
                            let formatted_gps_datetime_string = format!("{} {}", gps_date_entry.value_more_readable, gps_time_entry.value_more_readable);

                            let gps_naive = chrono::naive::NaiveDateTime::parse_from_str(
                                &formatted_gps_datetime_string, "%Y:%m:%d %H:%M:%S%.f UTC");

                            match gps_naive
                            {
                                Ok(gps_naive) =>
                                {
                                    gps_timestamp = Some(chrono::Utc.from_utc_datetime(&gps_naive));
                                },
                                Err(e) =>
                                {
                                    exif_warnings.push(Warning::new(
                                        file_name,
                                        WarningKind::ImgExifAnalyse,
                                        format!("Can't decode GPSDateStamp/GPSTimeStamp \"{}\": {:?}", formatted_gps_datetime_string, e)));
                                },
                            }
                        }
                    }
                }
            

                // Original Date/Time, Local

                let mut orig_taken = None;
                {
                    // If the image has a location, then use the Google services
                    // to look up the timezone. We'll use the "orig_taken" or "GPS",
                    // and we'll just assume the "orig_taken" is a GPS timezone, because
                    // it should be pretty good, and worst case is we get the DST incorrect...

                    if let (Some(gcache), Some(loc)) = (google_cache, &location)
                    {
                        if let Some(gps_ts) = &gps_timestamp
                        {
                            let gps_date = picvudb::data::Date::from_chrono(&gps_ts);

                            match gcache.get_timezone_for(loc, &gps_date)
                            {
                                Ok(tz_info) =>
                                {
                                    orig_taken = Some(tz_info.timezone.adjust(&gps_date));
                                },
                                Err(e) =>
                                {
                                    exif_warnings.push(Warning::new(file_name, WarningKind::ImgExifAnalyse,
                                        format!("Could not query Google for timezone information based on GPS timestamp {}: {}", gps_ts.to_string(), e)));
                                },
                            }
                        }
                        else if let Some(taken_naive_ts) = &orig_taken_naive
                        {
                            let hack_utc_ts = chrono::Utc.from_utc_datetime(&taken_naive_ts);
                            let hack_utc_date = picvudb::data::Date::from_chrono(&hack_utc_ts);

                            match gcache.get_timezone_for(loc, &hack_utc_date)
                            {
                                Ok(tz_info) =>
                                {
                                    orig_taken = Some(tz_info.timezone.from_local_assuming_tz(&taken_naive_ts));
                                },
                                Err(e) =>
                                {
                                    exif_warnings.push(Warning::new(file_name, WarningKind::ImgExifAnalyse,
                                        format!("Could not query Google for timezone information based on Orig Taken local time {}: {}", taken_naive_ts.to_string(), e)));
                                },
                            }
                        }
                    }

                    // If we couldn't use the location and Google services
                    // to work out the timezone, then see if we can look at
                    // the difference between the local time and the GPS timestamp
                    // to get the timezone

                    if orig_taken.is_none()
                    {
                        if let Some(ref_orig_taken_naive) = orig_taken_naive
                        {
                            if let Some(ref_gps) = gps_timestamp
                            {
                                match calc_timezone_from_taken_and_gps(&ref_orig_taken_naive, &ref_gps)
                                {
                                    Ok(calc_timezone) =>
                                    {
                                        orig_taken = Some(calc_timezone);
                                    },
                                    Err(e) =>
                                    {
                                        exif_warnings.push(Warning::new(
                                            file_name, WarningKind::ImgExifAnalyse,
                                            format!("Can't decode timezone from orig taken {} and GPS timestamp {}: {}", ref_orig_taken_naive, ref_gps, e)));
                                    }
                                }
                            }
                        }
                    }
                }

                // Exposure
                
                let mut camera_settings = None;
                if let Some(time_entry) = find_single(&exif.entries, rexif::ExifTag::ExposureTime)
                {
                    if let Some(aperture_entry) = find_single(&exif.entries, rexif::ExifTag::FNumber)
                    {
                        if let Some(focal_length_entry) = find_single(&exif.entries, rexif::ExifTag::FocalLength)
                        {
                            if let Some(iso_entry) = find_single(&exif.entries, rexif::ExifTag::ISOSpeedRatings)
                            {
                                let mut aperture = aperture_entry.value_more_readable;
                                if aperture.starts_with("f/")
                                {
                                    aperture = format!("\u{0192}/{}", &aperture[2..]);
                                }

                                let mut iso = iso_entry.value_more_readable;
                                if iso.starts_with("ISO ")
                                {
                                    iso = format!("ISO{}", &iso[4..]);
                                }

                                camera_settings = Some(CameraSettings
                                {
                                    exposure_time: time_entry.value_more_readable,
                                    aperture: aperture,
                                    focal_length: focal_length_entry.value_more_readable,
                                    iso: iso,
                                });
                            }
                        }
                    }
                }

                // Successful decode!

                Ok(Some((ImgAnalysis
                {
                    mime,
                    orientation,
                    orig_taken_naive,
                    orig_digitized_naive,
                    gps_timestamp,
                    orig_taken,
                    make_model,
                    camera_settings,
                    location,
                    location_dop,
                },
                exif_warnings)))
            },
            Err(err) =>
            {
                match &err
                {
                    rexif::ExifError::FileTypeUnknown
                        | rexif::ExifError::JpegWithoutExif(_)
                        | rexif::ExifError::ExifIfdTruncated(_)
                        | rexif::ExifError::ExifIfdEntryNotFound =>
                    {
                        // We'll just ignore these errors - they seem to
                        // be normal for files types without EXIF (e.g. PNG or GIF)
                        // or JPG files that just don't contain any

                        return Ok(None);
                    },
                    _ => {},
                }

                return Err(ImgAnalysisError::new(format!("EXIF parse error for file {}: {:?}", file_name, err)));
            }
        }
    }
}

pub fn parse_mvimg_split(data: &Vec<u8>, file_name: &String) -> MvImgSplit
{
    let mut result = MvImgSplit::Neither;

    if file_name.ends_with(".jpg")
    {
        result = MvImgSplit::JpegOnly;

        let mp4_header = b"ftypmp4";

        if let Some(pos) = data.windows(mp4_header.len()).position(|window| window == mp4_header)
        {
            if pos == 4
            {
                result = MvImgSplit::Mp4Only;
            }
            else if pos > 4
            {
                result = MvImgSplit::Both{ mp4_offset: pos - 4 };
            }
        }
    }

    result
}

fn calc_location_and_dop(entries: &Vec<rexif::ExifEntry>) -> Result<(Option<picvudb::data::Location>, Option<f64>), String>
{
    let mut location = None;
    let mut location_dop = None;

    if let (Some(lat), Some(long)) = (find_single(entries, rexif::ExifTag::GPSLatitude),
                                        find_single(entries, rexif::ExifTag::GPSLongitude))
    {
        if let (Some(lat_ref), Some(long_ref)) = (find_single(entries, rexif::ExifTag::GPSLatitudeRef),
                                                    find_single(entries, rexif::ExifTag::GPSLongitudeRef))
        {
            if (lat_ref.value_more_readable.eq("N") || lat_ref.value_more_readable.eq("S"))
                && (long_ref.value_more_readable.eq("E") || long_ref.value_more_readable.eq("W"))
            {
                let mut lat = parse_urational(
                    &lat,
                    vec![1.0, 1.0 / 60.0, 1.0 / 3600.0],
                    format!("Unsupported GPS format - invalid lat"))?;

                let mut long = parse_urational(
                    &long,
                    vec![1.0, 1.0 / 60.0, 1.0 / 3600.0],
                    format!("Unsupported GPS format - invalid long"))?;

                if lat_ref.value_more_readable.eq("S")
                {
                    lat = -1.0 * lat;
                }

                if long_ref.value_more_readable.eq("W")
                {
                    long = -1.0 * long;
                }

                let mut alt = None;
                {
                    let alt_entry = find_single(entries, rexif::ExifTag::GPSAltitude);
                    let alt_ref = find_single(entries, rexif::ExifTag::GPSAltitudeRef);

                    if let Some(alt_entry) = alt_entry
                    {
                        if alt_entry.unit != "m"
                        {
                            return Err("Unsupported GPS format - invalid alt units".to_owned());
                        }
                        else
                        {
                            let mut alt_sign_num = 1.0;

                            if let Some(alt_ref) = alt_ref
                            {
                                if (alt_ref.value_more_readable != "Above sea level")
                                    && (alt_ref.value_more_readable != "Below sea level")
                                {
                                    return Err("Unsupported GPS format - invalid alt_ref units".to_owned());
                                }

                                if alt_ref.value_more_readable.eq("Below sea level")
                                {
                                    alt_sign_num = -1.0;
                                }
                            }

                            let alt_val = parse_urational(
                                &alt_entry,
                                vec![1.0],
                                format!("Unsupported GPS format - invalid alt"))?;
                                
                            alt = Some(alt_val * alt_sign_num);
                        }
                    }
                }

                location = Some(picvudb::data::Location::new(lat, long, alt));

                let dop = find_single(entries, rexif::ExifTag::GPSDOP);

                if let Some(dop) = dop
                {
                    let dop = parse_urational(
                        &dop,
                        vec![1.0],
                        format!("Unsupported GPS format - invalid DOP"))?;
            
                    location_dop = Some(dop);
                }
            
            }
            else
            {
                return Err("Unsupported GPS format - invalid lat/long units".to_owned());
            }
        }
        else
        {
            return Err("Unsupported GPS format - missing lat/long reference info".to_owned());
        }
        
    }
    else
    {
        // No location data
    }

    Ok((location, location_dop))
}

fn calc_timezone_from_taken_and_gps(orig_taken_naive: &chrono::NaiveDateTime, gps_timestamp: &chrono::DateTime<chrono::Utc>) -> Result<picvudb::data::Date, String>
{
    let difference = orig_taken_naive.signed_duration_since(gps_timestamp.naive_utc());
    let signed_seconds = difference.num_seconds();
    let abs_seconds = signed_seconds.abs();
    let signum_seconds = signed_seconds.signum();

    // Now, we want to force the seconds to the nearest 15 minute interval

    let seconds_in_15_minutes = 15 * 60;

    let forced_seconds = ((abs_seconds + (seconds_in_15_minutes / 2)) / seconds_in_15_minutes) * seconds_in_15_minutes;

    // Now, we want to ensure that we are within 5 minutes of the difference.
    // E.g. we will accept differences 09:55:00 to 10:05:00 as being 10:00:00.

    {
        let forced_difference = abs_seconds - forced_seconds;

        if (forced_difference > 5*60)
            || (forced_difference < -5*60)
        {
            return Err("Local/UTC timestamps are not withing 5 minutes of a 15 minute interval from each other".to_owned());
        }
    }

    // Add the sign back to the difference and
    // construct a fixed offset timezone

    let offset_seconds = forced_seconds * signum_seconds;

    let offset_seconds: i32 = offset_seconds
        .try_into()
        .map_err(|_| { format!("Offset {} seconds is too large", offset_seconds) })?;

    let offset = chrono::FixedOffset::east_opt(offset_seconds)
        .ok_or(format!("Offset {} seconds is not a valid timezone", offset_seconds))?;

    // Return the converted local time

    match offset.from_local_datetime(orig_taken_naive)
    {
        chrono::LocalResult::None =>
        {
            Err("Local time conversion returned no results".to_owned())
        },
        chrono::LocalResult::Single(t) =>
        {
            Ok(picvudb::data::Date::from_chrono(&t))
        },
        chrono::LocalResult::Ambiguous(_, _) =>
        {
            Err("Local time conversion returned ambiguous results".to_owned())
        },
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn test_time(local: &str, gps: &str, expected: Option<&str>)
    {
        let local = chrono::naive::NaiveDateTime::parse_from_str(local, "%Y:%m:%d %H:%M:%S");
        assert!(local.is_ok());
        let local = local.unwrap();

        let gps =  chrono::naive::NaiveDateTime::parse_from_str(gps, "%Y:%m:%d %H:%M:%S%.f UTC");
        assert!(gps.is_ok());
        let gps = gps.unwrap();
        let gps = chrono::Utc.from_utc_datetime(&gps);

        let result = calc_timezone_from_taken_and_gps(&local, &gps)
            .ok()
            .map(|t| { t.to_chrono_fixed_offset().format("%Y:%m:%d %H:%M:%S %:z").to_string() });

        assert_eq!(result, expected.map(|s| { s.to_owned() }));
    }

    #[test]
    fn test_calc_timezone_from_taken_and_gps()
    {
        // Check positive timezones work - with 5 minute errors allowed

        test_time("2018:03:02 19:20:09", "2018:03:02 09:15:45.0 UTC", Some("2018:03:02 19:20:09 +10:00"));
        test_time("2018:03:02 19:20:09", "2018:03:02 09:20:00.0 UTC", Some("2018:03:02 19:20:09 +10:00"));
        test_time("2018:03:02 19:20:09", "2018:03:02 09:20:09.0 UTC", Some("2018:03:02 19:20:09 +10:00"));
        test_time("2018:03:02 19:20:09", "2018:03:02 09:24:50.0 UTC", Some("2018:03:02 19:20:09 +10:00"));

        // Check negative timezones work - with 5 minute errors allowed

        test_time("2018:03:02 19:20:09", "2018:03:02 20:15:45.0 UTC", Some("2018:03:02 19:20:09 -01:00"));
        test_time("2018:03:02 19:20:09", "2018:03:02 20:20:00.0 UTC", Some("2018:03:02 19:20:09 -01:00"));
        test_time("2018:03:02 19:20:09", "2018:03:02 20:20:09.0 UTC", Some("2018:03:02 19:20:09 -01:00"));
        test_time("2018:03:02 19:20:09", "2018:03:02 20:24:50.0 UTC", Some("2018:03:02 19:20:09 -01:00"));

        // Check that it doesn't work if they are 8 minutes out

        test_time("2018:03:02 19:20:09", "2018:03:02 09:28:09.0 UTC", None);
        test_time("2018:03:02 19:20:09", "2018:03:02 09:12:09.0 UTC", None);
    }
}
