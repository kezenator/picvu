use horrorshow::{owned_html, Raw, Template};

#[derive(Debug, Clone)]
pub enum OutlineIcon
{
    AlertTriangle,
    Calendar,
    CloudUpload,
    Edit,
    Export,
    Delete,
    FilePlus,
    FileText,
    Image,
    Import,
    Label,
    List,
    Location,
    Login,
    Search,
    Settings,
    Star,
    Sun,
    User,
    Video,
}

#[derive(Debug, Clone)]
pub enum ColoredIcon
{
    ManWomanBoy,
    Peach,
    Eggplant,
    EvilGrin,
    Memo,
    Play,
    RoundPushpin,
    Star,
}

#[derive(Debug, Clone)]
pub enum Icon
{
    Outline(OutlineIcon),
    Color(ColoredIcon)
}

pub enum IconSize
{
    Size16x16,
    Size32x32,
}

impl OutlineIcon
{
    pub fn render(&self, size: IconSize) -> Raw<String>
    {
        self.render_internal(size, "#000000")
    }

    fn render_internal(&self, size: IconSize, color: &str) -> Raw<String>
    {
        let name = match self
        {
            OutlineIcon::AlertTriangle => "alert-triangle",
            OutlineIcon::Calendar => "calendar",
            OutlineIcon::CloudUpload => "upload-cloud",
            OutlineIcon::Edit => "edit",
            OutlineIcon::Export => "hard-drive",
            OutlineIcon::Delete => "delete",
            OutlineIcon::FilePlus => "file-plus",
            OutlineIcon::FileText => "file-text",
            OutlineIcon::Image => "image",
            OutlineIcon::Import => "plus-square",
            OutlineIcon::Label => "tag",
            OutlineIcon::List => "list",
            OutlineIcon::Location => "map-pin",
            OutlineIcon::Login => "log-in",
            OutlineIcon::Search => "search",
            OutlineIcon::Settings => "settings",
            OutlineIcon::Star => "star",
            OutlineIcon::Sun => "sun",
            OutlineIcon::User => "user",
            OutlineIcon::Video => "video",
        };

        let size = match size
        {
            IconSize::Size16x16 => 16,
            IconSize::Size32x32 => 32,
        };

        let html = owned_html!
        {
            svg(class=format!("icon-{}", size), width=size, height=size, fill="none", stroke=color, stroke-width="2", stroke-linecap="round", stroke-linejoin="round")
            {
                use(xlink:href=format!("/assets/feather-sprite.svg#{}", name));
            }
        }.into_string().unwrap();

        Raw(html)
    }
}

impl ColoredIcon
{
    pub fn render(&self, _size: IconSize) -> Raw<String>
    {
        let text = match self
        {
            ColoredIcon::ManWomanBoy => "&#x1F468;&#x200D;&#x1F469;&#x200D;&#x1F466;",
            ColoredIcon::Peach => "&#x1F351;",
            ColoredIcon::Eggplant => "&#x1F346;",
            ColoredIcon::EvilGrin => "&#x1F608;",
            ColoredIcon::Memo => "&#x1F4DD;",
            ColoredIcon::Play => "&#x25B6;",
            ColoredIcon::RoundPushpin => "&#x1F4CD;",
            ColoredIcon::Star => "&#x2B50;",
        };

        Raw(text.to_owned())
    }
}

impl Icon
{
    pub fn render(&self, size: IconSize) -> Raw<String>
    {
        match self
        {
            Self::Outline(outline) =>
            {
                outline.render(size)
            },
            Self::Color(colored) =>
            {
                colored.render(size)
            },
        }
    }
}

impl From<OutlineIcon> for Icon
{
    fn from(icon: OutlineIcon) -> Icon
    {
        Icon::Outline(icon)
    }
}

impl From<ColoredIcon> for Icon
{
    fn from(icon: ColoredIcon) -> Icon
    {
        Icon::Color(icon)
    }
}
