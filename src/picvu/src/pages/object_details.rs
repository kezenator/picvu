use actix_web::{web, HttpRequest, HttpResponse};
use horrorshow::{owned_html, Raw, Template};

use crate::icons::{ColoredIcon, IconSize, OutlineIcon};
use crate::pages::{HeaderLinkCollection, PageResources, PageResourcesBuilder};
use crate::view;
use crate::State;
use crate::analyse;
use crate::format;
use crate::pages;

#[allow(dead_code)]
pub struct ObjectDetailsPage
{
}

impl ObjectDetailsPage
{
    pub fn path_for(obj_id: &picvudb::data::ObjectId) -> String
    {
        format!("/view/object/{}", obj_id.to_string())
    }
}

impl PageResources for ObjectDetailsPage
{
    fn page_resources(builder: &mut PageResourcesBuilder)
    {
        builder
            .route_view("/view/object/{obj_id}", web::get().to(get_object_details));
    }
}

async fn get_object_details(state: web::Data<State>, object_id: web::Path<String>, req: HttpRequest) -> Result<HttpResponse, view::ErrorResponder>
{
    let object_id = picvudb::data::ObjectId::try_new(object_id.to_string())?;

    let query = picvudb::data::get::GetObjectsQuery::ByObjectId(object_id);

    let msg = picvudb::msgs::GetObjectsRequest
    {
        query,
        pagination: None,
    };

    let response = state.db.send(msg).await??;
    let mut objects = response.objects;
    let object = objects.drain(..).nth(0);

    match object
    {
        None =>
        {
            Ok(view::err(HttpResponse::NotFound(), "Not Found"))
        },
        Some(object) =>
        {
            let get_attachment_data_msg = picvudb::msgs::GetAttachmentDataRequest{
                object_id: object.id.clone(),
                specific_hash: None,
            };

            let attachment_response = state.db.send(get_attachment_data_msg).await??;

            match attachment_response
            {
                picvudb::msgs::GetAttachmentDataResponse::ObjectNotFound
                    | picvudb::msgs::GetAttachmentDataResponse::HashNotFound =>
                {
                    Ok(view::err(HttpResponse::NotFound(), "Not Found"))
                },
                picvudb::msgs::GetAttachmentDataResponse::Found{metadata, bytes} =>
                {
                    let api_key = pages::setup::get_api_key(&state).await?;
                    let google_cache = analyse::google::GoogleCache::new(api_key);
                    let google_cache1 = google_cache.clone();
                    let google_cache2 = google_cache.clone();

                    let mut timezone_info: Option<analyse::google::TimezoneInfo> = None;
                    let mut geocode_info: Option<analyse::google::ReverseGeocode> = None;

                    if let Some(location) = &object.location
                    {
                        let location1 = location.clone();
                        let timestamp1 = object.activity_time.clone();
                        let location2 = location.clone();

                        if let Ok(tz_result) = web::block(move ||
                            {
                                google_cache1.get_timezone_for(&location1, &timestamp1)
                            }).await
                        {
                            timezone_info = Some(tz_result);
                        }

                        if let Ok(rg_result) = web::block(move ||
                            {
                                google_cache2.reverse_geocode(&location2)
                            }).await
                        {
                            geocode_info = Some(rg_result);
                        }
                    }

                    let image_analysis = analyse::img::ImgAnalysis::decode(&bytes, &metadata.filename, Some(&google_cache));
                    let mvimg_split = analyse::img::parse_mvimg_split(&bytes, &metadata.filename);

                    Ok(render_object_details(object, image_analysis, mvimg_split, timezone_info, geocode_info, &req, &state.header_links))
                },
            }
        },
    }
}

fn render_object_details(object: picvudb::data::get::ObjectMetadata, image_analysis: Result<Option<(analyse::img::ImgAnalysis, Vec<analyse::warning::Warning>)>, analyse::img::ImgAnalysisError>, mvimg_split: analyse::img::MvImgSplit, timezone_info: Option<analyse::google::TimezoneInfo>, geocode_info: Option<analyse::google::ReverseGeocode>, req: &HttpRequest, header_links: &HeaderLinkCollection) -> HttpResponse
{
    let now = picvudb::data::Date::now();

    let filename = object.attachment.filename.clone();

    let title = view::Title
    {
        text: object.title.clone().map(|m| m.get_display_text()).unwrap_or(filename.clone()),
        html: Raw(object.title.clone().map(|m| m.get_html()).unwrap_or(owned_html!{ : filename.clone() }.into_string().unwrap())),
    };

    let icon = if object.attachment.mime.type_() == mime::IMAGE
    {
        OutlineIcon::Image
    }
    else if object.attachment.mime.type_() == mime::VIDEO
    {
        OutlineIcon::Video
    }
    else
    {
        OutlineIcon::FileText
    };

    let contents = owned_html!
    {
        div(class="cmdbar cmdbar-top")
        {
            a(href=pages::edit_object::EditObjectPage::path_for(&object.id), class="cmdbar-link")
            {
                : OutlineIcon::Edit.render(IconSize::Size16x16);
                : " Edit"
            }
            a(href=pages::delete_object::DeleteObjectPage::path_for(&object.id), class="cmdbar-link")
            {
                : OutlineIcon::Trash2.render(IconSize::Size16x16);
                : " Delete"
            }
            div(class="cmdbar-summary")
            {
            }
        }
        
        table(class="details-table")
        {
            tr
            {
                th(colspan="2"): "Preview";
            }
            tr
            {
                td(colspan="2")
                {
                    a(href=pages::attachments::AttachmentsPage::path_attachment(&object.id, &object.attachment.hash))
                    {
                        : pages::attachments::AttachmentsPage::raw_html_for_thumbnail(&object, 512, true);
                    }
                }
            }

            tr
            {
                th(colspan="2")
                {
                    : "Details";
                }
            }

            tr
            {
                td: "Created";
                td: format::date_to_str(&object.created_time, &now);
            }
            tr
            {
                td: "Modified";
                td: format::date_to_str(&object.modified_time, &now);
            }
            tr
            {
                td: "Activity";
                td: format::date_to_str(&object.activity_time, &now);
            }

            @if object.title.is_some()
            {
                tr
                {
                    td: "Title";
                    td: Raw(object.title.clone().map(|m| m.get_html()).unwrap_or(String::new()));
                }
            }

            tr
            {
                td: "Rating";
                td
                {
                    @for _ in 0..object.rating.num_stars()
                    {
                        : ColoredIcon::Star.render(IconSize::Size16x16);
                    }
                    : " ";
                    : object.rating.to_string();
                }
            }

            tr
            {
                td: "Censor";
                td
                {
                    : (match object.censor
                        {
                            picvudb::data::Censor::FamilyFriendly => ColoredIcon::ManWomanBoy,
                            picvudb::data::Censor::TastefulNudes => ColoredIcon::Peach,
                            picvudb::data::Censor::FullNudes => ColoredIcon::Eggplant,
                            picvudb::data::Censor::Explicit => ColoredIcon::EvilGrin,
                        }).render(IconSize::Size16x16);

                    : " ";
                    : object.censor.to_string();
                }
            }

            @if !object.tags.is_empty()
            {
                tr
                {
                    td: "Tags";
                    td
                    {
                        @for tag in object.tags.iter()
                        {
                            a(href=pages::object_listing::ObjectListingPage::path(picvudb::data::get::GetObjectsQuery::TagByActivityDesc{ tag_id: tag.tag_id.clone() }),
                                class="tag")
                            {
                                : pages::templates::tags::render_existing(tag);
                            }
                        }
                    }
                }
            }

            @if object.notes.is_some()
            {
                tr
                {
                    td: "Notes";
                    td: Raw(object.notes.clone().map(|m| m.get_html()).unwrap_or(String::new()));
                }
            }

            @if let Some(ext_ref) = object.ext_ref
            {
                tr
                {
                    td: "Reference";
                    td
                    {
                        a(href=ext_ref.get_url(), target="_blank")
                        {
                            : ext_ref.get_type()
                        }
                    }
                }
            }

            : location_details(&object.location, &timezone_info, &geocode_info);

            : attachment_details(&object.id, &object.attachment, &mvimg_split, &now);

            : exif_details(&image_analysis);
        }
    }.into_string().unwrap();

    view::html_page(req, header_links, title, icon, &contents)
}

fn exif_details(exif: &Result<Option<(analyse::img::ImgAnalysis, Vec<analyse::warning::Warning>)>, analyse::img::ImgAnalysisError>) -> Raw<String>
{
    let exif = exif.clone();
    let now = picvudb::data::Date::now();

    Raw(owned_html!
    {
        @if let Ok(image_analysis) = exif
        {
            @if let Some((image_analysis, exif_warnings)) = image_analysis
            {
                tr
                {
                    th(colspan="2"): "Photo EXIF Data";
                }

                @if let Some(orientation) = &image_analysis.orientation
                {
                    tr
                    {
                        td: "Orientation";
                        td: orientation.to_string();
                    }
                }

                @if let Some(make_model) = &image_analysis.make_model
                {
                    tr
                    {
                        td: "Model";
                        td: format!("{} {}", make_model.make, make_model.model);
                    }
                }

                @if let Some(orig_taken) = &image_analysis.orig_taken
                {
                    tr
                    {
                        td: "Taken";
                        td: format::date_to_str(&orig_taken, &now);
                    }
                }

                @if image_analysis.orig_taken_naive.is_some()
                    || image_analysis.orig_digitized_naive.is_some()
                    || image_analysis.gps_timestamp.is_some()
                {
                    tr
                    {
                        td: "Timestamps";
                        td
                        {
                            @if let Some(taken) = image_analysis.orig_taken_naive
                            {
                                p: format!("Orig Taken: {:?}", taken);
                            }
                            @if let Some(digitized) = image_analysis.orig_digitized_naive
                            {
                                p: format!("Digitized: {:?}", digitized);
                            }
                            @if let Some(gps) = image_analysis.gps_timestamp
                            {
                                p: format!("GPS: {:?}", gps);
                            }
                        }
                    }
                }

                @if let Some(camera_settings) = &image_analysis.camera_settings
                {
                    tr
                    {
                        td: "Camera Settings";
                        td: format!("{} {} {} {}",
                            camera_settings.aperture,
                            camera_settings.exposure_time,
                            camera_settings.focal_length,
                            camera_settings.iso);
                    }
                }

                : location_details(&image_analysis.location, &None, &None);

                @if let Some(dop) = image_analysis.location_dop
                {
                    tr
                    {
                        td: "Location DOP";
                        td: format!("{:.1}", dop);
                    }
                }

                @for w in exif_warnings
                {
                    tr
                    {
                        td: "Warning";
                        td
                        {
                            : format!("{:?}", w);
                        }
                    }
                }
            }
        }
        else if let Err(image_analysis_err) = exif
        {
            tr
            {
                th(colspan="2"): "Photo EXIF Data";
            }

            tr
            {
                td: "Error";
                td: image_analysis_err.msg.clone();
            }
        }
    }.into_string().unwrap())
}

fn location_details(location: &Option<picvudb::data::Location>, timezone_info: &Option<analyse::google::TimezoneInfo>, geocode_info: &Option<analyse::google::ReverseGeocode>) -> Raw<String>
{
    let location = location.clone();

    Raw(owned_html!
    {
        @if let Some(location) = location
        {
            tr
            {
                td: "Location";
                td
                {
                    p
                    {
                        a(href=format!("https://www.google.com/maps/search/?api=1&query={},{}", location.latitude, location.longitude),
                            target="_blank")
                        {
                            : format!("{}, {}", location.latitude, location.longitude);
                        }

                        @if let Some(altitude) = location.altitude
                        {
                            : format!(", {:.0}m", altitude);
                        }

                        : format!(", ({:?})", location.source);
                    }

                    @if let Some(timezone_info) = timezone_info.clone()
                    {
                        p
                        {
                            : format!("Timezone {} ID {:?} Name {:?}",
                                timezone_info.timezone.to_string(),
                                timezone_info.id,
                                timezone_info.name);
                        }
                    }

                    @if let Some(geocode_info) = geocode_info.clone()
                    {
                        p
                        {
                            : format!("Geocode {:?}", geocode_info);
                        }
                    }
                }
            }

            @if let Some(altitude) = location.altitude
            {
                tr
                {
                    td: "Altitude";
                    td: format!("{:.0} m", altitude);
                }
            }
        }
    }.into_string().unwrap())
}

fn attachment_details(obj_id: &picvudb::data::ObjectId, attachment: &picvudb::data::get::AttachmentMetadata, mvimg_split: &analyse::img::MvImgSplit, now: &picvudb::data::Date) -> Raw<String>
{
    let now = now.clone();
    let obj_id = obj_id.clone();
    let file_name = attachment.filename.clone();
    let created = attachment.created.clone();
    let modified = attachment.modified.clone();
    let size = attachment.size.clone();
    let mime = attachment.mime.clone();
    let orientation = attachment.orientation.clone();
    let dimensions = attachment.dimensions.clone();
    let duration = attachment.duration.clone();
    let hash = attachment.hash.clone();
    let mvimg_split = mvimg_split.clone();

    Raw(owned_html!
    {
        tr
        {
            th(colspan="2"): "Attachment";
        }
        tr
        {
            td: "File Name";
            td: file_name;
        }
        tr
        {
            td: "Created";
            td: format::date_to_str(&created, &now);
        }
        tr
        {
            td: "Modified";
            td: format::date_to_str(&modified, &now);
        }
        tr
        {
            td: "Size";
            td: format::bytes_to_string(size);
        }
        tr
        {
            td: "Mime Type";
            td: mime.to_string();
        }
        @if let Some(orientation) = orientation
        {
            tr
            {
                td: "Orientation";
                td: orientation.to_string();
            }
        }
        @if let Some(dimensions) = dimensions
        {
            tr
            {
                td: "Dimensions";
                td: dimensions.to_string();
            }
        }
        @if let Some(duration) = duration
        {
            tr
            {
                td: "Duration";
                td: duration.to_string();
            }
        }
        tr
        {
            td: "Hash";
            td: hash.clone();
        }
        tr
        {
            td: "Link";
            td
            {
                p
                {
                    a(href=pages::attachments::AttachmentsPage::path_attachment(&obj_id, &hash)): "View";
                }

                @if let analyse::img::MvImgSplit::Mp4Only = mvimg_split
                {
                    p
                    {
                        a(href=pages::attachments::AttachmentsPage::path_mvimg(&obj_id, &hash))
                        {
                            : "This file is actually a MP4 movie, not a JPEG image.";
                        }
                    }
                }
                else if let analyse::img::MvImgSplit::Both{mp4_offset} = mvimg_split
                {
                    p
                    {
                        a(href=pages::attachments::AttachmentsPage::path_mvimg(&obj_id, &hash))
                        {
                            : format!("This file contains a {} JPEG image then a {} MP4 movie",
                                format::bytes_to_string(mp4_offset as u64),
                                format::bytes_to_string(size - (mp4_offset as u64)));
                        }
                    }
                }
            }
        }
    }.into_string().unwrap())
}
