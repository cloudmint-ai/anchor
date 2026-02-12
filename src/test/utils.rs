use crate::*;
use runtime::{fs::File, io::AsyncReadExt};

pub fn path(path: &str) -> String {
    format!("tests/data/{path}")
}

pub async fn read(sub_path: &str) -> Result<Vec<u8>> {
    let mut file = File::open(path(sub_path)).await?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).await?;
    Ok(data)
}

#[macro_export]
macro_rules! test_path {
    ($($arg:tt)*) => {{
        let path = format!($($arg)*);
        test::path(&path)
    }};
}

#[macro_export]
macro_rules! test_read {
    ($($arg:tt)*) => {{
        let path = format!($($arg)*);
        test::read(&path).await?
    }};
}
