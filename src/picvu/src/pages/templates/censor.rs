use horrorshow::{Raw, Template, labels, owned_html};
use crate::icons::{IconSize, ColoredIcon};
use picvudb::data::Censor;

pub fn render(censor: &Censor) -> Raw<String>
{
    Raw(owned_html!
    {
        input(id="hidden-censor", type="hidden", name="censor", value=censor.to_string());

        div(class="combo-list")
        {
            @for c in all_censors()
            {
                a(class=labels!(
                        "combo-item",
                        "combo-selected" => c == *censor),
                    href=format!("javascript:submit_funcs.censor('{}');", c.to_string()))
                {
                    div(class="combo-icon")
                    {
                        : censor_to_strs(&c).2.render(IconSize::Size32x32);
                    }

                    div { : censor_to_strs(&c).0; }
                    div { : censor_to_strs(&c).1; }
                }
            }
        }
    }.into_string().unwrap())
}

fn censor_to_strs(censor: &Censor) -> (&'static str, &'static str, ColoredIcon)
{
    match censor
    {
        Censor::FamilyFriendly => ("Family", "Friendly", ColoredIcon::ManWomanBoy),
        Censor::TastefulNudes => ("Tasteful", "Nudes", ColoredIcon::Peach),
        Censor::FullNudes => ("Full", "Nudes", ColoredIcon::Eggplant),
        Censor::Explicit => ("Sexy and", "Explicit", ColoredIcon::EvilGrin),
    }
}

fn all_censors() -> Vec<Censor>
{
    vec! [
        Censor::FamilyFriendly,
        Censor::TastefulNudes,
        Censor::FullNudes,
        Censor::Explicit,
    ]
}