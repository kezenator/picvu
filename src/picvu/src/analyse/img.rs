use std::convert::TryInto;
use chrono::offset::TimeZone;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MvImgSplit
{
    Neither,
    JpegOnly,
    Mp4Only,
    Both{mp4_offset: usize},
}

#[derive(Debug, Clone)]
pub enum Orientation
{
    Straight,
    RotatedRight,
    UpsideDown,
    RotatedLeft,
}

impl Orientation
{
    fn from_rexif_str(s: &str) -> Result<Option<Self>, ImgAnalysisError>
    {
        match s
        {
            "Straight" => Ok(Some(Orientation::Straight)),
            "Upside down" => Ok(Some(Orientation::UpsideDown)),
            "Rotated to left" => Ok(Some(Orientation::RotatedLeft)),
            "Rotated to right" => Ok(Some(Orientation::RotatedRight)),
            "Undefined" 
                | "Unknown (0)" => Ok(None),
            _ => Err(ImgAnalysisError{
                msg: format!("Unknown Orientation {:?}", s),
                debug_entries: Vec::new(),
            }),
        }
    }
}

impl std::fmt::Display for Orientation
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        write!(fmt, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct MakeModel
{
    pub make: String,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct DebugEntry
{
    tag: rexif::ExifTag,
    len: usize,
    value: String,
    unit: String,
    details: String,
    kind: rexif::IfdKind,
}

#[derive(Debug, Clone)]
pub struct ImgAnalysisError
{
    pub msg: String,
    pub debug_entries: Vec<DebugEntry>,
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

fn parse_urational(entry: &rexif::ExifEntry, factors: Vec<f64>, msg: String, debug_entries: &Vec<DebugEntry>) -> Result<f64, ImgAnalysisError>
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

    return Err(ImgAnalysisError{
        msg: msg,
        debug_entries: debug_entries.to_vec(),
    });
}

impl ImgAnalysis
{
    pub fn decode(data: &Vec<u8>, file_name: &String) -> Result<Option<(Self, Vec<String>)>, ImgAnalysisError>
    {
        let (exif_result, exif_warnings) = rexif::parse_buffer_quiet(&data);

        match exif_result
        {
            Ok(exif) =>
            {
                // Mime type

                let mime = exif.mime.parse::<mime::Mime>();
                if mime.is_err()
                {
                    return Err(ImgAnalysisError
                    {
                        msg: format!("Mime parse error({:?}): {:?}", exif.mime, mime),
                        debug_entries: Vec::new(),
                    });
                }
                let mime = mime.unwrap();

                // Debug entries - TODO - remove

                let debug_entries = exif.entries
                    .iter()
                    .map(|entry|
                        {
                            let value = match &entry.value
                            {
                                rexif::TagValue::Ascii(s) => s.to_owned(),
                                rexif::TagValue::U8(d) => format!("U8 {:?}", d),
                                rexif::TagValue::U16(d) => format!("U16 {:?}", d),
                                rexif::TagValue::U32(d) => format!("U32 {:?}", d),
                                rexif::TagValue::URational(d) => format!("{:?}", d),
                                rexif::TagValue::I8(d) => format!("I8 {:?}", d),
                                rexif::TagValue::I16(d) => format!("I16 {:?}", d),
                                rexif::TagValue::I32(d) => format!("I32 {:?}", d),
                                rexif::TagValue::IRational(d) => format!("{:?}", d),
                                rexif::TagValue::F32(d) => format!("F32 {:?}", d),
                                rexif::TagValue::F64(d) => format!("F64 {:?}", d),
                                _ => String::new(),
                            };

                            DebugEntry
                            {
                                tag: entry.tag,
                                len: entry.ifd.data.len(),
                                value: value,
                                unit: entry.unit.clone(),
                                details: entry.value_more_readable.clone(),
                                kind: entry.kind,
                            }
                        })
                    .collect::<Vec<DebugEntry>>();

                // TODO - remove
                //println!("{:#?}", debug_entries);

                // Orientation

                let mut orientation = None;
                if let Some(entry) = find_single(&exif.entries, rexif::ExifTag::Orientation)
                {
                    orientation = Orientation::from_rexif_str(&entry.value_more_readable)?;
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

                // Original Date/Time, Naive

                let mut orig_taken_naive = None;
                {
                    if let Some(datetime_entry) = find_single(&exif.entries, rexif::ExifTag::DateTimeOriginal)
                    {
                        let datetime_string = datetime_entry.value_more_readable;

                        let datetime_naive = chrono::naive::NaiveDateTime::parse_from_str(
                            &datetime_string,
                            "%Y:%m:%d %H:%M:%S")
                            .map_err(|e| { ImgAnalysisError { msg: format!("Can't decode DateTimeOriginal \"{}\": {:?}", datetime_string, e), debug_entries: debug_entries.clone() }})?;

                            orig_taken_naive = Some(datetime_naive);
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
                            "%Y:%m:%d %H:%M:%S")
                            .map_err(|e| { ImgAnalysisError { msg: format!("Can't decode DateTimeDigitized \"{}\": {:?}", datetime_string, e), debug_entries: debug_entries.clone() }})?;

                            orig_digitized_naive = Some(datetime_naive);
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
                                &formatted_gps_datetime_string, "%Y:%m:%d %H:%M:%S%.f UTC")
                                .map_err(|e| { ImgAnalysisError { msg: format!("Can't decode GPSDateStamp/GPSTimeStamp \"{}\": {:?}", formatted_gps_datetime_string, e), debug_entries: debug_entries.clone() }})?;

                            gps_timestamp = Some(chrono::Utc.from_utc_datetime(&gps_naive));
                        }
                    }
                }
            

                // Original Date/Time, Local

                let mut orig_taken = None;
                {
                    if let Some(ref_orig_taken_naive) = orig_taken_naive
                    {
                        if let Some(ref_gps) = gps_timestamp
                        {
                            orig_taken = Some(calc_timezone_from_taken_and_gps(&ref_orig_taken_naive, &ref_gps)
                                .map_err(|e| ImgAnalysisError { msg: format!("Can't decode timezone from orig taken {} and GPS timestamp {}: {}", ref_orig_taken_naive, ref_gps, e), debug_entries: debug_entries.clone() })?);
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

                // Location

                let mut location = None;
                let mut location_dop = None;

                if let Some(lat) = find_single(&exif.entries, rexif::ExifTag::GPSLatitude)
                {
                    if let Some(long) = find_single(&exif.entries, rexif::ExifTag::GPSLongitude)
                    {
                        let lat_ref = find_single(&exif.entries, rexif::ExifTag::GPSLatitudeRef);
                        let long_ref = find_single(&exif.entries, rexif::ExifTag::GPSLongitudeRef);

                        if lat_ref.is_none()
                            || long_ref.is_none()
                        {
                            return Err(ImgAnalysisError{
                                msg: format!("Unsupported GPS format - missing lat/long reference info"),
                                debug_entries,
                            });
                        }

                        let lat_ref = lat_ref.unwrap();
                        let long_ref = long_ref.unwrap();

                        if (lat_ref.value_more_readable.eq("N") || lat_ref.value_more_readable.eq("S"))
                            && (long_ref.value_more_readable.eq("E") || long_ref.value_more_readable.eq("W"))
                        {
                            let mut lat = parse_urational(
                                &lat,
                                vec![1.0, 1.0 / 60.0, 1.0 / 3600.0],
                                format!("Unsupported GPS format - invalid lat"),
                                &debug_entries)?;

                            let mut long = parse_urational(
                                &long,
                                vec![1.0, 1.0 / 60.0, 1.0 / 3600.0],
                                format!("Unsupported GPS format - invalid long"),
                                &debug_entries)?;

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
                                let alt_entry = find_single(&exif.entries, rexif::ExifTag::GPSAltitude);
                                let alt_ref = find_single(&exif.entries, rexif::ExifTag::GPSAltitudeRef);

                                if let Some(alt_entry) = alt_entry
                                {
                                    if alt_entry.unit != "m"
                                    {
                                        return Err(ImgAnalysisError{
                                            msg: format!("Unsupported GPS format - invalid alt units"),
                                            debug_entries,
                                        });
                                    }
                                    else
                                    {
                                        let mut alt_sign_num = 1.0;

                                        if let Some(alt_ref) = alt_ref
                                        {
                                            if (alt_ref.value_more_readable != "Above sea level")
                                                && (alt_ref.value_more_readable != "Below sea level")
                                            {
                                                return Err(ImgAnalysisError{
                                                    msg: format!("Unsupported GPS format - invalid alt_ref units"),
                                                    debug_entries,
                                                });
                                            }

                                            if alt_ref.value_more_readable.eq("Below sea level")
                                            {
                                                alt_sign_num = -1.0;
                                            }
                                        }

                                        let alt_val = parse_urational(
                                            &alt_entry,
                                            vec![1.0],
                                            format!("Unsupported GPS format - invalid alt"),
                                            &debug_entries)?;
                                            
                                        alt = Some(alt_val * alt_sign_num);
                                    }
                                }
                            }

                            location = Some(picvudb::data::Location::new(lat, long, alt));

                            let dop = find_single(&exif.entries, rexif::ExifTag::GPSDOP);

                            if let Some(dop) = dop
                            {
                                let dop = parse_urational(
                                    &dop,
                                    vec![1.0],
                                    format!("Unsupported GPS format - invalid DOP"),
                                    &debug_entries)?;

                                location_dop = Some(dop);
                            }
                        }
                        else
                        {
                            return Err(ImgAnalysisError{
                                msg: format!("Unsupported GPS format - invalid lat/long units"),
                                debug_entries,
                            });
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

                return Err(ImgAnalysisError{
                    msg: format!("EXIF parse error for file {}: {:?}", file_name, err),
                    debug_entries: Vec::new(),
                })
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
            Ok(picvudb::data::Date::from_chrono_datetime(t))
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
