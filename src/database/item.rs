use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    id: u32,
    pub title: String,
    pub parent: Option<u32>,
    pub description: Option<String>,
    pub done: bool,
}

impl Item {
    pub const HEADERS: [&str; 5] = ["id", "title", "description", "parent", "done"];

    pub fn new(id: u32, title: String, description: Option<String>, parent: Option<u32>) -> Item {
        Item {
            id,
            title,
            parent,
            description,
            done: false
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn print_pretty(&self, indent: usize) {
        let mut line = format!("{}[{}] {} {}",
                               " ".repeat(indent),
                               self.id,
                               if self.done { "✅" } else { "❌" },
                               self.title);

        if let Some(desc) = &self.description {
            line.push_str(&format!(" - {}", desc));
        }

        print!("{}\n", line);
    }

    pub fn collect_children(&self, items: &[Item], out: &mut Vec<(u32, usize)>) {
        Item::collect_tree(items, self, 0, out);
    }
    fn collect_tree(items: &[Item], node: &Item, depth: usize, out: &mut Vec<(u32, usize)>) {
        out.push((node.id, depth));
        for item in items {
            if item.parent == Some(node.id()) &&
                !out.iter().any(|(id, _)| id == &item.id)
            {
                Item::collect_tree(items, item, depth + 1, out);
            }
        }
    }
}