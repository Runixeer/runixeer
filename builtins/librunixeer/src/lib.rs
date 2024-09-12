use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ListItem {
    pub index: u64,
    pub name: String,
    pub subitems: Option<Vec<SubListItem>>,
}

#[derive(Debug,Deserialize,Serialize)]
pub struct SubListItem {
    pub index: u64,
    pub name: String,
}

impl ListItem {
    pub fn new(index: u64, name: String) -> Self {
        ListItem {
            index,
            name,
            subitems: None,
        }
    }
    pub fn with_subitems(index: u64, name: String, subitems: Vec<SubListItem>) -> Self {
        ListItem {
            index,
            name,
            subitems: Some(subitems),
        }
    }
}

impl SubListItem {
    pub fn new(index: u64, name: String) -> SubListItem {
        SubListItem { index, name }
    }
}
