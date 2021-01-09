use horrorshow::{owned_html, Raw, Template};
use crate::icons::{ColoredIcon, IconSize, OutlineIcon};

pub fn render_existing(tag: &picvudb::data::get::TagMetadata) -> Raw<String>
{
    render_add(&picvudb::data::add::Tag
    {
        name: tag.name.clone(),
        kind: tag.kind.clone(),
        rating: tag.rating.clone(),
        censor: tag.censor.clone(),
    })
}

pub fn render_add(tag: &picvudb::data::add::Tag) -> Raw<String>
{
    let contents = owned_html!
    {
        : (match tag.kind
            {
                picvudb::data::TagKind::Activity => OutlineIcon::Sun,
                picvudb::data::TagKind::Event => OutlineIcon::Calendar,
                picvudb::data::TagKind::Label => OutlineIcon::Label,
                picvudb::data::TagKind::List => OutlineIcon::List,
                picvudb::data::TagKind::Location => OutlineIcon::Location,
                picvudb::data::TagKind::Person => OutlineIcon::User,
                picvudb::data::TagKind::Trash => OutlineIcon::Trash2,
                picvudb::data::TagKind::Unsorted => OutlineIcon::PatchQuestion,
            }).render(IconSize::Size16x16);

        @if tag.rating != picvudb::data::Rating::NotRated
        {
            : OutlineIcon::Star.render(IconSize::Size16x16);
        }

        : (match tag.censor
        {
            picvudb::data::Censor::FamilyFriendly => Raw(String::new()),
            picvudb::data::Censor::TastefulNudes => ColoredIcon::Peach.render(IconSize::Size16x16),
            picvudb::data::Censor::FullNudes => ColoredIcon::Eggplant.render(IconSize::Size16x16),
            picvudb::data::Censor::Explicit => ColoredIcon::EvilGrin.render(IconSize::Size16x16),
        });

        : " ";
        : &tag.name;
    }.into_string().unwrap();

    Raw(contents)
}
