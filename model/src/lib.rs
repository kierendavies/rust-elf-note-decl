use serde::{Deserialize, Serialize};

pub mod note;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
    pub an_int: i32,
    pub some_strings: Vec<String>,
}
