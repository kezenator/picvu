use crate::data::Orientation;

#[derive(Clone, Debug, PartialEq, Eq)]
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

    pub fn resize_to_max_dimension(&self, max: u32) -> Self
    {
        if self.width <= max && self.height <= max
        {
            self.clone()
        }
        else if self.width == self.height
        {
            Dimensions::new(max, max)
        }
        else if self.width > self.height
        {
            let new_height = ((((self.height as u64) * (max as u64)) + ((self.width as u64) / 2)) / (self.width as u64)) as u32;
            Dimensions::new(max, new_height)
        }
        else // height is bigger
        {
            let new_width = ((((self.width as u64) * (max as u64)) + ((self.height as u64) / 2)) / (self.height as u64)) as u32;
            Dimensions::new(new_width, max)
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

#[cfg(test)]
mod tests
{
    use super::Dimensions;

    #[test]
    fn test_dimensions_resize()
    {
        assert_eq!(Dimensions::new(512, 512).resize_to_max_dimension(1024), Dimensions::new(512, 512));

        assert_eq!(Dimensions::new(1024, 512).resize_to_max_dimension(1024), Dimensions::new(1024, 512));
        assert_eq!(Dimensions::new(512, 1024).resize_to_max_dimension(1024), Dimensions::new(512, 1024));

        assert_eq!(Dimensions::new(512, 1024).resize_to_max_dimension(256), Dimensions::new(128, 256));
        assert_eq!(Dimensions::new(1024, 512).resize_to_max_dimension(256), Dimensions::new(256, 128));
    }
}