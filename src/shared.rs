//! Some shared types

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct AppInfo {
    pub name: String,
    pub author_name: String,
    pub author_email: String,
    // This field has to be unique
    pub source_code_url: String,
    pub description: String,
    pub version: String,
}