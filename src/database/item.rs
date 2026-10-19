use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub title: String,
    pub parent: Option<u32>,
    pub description: Option<String>,
}
impl Item {
    pub const HEADERS: [&str; 4] = ["id", "title", "description", "parent"];

    pub fn new(id: u32, title: String, description: Option<String>, parent: Option<u32>) -> Item {
        Item {
            id,
            title,
            parent,
            description
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn pretty(&self, indent: &str, show_description: bool) {
        let mut line = format!("{}[{}] {}", indent, self.id, self.title);
        
        if show_description && let Some(desc) = &self.description {
        line.push_str(&format!(" ({})", desc));
        }

        print!("{}\n", line);
    }
}