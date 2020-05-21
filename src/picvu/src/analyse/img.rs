use std::convert::TryInto;
use chrono::offset::TimeZone;

#[derive(Debug)]
pub enum Orientation
{
    Undefined,
    Straight,
    RotatedRight,
    UpsideDown,
    RotatedLeft,
}

impl Orientation
{
    fn from_rexif_str(s: &str) -> Result<Self, ImgAnalysisError>
    {
        match s
        {
            "Straight" => Ok(Orientation::Straight),
            "Upside down" => Ok(Orientation::UpsideDown),
            "Rotated to left" => Ok(Orientation::RotatedLeft),
            "Rotated to right" => Ok(Orientation::RotatedRight),
            "Undefined" => Ok(Orientation::Undefined),
            _ => Err(ImgAnalysisError{
                msg: format!("Unknown Orientation {:?}", s),
                debug_entries: Vec::new(),
            }),
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct ImgAnalysisError
{
    msg: String,
    debug_entries: Vec<DebugEntry>,
}

#[derive(Debug)]
pub struct Exposure
{
    pub time: String,
    pub aperture: String,
    pub iso: String,
}

#[derive(Debug)]
pub struct Location
{
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_meters: f64,
    pub dop: f64,
}

#[derive(Debug)]
pub struct ImgAnalysis
{
    pub mime: mime::Mime,
    pub orientation: Orientation,
    pub original_datetime: Option<picvudb::data::Date>,
    pub exposure: Option<Exposure>,
    pub location: Option<Location>,
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
    pub fn decode(data: &Vec<u8>, _filename: &String) -> Result<Option<Self>, ImgAnalysisError>
    {
        match rexif::parse_buffer(&data)
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

                // Orientation

                let mut orientation = Orientation::Undefined;
                if let Some(entry) = find_single(&exif.entries, rexif::ExifTag::Orientation)
                {
                    orientation = Orientation::from_rexif_str(&entry.value_more_readable)?;
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

                                let fixedup_diff = vec![-2, -1, 0, 1, 2]
                                    .iter()
                                    .map(|fixup| {difference.checked_add(&chrono::Duration::seconds(*fixup))})
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
                
                let mut exposure = None;
                if let Some(time_entry) = find_single(&exif.entries, rexif::ExifTag::ExposureTime)
                {
                    if let Some(aperture_entry) = find_single(&exif.entries, rexif::ExifTag::FNumber)
                    {
                        if let Some(iso_entry) = find_single(&exif.entries, rexif::ExifTag::ISOSpeedRatings)
                        {
                            exposure = Some(Exposure
                            {
                                time: time_entry.value_more_readable,
                                aperture: aperture_entry.value_more_readable,
                                iso: iso_entry.value_more_readable,
                            });
                        }
                    }
                }

                // Location

                let mut location = None;
                if let Some(lat) = find_single(&exif.entries, rexif::ExifTag::GPSLatitude)
                {
                    if let Some(long) = find_single(&exif.entries, rexif::ExifTag::GPSLongitude)
                    {
                        let lat_ref = find_single(&exif.entries, rexif::ExifTag::GPSLatitudeRef);
                        let long_ref = find_single(&exif.entries, rexif::ExifTag::GPSLongitudeRef);
                        let alt = find_single(&exif.entries, rexif::ExifTag::GPSAltitude);
                        let alt_ref = find_single(&exif.entries, rexif::ExifTag::GPSAltitudeRef);
                        let dop = find_single(&exif.entries, rexif::ExifTag::GPSDOP);

                        if lat_ref.is_none()
                            || long_ref.is_none()
                            || alt.is_none()
                            || alt_ref.is_none()
                            || dop.is_none()
                        {
                            return Err(ImgAnalysisError{
                                msg: format!("Unsupported GPS format - missing info"),
                                debug_entries,
                            });
                        }

                        let lat_ref = lat_ref.unwrap();
                        let long_ref = long_ref.unwrap();
                        let alt = alt.unwrap();
                        let alt_ref = alt_ref.unwrap();
                        let dop = dop.unwrap();

                        if (lat_ref.value_more_readable.eq("N") || lat_ref.value_more_readable.eq("S"))
                            && (long_ref.value_more_readable.eq("E") || long_ref.value_more_readable.eq("W"))
                            && alt_ref.value_more_readable.eq("Above sea level")
                            && alt.unit.eq("m")
                        {
                            let alt = parse_urational(
                                &alt,
                                vec![1.0],
                                format!("Unsupported GPS format - invalid alt"),
                                &debug_entries)?;

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

                            let dop = parse_urational(
                                &dop,
                                vec![1.0],
                                format!("Unsupported GPS format - invalid DOP"),
                                &debug_entries)?;

                            if lat_ref.value_more_readable.eq("S")
                            {
                                lat = -1.0 * lat;
                            }

                            if long_ref.value_more_readable.eq("W")
                            {
                                long = -1.0 * long;
                            }

                            location = Some(Location{
                                latitude: lat,
                                longitude: long,
                                altitude_meters: alt,
                                dop: dop,
                            });
                        }
                        else
                        {
                            return Err(ImgAnalysisError{
                                msg: format!("Unsupported GPS format - invalid units"),
                                debug_entries,
                            });
                        }
                    }
                }

                // Successful decode!

                Ok(Some(ImgAnalysis
                {
                    mime,
                    orientation,
                    original_datetime,
                    exposure,
                    location,
                }))
            },
            Err(_) =>
            {
                Ok(None)
            }
        }
    }
}