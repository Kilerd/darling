use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize,Debug)]
pub struct Config {
    pub password: String,
    pub links: Vec<NoteLink>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NoteLink {
    pub title: String,
    pub name: String,
    pub create_at: DateTime<Utc>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NoteDetail {
    pub title: String,
    pub content: String,
    pub create_at: DateTime<Utc>
}

impl Config {
    pub fn from_raw(s:String) -> Self {
        toml::from_str(&s).expect("cannot deserialize model")
    }
}