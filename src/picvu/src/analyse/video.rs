use std::io::Write;

use picvudb::data::{Date, Dimensions, Duration, Location, Orientation};

#[derive(Debug)]
pub struct VideoAnalysisResults
{
    pub date: Option<Date>,
    pub location: Option<Location>,
    pub orientation: Option<Orientation>,
    pub dimensions: Option<Dimensions>,
    pub duration: Option<Duration>,
}

pub fn analyse_video(bytes: &[u8]) -> Result<VideoAnalysisResults, std::io::Error>
{
    let mut date = None;
    let mut location = None;
    let mut orientation = None;
    let mut dimensions = None;
    let mut duration = None;

    let video_file = tempfile::NamedTempFile::new()?;
    video_file.as_file().write_all(bytes)?;

    let output = std::process::Command::new("ffprobe").arg(video_file.path()).output()?;
    let output = String::from_utf8(output.stderr).map_err(|e| { std::io::Error::new(std::io::ErrorKind::InvalidData, format!("ffprobe output is no UTF-8: {:?}", e)) })?;

    println!("ffprobe output: {}", output);

    for line in output.split('\n')
    {
        if let Some(offset) = line.find(':')
        {
            let param = line[0..offset].trim();
            let value = line[(offset+1)..].trim();
            
            match param
            {
                "creation_time" =>
                {
                    if date.is_none()
                    {
                        if let Ok(decoded) = value.parse::<chrono::DateTime<chrono::Utc>>()
                        {
                            date = Some(Date::from_chrono(&decoded));
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
                                location = Some(Location::new(lat, long, None));
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
                },
            }
        }
    }

    Ok(VideoAnalysisResults { date, location, orientation, dimensions, duration })
}