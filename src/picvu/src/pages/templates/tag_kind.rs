use horrorshow::{Raw, Template, labels, owned_html};
use crate::icons::{IconSize, OutlineIcon};
use picvudb::data::TagKind;

pub fn render(name: &str, kind: &TagKind) -> Raw<String>
{
    let rust_name = name.replace("-", "_");

    Raw(owned_html!
    {
        label(for=rust_name.clone())
        {
            : "Icon";
        }
        input(id=format!("hidden-{}", name), type="hidden", name=rust_name, value=kind.to_string());

        div(id=format!("combo-{}", name), class="combo-list")
        {
            @for k in all_kinds()
            {
                a(class=labels!(
                        "combo-item",
                        "combo-selected" => k == *kind),
                    href=format!("javascript:picvu.set_combo('{}', '{}');", name, k.to_string()),
                    value=k.to_string())
                {
                    div(class="combo-icon")
                    {
                        : censor_to_strs(&k).1.render(IconSize::Size32x32);
                    }

                    div { : censor_to_strs(&k).0; }
                }
            }
        }
    }.into_string().unwrap())
}

fn censor_to_strs(kind: &TagKind) -> (&'static str, OutlineIcon)
{
    match kind
    {
        TagKind::Location => ("Location", OutlineIcon::Location),
        TagKind::Person => ("Person", OutlineIcon::User),
        TagKind::Event => ("Event", OutlineIcon::Calendar),
        TagKind::Label => ("Label", OutlineIcon::Label),
        TagKind::List => ("List", OutlineIcon::List),
        TagKind::Activity => ("Activity", OutlineIcon::Sun),
        TagKind::Trash => ("Trash", OutlineIcon::Trash2),
    }
}

fn all_kinds() -> Vec<TagKind>
{
    vec! [
        TagKind::Label,
        TagKind::Location,
        TagKind::Event,
        TagKind::Person,
        TagKind::List,
        TagKind::Activity,
        TagKind::Trash,
    ]
}