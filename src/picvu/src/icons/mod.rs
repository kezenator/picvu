use horrorshow::{owned_html, Raw, Template};

#[derive(Debug, Clone)]
pub enum Icon
{
    Calendar,
    CloudUpload,
    Edit,
    FilePlus,
    Image,
    Import,
    Label,
    List,
    Location,
    Login,
    Search,
    Settings,
}

pub enum IconSize
{
    Size16x16,
    Size32x32,
}

impl Icon
{
    pub fn render(&self, size: IconSize) -> Raw<String>
    {
        let name = match self
        {
            Icon::Calendar => "calendar",
            Icon::CloudUpload => "upload-cloud",
            Icon::Edit => "edit",
            Icon::FilePlus => "file-plus",
            Icon::Image => "image",
            Icon::Import => "plus-square",
            Icon::Label => "tag",
            Icon::List => "list",
            Icon::Location => "map-pin",
            Icon::Login => "log-in",
            Icon::Search => "search",
            Icon::Settings => "settings",
        };

        let size = match size
        {
            IconSize::Size16x16 => "16",
            IconSize::Size32x32 => "32",
        };

        let html = owned_html!
        {
            svg(class=format!("icon-{}", size), width=size, height=size, fill="none", stroke="#000000", stroke-width="2", stroke-linecap="round", stroke-linejoin="round")
            {
                use(xlink:href=format!("/assets/feather-sprite.svg#{}", name));
            }
        }.into_string().unwrap();

        Raw(html)
    }
}