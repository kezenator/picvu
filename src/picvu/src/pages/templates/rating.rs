use horrorshow::{Raw, Template, labels, owned_html};
use crate::icons::{IconSize, OutlineIcon};
use picvudb::data::Rating;

pub fn render(rating: &Option<Rating>) -> Raw<String>
{
    let num_stars = rating.clone().map_or(0, |r| r.num_stars());

    Raw(owned_html!
    {
        input(id="hidden-rating", type="hidden", name="rating", value=rating_to_strs(rating).0);

        div(class="combo-list")
        {
            @for r in all_ratings()
            {
                a(class=labels!(
                        "combo-item",
                        "combo-selected" => r == *rating,
                        "rating-yellow" => r.is_some() && r.clone().map_or(0, |r| r.num_stars()) <= num_stars),
                    href=format!("javascript:submit_funcs.rating('{}');", rating_to_strs(&r).0))
                {
                    div(class="combo-icon")
                    {
                        @if r.is_none()
                        {
                            : OutlineIcon::DashCircle.render(IconSize::Size32x32)
                        }
                        else if r.clone().map_or(0, |r| r.num_stars()) <= num_stars
                        {
                            : OutlineIcon::StarFill.render(IconSize::Size32x32)
                        }
                        else
                        {
                            : OutlineIcon::Star.render(IconSize::Size32x32)
                        }
                    }

                    div { : rating_to_strs(&r).1; }
                    div { : rating_to_strs(&r).2; }
                }
            }
        }
    }.into_string().unwrap())
}

fn rating_to_strs(rating: &Option<Rating>) -> (&'static str, &'static str, &'static str)
{
    match rating
    {
        None => ("", "Not", "Rated"),
        Some(rating) =>
        {
            match rating
            {
                Rating::OneStar => ("1", "One", "Star"),
                Rating::TwoStars => ("2", "Two", "Stars"),
                Rating::ThreeStars => ("3", "Three", "Stars"),
                Rating::FourStars => ("4", "Four", "Stars"),
                Rating::FiveStars => ("5", "Five", "Stars"),
            }
        }
    }
}

fn all_ratings() -> Vec<Option<Rating>>
{
    vec! [
        None,
        Some(Rating::OneStar),
        Some(Rating::TwoStars),
        Some(Rating::ThreeStars),
        Some(Rating::FourStars),
        Some(Rating::FiveStars),
    ]
}