use horrorshow::{owned_html, Raw, Template};

#[derive(Debug, Clone)]
pub enum OutlineIcon
{
    AlertTriangle,
    Calendar,
    Cancel,
    CloudUpload,
    Edit,
    Export,
    FilePlus,
    FileText,
    Image,
    Import,
    Label,
    List,
    Location,
    Login,
    Save,
    Search,
    Settings,
    Star,
    Sun,
    Trash2,
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
    Trash,
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
            OutlineIcon::AlertTriangle => "exclamation-triangle",
            OutlineIcon::Calendar => "calendar4-week",
            OutlineIcon::Cancel => "x-circle",
            OutlineIcon::CloudUpload => "cloud-upload",
            OutlineIcon::Edit => "pencil-square",
            OutlineIcon::Export => "cloud-download",
            OutlineIcon::FilePlus => "file-earmark-plus",
            OutlineIcon::FileText => "file-earmark-text",
            OutlineIcon::Image => "file-earmark-image",
            OutlineIcon::Import => "cloud-upload",
            OutlineIcon::Label => "tag",
            OutlineIcon::List => "list-ul",
            OutlineIcon::Location => "geo-alt",
            OutlineIcon::Login => "box-arrow-in-right",
            OutlineIcon::Save => "check-circle",
            OutlineIcon::Search => "search",
            OutlineIcon::Settings => "gear",
            OutlineIcon::Star => "star",
            OutlineIcon::Sun => "sun",
            OutlineIcon::Trash2 => "trash",
            OutlineIcon::User => "person",
            OutlineIcon::Video => "camera-video",
        };

        let size = match size
        {
            IconSize::Size16x16 => 16,
            IconSize::Size32x32 => 32,
        };

        let html = owned_html!
        {
            i(class=format!("bi-{} icon-{}", name, size), style=format!("font-size: {}px; color: {}", size, color))
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
            ColoredIcon::Trash => "&#x1F5D1",
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
