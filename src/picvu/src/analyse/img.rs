use std::convert::TryInto;
use chrono::offset::TimeZone;

#[derive(Debug, Clone)]
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
    pub original_datetime: Option<picvudb::data::Date>,
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

                // Original Date/Time

                let mut original_datetime = None;
                {
                    if let Some(gps_date_entry) = find_single(&exif.entries, rexif::ExifTag::GPSDateStamp)
                    {
                        if let Some(gps_time_entry) = find_single(&exif.entries, rexif::ExifTag::GPSTimeStamp)
                        {
                            if let Some(local_original) = find_single(&exif.entries, rexif::ExifTag::DateTimeOriginal)
                            {
                                let gps_datetime = chrono::naive::NaiveDateTime::parse_from_str(
                                    &format!("{} {}", gps_date_entry.value_more_readable, gps_time_entry.value_more_readable),
                                    "%Y:%m:%d %H:%M:%S%.f UTC")
                                    .map_err(|e| { ImgAnalysisError { msg: format!("Can't decode GPS datetime: {:?}", e), debug_entries: debug_entries.clone() }})?;

                                let local_datetime = chrono::naive::NaiveDateTime::parse_from_str(
                                    &local_original.value_more_readable,
                                    "%Y:%m:%d %H:%M:%S")
                                    .map_err(|e| { ImgAnalysisError { msg: format!("Can't decode local datetime: {:?}", e), debug_entries: debug_entries.clone() }})?;

                                let difference = local_datetime.signed_duration_since(gps_datetime);

                                // Accept up to a couple of seconds difference between the two times.
                                // Find an adjustment that makes the difference a while number of minutes.

                                let fixedup_diff = (-120..120)
                                    .map(|fixup| {difference.checked_add(&chrono::Duration::seconds(fixup))})
                                    .filter(|diff| diff.is_some())
                                    .map(|diff| diff.unwrap())
                                    .filter(|diff| *diff == chrono::Duration::minutes(diff.num_minutes()))
                                    .nth(0)
                                    .ok_or(ImgAnalysisError{
                                        msg: format!("GPS/Local difference {:?} ({:?} - {:?}) is not a whole number of minutes", difference, local_datetime, gps_datetime),
                                        debug_entries: debug_entries.clone()
                                    })?;

                                // Now, convert it into a local time using the
                                // differece as the local offset

                                let offset_seconds: i32 = fixedup_diff.num_seconds().try_into().map_err(|_| {ImgAnalysisError{
                                    msg: format!("GPS/Local difference {:?} ({:?} - {:?}) is out of range", fixedup_diff, local_datetime, gps_datetime),
                                    debug_entries: debug_entries.clone()
                                }})?;

                                let fixed_offset = chrono::FixedOffset::east_opt(offset_seconds)
                                    .ok_or(ImgAnalysisError{
                                        msg: format!("GPS/Local difference {:?} ({:?} - {:?}) is out of range", fixedup_diff, local_datetime, gps_datetime),
                                        debug_entries: debug_entries.clone()
                                    })?;

                                let final_local = fixed_offset.from_utc_datetime(&gps_datetime);

                                original_datetime = Some(picvudb::data::Date::from_chrono_datetime(final_local));
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
                    original_datetime,
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