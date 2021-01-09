use horrorshow::{Raw, Template, labels, owned_html};
use crate::icons::{IconSize, OutlineIcon};
use picvudb::data::Rating;

pub fn render(name: &str, rating: &Rating) -> Raw<String>
{
    let rust_name = name.replace("-", "_");
    let num_stars = rating.num_stars();

    Raw(owned_html!
    {
        label(for=rust_name.clone())
        {
            : "Rating";
        }
        input(id=format!("hidden-{}", name), type="hidden", name=rust_name, value=rating.num_stars().to_string());

        div(id=format!("combo-{}", name), class="combo-list")
        {
            @for r in all_ratings()
            {
                a(class=labels!(
                        "combo-item",
                        "combo-selected" => r == *rating,
                        "rating-yellow" => (r.num_stars() > 0) && (r.num_stars() <= num_stars)),
                    href=format!("javascript:picvu.set_combo('{}', '{}', picvu.rating_combo);", name, r.num_stars().to_string()),
                    value=r.num_stars().to_string())
                {
                    div(class="combo-icon")
                    {
                        @if r.num_stars() == 0
                        {
                            : OutlineIcon::DashCircle.render(IconSize::Size32x32)
                        }
                        else if r.num_stars() <= num_stars
                        {
                            : OutlineIcon::StarFill.render(IconSize::Size32x32)
                        }
                        else
                        {
                            : OutlineIcon::Star.render(IconSize::Size32x32)
                        }
                    }

                    div { : rating_to_strs(&r).0; }
                    div { : rating_to_strs(&r).1; }
                }
            }
        }
    }.into_string().unwrap())
}

fn rating_to_strs(rating: &Rating) -> (&'static str, &'static str)
{
    match rating
    {
        Rating::NotRated => ("Not", "Rated"),
        Rating::OneStar => ("One", "Star"),
        Rating::TwoStars => ("Two", "Stars"),
        Rating::ThreeStars => ("Three", "Stars"),
        Rating::FourStars => ("Four", "Stars"),
        Rating::FiveStars => ("Five", "Stars"),
    }
}

fn all_ratings() -> Vec<Rating>
{
    vec! [
        Rating::NotRated,
        Rating::OneStar,
        Rating::TwoStars,
        Rating::ThreeStars,
        Rating::FourStars,
        Rating::FiveStars,
    ]
}