use horrorshow::{labels, owned_html, Raw, Template};
use crate::pages;
use crate::icons::{ColoredIcon, IconSize};

pub fn render(object: &picvudb::data::get::ObjectMetadata, href: String, selected: bool) -> Raw<String>
{
    let icons_style = |o: &picvudb::data::get::ObjectMetadata|
    {
        let dimensions = o.attachment.dimensions.clone().map(|d| d.resize_to_max_dimension(128));
        let width = dimensions.clone().map_or("100%".to_owned(), |d| format!("{}px", d.width));
        let height = dimensions.map_or("100%".to_owned(), |d| format!("{}px", d.height));

        format!("width: {};height: {};", width, height)
    };

    Raw(owned_html!
    {
        div(class=labels!("object-listing-entry", "object-listing-selected" => selected))
        {
            div(class="object-listing-thumbnail")
            {
                a(href=href)
                {
                    div(class="object-listing-icons", style=icons_style(object))
                    {
                        @if object.tags.iter().filter(|t| t.kind == picvudb::data::TagKind::Trash).count() != 0
                        {
                            : ColoredIcon::Trash.render(IconSize::Size16x16);
                        }

                        @if object.notes.is_some()
                        {
                            : ColoredIcon::Memo.render(IconSize::Size16x16);
                        }

                        @if object.location.is_some()
                        {
                            : ColoredIcon::RoundPushpin.render(IconSize::Size16x16);
                        }

                        @if object.rating.is_some()
                        {
                            : ColoredIcon::Star.render(IconSize::Size16x16);
                        }

                        : (match object.censor
                            {
                                picvudb::data::Censor::FamilyFriendly => Raw(String::new()),
                                picvudb::data::Censor::TastefulNudes => ColoredIcon::Peach.render(IconSize::Size16x16),
                                picvudb::data::Censor::FullNudes => ColoredIcon::Eggplant.render(IconSize::Size16x16),
                                picvudb::data::Censor::Explicit => ColoredIcon::EvilGrin.render(IconSize::Size16x16),
                            });

                        @if let Some(duration) = &object.attachment.duration
                        {
                            : ColoredIcon::Play.render(IconSize::Size16x16);
                            : " ";
                            : duration.to_string();
                        }
                    }

                    : pages::attachments::AttachmentsPage::raw_html_for_thumbnail(&object, 128, false);
                }
            }

            div(class="object-listing-title")
            {
                @if let Some(title) = &object.title
                {
                    : render_with_zero_width_spaces(title.get_events());
                }
                else
                {
                    : render_with_zero_width_spaces(vec![ pulldown_cmark::Event::Text(pulldown_cmark::CowStr::Borrowed(&object.attachment.filename))].drain(..));
                }
            }
        }
    }.into_string().unwrap())
}

fn render_with_zero_width_spaces<'a, T: Iterator<Item = pulldown_cmark::Event<'a>>>(events: T) -> Raw<String>
{
    let mut result = String::new();

    let events = events.map(|e|
    {
        match e
        {
            pulldown_cmark::Event::Text(t) =>
            {
                let t = t.to_string();
                let t = t.replace("_", "_\u{200B}");
                return pulldown_cmark::Event::Text(t.into());
            },
            pulldown_cmark::Event::Code(t) =>
            {
                let t = t.to_string();
                let t = t.replace("_", "_\u{200B}");
                return pulldown_cmark::Event::Code(t.into());
            },
            _ => {},
        }
        // Other events unchanged
        e
    });

    pulldown_cmark::html::push_html(&mut result, events);

    Raw(result)
}
