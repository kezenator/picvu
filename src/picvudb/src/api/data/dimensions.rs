use crate::data::Orientation;

#[derive(Clone, Debug)]
pub struct Dimensions
{
    pub width: u32,
    pub height: u32,
}

impl Dimensions
{
    pub fn new(width: u32, height: u32) -> Self
    {
        Dimensions{ width, height }
    }

    pub fn adjust_for_orientation(&self, orientation: &Option<Orientation>) -> Self
    {
        if let Some(Orientation::RotatedLeft) = orientation
        {
            Dimensions{ width: self.height, height: self.width }
        }
        else if let Some(Orientation::RotatedRight) = orientation
        {
            Dimensions{ width: self.height, height: self.width }
        }
        else
        {
            self.clone()
        }
    }

    pub(crate) fn to_db_field_width(&self) -> i32
    {
        self.width as i32
    }

    pub(crate) fn to_db_field_height(&self) -> i32
    {
        self.height as i32
    }

    pub(crate) fn from_db_fields(width: Option<i32>, height: Option<i32>) -> Option<Self>
    {
        if let Some(width) = width
        {
            if let Some(height) = height
            {
                let width = width as u32;
                let height = height as u32;

                return Some(Dimensions{ width, height });
            }
        }
        None
    }
}

impl ToString for Dimensions
{
    fn to_string(&self) -> String
    {
        format!("{} x {}", self.width, self.height)
    }
}