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

pub fn print_pretty(items: &[Item]) {
    let root_items = items.iter().filter(|item| item.parent.is_none()).collect::<Vec<&Item>>();
    let mut targets: Vec<(u32, usize)> = Vec::new();
    let mut children: Vec<(u32, usize)> = Vec::new();

    for item in root_items {
        children.clear();
        item.collect_children(items, &mut children);
        if children.is_empty() { continue; }
        targets.extend(children.iter());
    }

    for (id, depth) in targets {
        let item = items.iter().find(|item| item.id() == id);
        if let Some(item) = item {
            item.print_pretty(depth * 4);
        }
    }
}

pub fn check_parent(items: &[Item], parent_id: u32) -> Result<(), String> {
    let valid_parent = items.iter().find(|item| parent_id == item.id()).is_some();

    if !valid_parent {
        return Err(format!("Invalid parent ID: {}", parent_id));
    }

    Ok(())
}


pub fn check_ids(items: &[Item], ids: &[u32]) -> Result<(), String> {
    let found = items.iter().any(|item| ids.contains(&item.id()));
    if !found {
        return Err(
            format!(
                "Items with id not found: {:?}",
                ids.iter().map(|item| item.to_string()).collect::<Vec<String>>()
            )
        );
    }
    Ok(())
}

pub fn get_selected_ids(items: &[Item], ids: &[u32], recursive: bool) -> Vec<u32> {
    let mut targets: Vec<u32> = Vec::new();
    let mut children: Vec<(u32, usize)> = Vec::new();
    let selected_items = items.iter().filter(|item| ids.contains(&item.id())).collect::<Vec<&Item>>();
    for item in selected_items {
        children.clear();
        item.collect_children(items, &mut children);

        if ids.contains(&item.id()) {
            targets.push(item.id());
        }
        if !!recursive && !children.is_empty() {
            targets.extend(children.iter().map(|(id, _)| id));
        }

    }

    targets
}