use std::io::{Read, Write};

use picvudb::data::{Date, Dimensions, Duration, Location, LocationSource, Orientation};
use crate::analyse::google::GoogleCache;
use crate::analyse::tz::ExplicitTimezone;
use crate::analyse::warning::{Warning, WarningKind};

#[derive(Debug)]
pub struct Thumbnail
{
    pub filename: String,
    pub mime: mime::Mime,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct VideoAnalysisResults
{
    pub date: Option<Date>,
    pub location: Option<Location>,
    pub orientation: Option<Orientation>,
    pub dimensions: Option<Dimensions>,
    pub duration: Option<Duration>,
    pub thumbnail: Option<Thumbnail>,
}

pub fn analyse_video(bytes: &[u8], filename: &str, thumbnail_size: u32, assume_timezone: &Option<ExplicitTimezone>, google_cache: Option<&GoogleCache>, warnings: &mut Vec<Warning>) -> Result<VideoAnalysisResults, std::io::Error>
{
    let mut date = None;
    let mut location = None;
    let mut orientation = None;
    let mut dimensions = None;
    let mut duration = None;
    let mut thumbnail = None;

    let mut times_are_local = false;

    let video_file = tempfile::NamedTempFile::new()?;
    video_file.as_file().write_all(bytes)?;

    let output = std::process::Command::new("ffprobe").arg(video_file.path()).output()?;
    let output = String::from_utf8(output.stderr).map_err(|e| { std::io::Error::new(std::io::ErrorKind::InvalidData, format!("ffprobe output is no UTF-8: {:?}", e)) })?;

    for line in output.split('\n')
    {
        if let Some(offset) = line.find(':')
        {
            let param = line[0..offset].trim();
            let value = line[(offset+1)..].trim();
            
            match param
            {
                "compatible_brands" =>
                {
                    // For the movies my Nikon CoolPix W300 makes,
                    // FFPROBE reports the creation time with a "Z" prefix,
                    // but the times are actually in local time

                    if value == "mp42avc1niko"
                    {
                        times_are_local = true;
                    }
                },
                "creation_time" =>
                {
                    if date.is_none()
                    {
                        if times_are_local
                        {
                            if let Some(assume_timezone) = assume_timezone.clone()
                            {
                                if let Ok(decoded) = value.parse::<chrono::DateTime<chrono::Utc>>()
                                {
                                    let naive = decoded.naive_local();
                                    let to_local = assume_timezone.from_local_assuming_tz(&naive);
                                    date = Some(to_local);
                                }
                            }
                            else
                            {
                                warnings.push(Warning::new(filename, WarningKind::VideoAnalysis,
                                    format!("No assumed timezone has been provided to process video creation time in local time format: {}", value)));
                            }
                        }
                        else if let Ok(decoded) = value.parse::<chrono::DateTime<chrono::Utc>>()
                        {
                            date = Some(Date::from_chrono_utc(&decoded));
                        }
                    }
                },
                "location" =>
                {
                    let value = value.trim_end_matches('/');
                    if let Some(offset) = value.rfind(|c| c == '-' || c == '+')
                    {
                        if let Ok(lat) = value[0..offset].parse::<f64>()
                        {
                            if let Ok(long) = value[(offset + 1)..].parse::<f64>()
                            {
                                location = Some(Location::new(
                                    LocationSource::CameraGps,
                                    lat,
                                    long,
                                    None));
                            }
                        }
                    }
                },
                "rotate" =>
                {
                    if value == "0"
                    {
                        orientation = Some(Orientation::Straight);
                    }
                    else if value == "90"
                    {
                        orientation = Some(Orientation::RotatedLeft);
                    }
                    else if value == "180"
                    {
                        orientation = Some(Orientation::UpsideDown);
                    }
                    else if value == "270"
                    {
                        orientation = Some(Orientation::RotatedRight);
                    }
                },
                "Duration" =>
                {
                    if let Some(hms) = value.split('.').nth(0)
                    {
                        let parts = hms.split(':').collect::<Vec<_>>();

                        if parts.len() == 3
                        {
                            if let Ok(h) = parts[0].parse::<u32>()
                            {
                                if let Ok(m) = parts[1].parse::<u32>()
                                {
                                    if let Ok(s) = parts[2].parse::<u32>()
                                    {
                                        let seconds = s + (60 * m) + (3600 * h);

                                        duration = Some(Duration::from_seconds(seconds));
                                    }
                                }
                            }
                        }
                    }
                },
                _ =>
                {
                    if param.starts_with("Stream #")
                    {
                        if let Some(_) = value.find("Video:")
                        {
                            for part in value.split(",")
                            {
                                let part = part.trim();
                                if let Some(part) = part.split(' ').nth(0)
                                {
                                    let strs = part.split('x').collect::<Vec<_>>();
                                    if strs.len() == 2
                                    {
                                        if let Ok(w) = strs[0].parse::<u32>()
                                        {
                                            if let Ok(h) = strs[1].parse::<u32>()
                                            {
                                                if dimensions.is_none()
                                                {
                                                    dimensions = Some(Dimensions::new(w, h));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            }
        }
    }

    // Update the time with timezone information if available

    if let Some(google_cache) = google_cache
    {
        if let (Some(loc), Some(d)) = (&location, &date)
        {
            match google_cache.get_timezone_for(loc, d)
            {
                Ok(tz_info) =>
                {
                    date = tz_info.timezone.adjust_opt(&date);
                },
                Err(e) =>
                {
                    warnings.push(Warning::new(filename, WarningKind::VideoAnalysisError,
                        format!("Could not query Google for timezone information: {}", e)));
                },
            }
        }
    }

    // Now, attempt to create a thumbnail

    if let Some(dimensions) = dimensions.clone()
    {
        // FFMPEG automatically applies the rotation. But our dimensions
        // are of the raw data - not the rotated date.
        // So we need to adjust the dimensions for the rotation, and
        // then resize to the requested thumbnail size

        let dimensions = dimensions.adjust_for_orientation(&orientation);
        let dimensions = dimensions.resize_to_max_dimension(thumbnail_size);

        let mut jpeg_file = tempfile::Builder::new().suffix(".jpg").tempfile()?;

        let output = std::process::Command::new("ffmpeg")
            .arg("-i")
            .arg(video_file.path())
            .arg("-vframes")
            .arg("1")
            .arg("-an")
            .arg("-s")
            .arg(format!("{}x{}", dimensions.width, dimensions.height))
            .arg("-ss")
            .arg("0")
            .arg("-y")
            .arg(jpeg_file.path())
            .output()?;

        if output.status.success()
        {
            let mut bytes = Vec::new();
            jpeg_file.read_to_end(&mut bytes)?;

            thumbnail = Some(Thumbnail
            {
                filename: format!("{}.jpg", filename),
                bytes: bytes,
                mime: mime::IMAGE_JPEG,
            });
        }
    }

    Ok(VideoAnalysisResults { date, location, orientation, dimensions, duration, thumbnail })
}