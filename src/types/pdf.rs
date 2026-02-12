use crate::*;
pub struct Pdf(pub Vec<u8>);

impl Pdf {
    pub fn from_data_url(data_url: &str) -> Result<Self> {
        let items: Vec<&str> = data_url.split(";base64,").collect();
        if items.len() != 2 {
            return Unexpected!("items {:?}", items);
        }

        Ok(Self(base64::decode(items[1])?))
    }
}
