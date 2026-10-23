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

        println!("{}", line);
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

pub fn check_id(items: &[Item], item_id: u32) -> Result<(), String> {
    let valid = items.iter().find(|item| item_id == item.id()).is_some();

    if !valid {
        return Err(format!("Invalid item ID: {}", item_id));
    }

    Ok(())
}


pub fn check_ids(items: &[Item], ids: &[u32]) -> Result<(), String> {
    let found = ids.iter().all(|id| *(&items.iter().any(|item| item.id() == *id)));
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

        if ids.contains(&item.id()) && !targets.contains(&item.id()) {
            targets.push(item.id());
        }
        if !!recursive && !children.is_empty() {
            targets.extend(children.iter().map(|(id, _)| id));
        }

    }

    targets
}

#[cfg(test)]
mod tests {
    use crate::tests;
    use crate::tests::SetupItems;
    use super::*;

    #[test]
    fn _item_collect_children() {
        let items = tests::setup_items(SetupItems::RecursiveDepth2);
        let root_item = &items[0];

        let mut children: Vec<(u32, usize)> = vec![];

        root_item.collect_children(&items, &mut children);
        assert!(!children.is_empty(), "Expected items to be collected. Root: {:?}, children: {:?}", root_item, children);
        assert!(children.contains(&(0, 0)), "Don't contain root element. Root: {:?}, children: {:?}", root_item, children);
        assert!(children.contains(&(1, 1)), "Don't contain item with ID=1 at depth=1. Root: {:?}, children: {:?}", root_item, children);
        assert!(children.contains(&(3, 1)), "Don't contain item with ID=3 at depth=1. Root: {:?}, children: {:?}", root_item, children);
        assert!(children.contains(&(2, 2)), "Don't contain item with ID=2 at depth=2. Root: {:?}, children: {:?}", root_item, children);
    }
    #[test]
    fn _check_id() {
        let items = tests::setup_items(SetupItems::Flat);

        assert!(check_id(&items, 0).is_ok(), "Item not found");
        assert!(check_id(&items, 1000).is_err(), "Item found while it shouldn't");

    }
    #[test]
    fn _check_ids() {
        let items = tests::setup_items(SetupItems::Flat);
        let mut ids = vec![1, 2];

        assert!(check_ids(&items, &ids).is_ok(), "Doesn't return Ok. Item ids: {:?}, Ids: {:?}", tests::setup_ids(SetupItems::Flat), ids);

        ids.push(1000);

        assert!(check_ids(&items, &ids).is_err(), "Doesn't return an error. Items ids: {:?}, Ids: {:?}", tests::setup_ids(SetupItems::Flat), ids);


    }
    #[test]
    fn _get_selected_ids() {
        let items = tests::setup_items(SetupItems::RecursiveDepth1);
        let mut ids = vec![];

        let selected_ids = get_selected_ids(&items, &ids, false);

        assert!(selected_ids.is_empty(), "Returned something while shouldn't. Ids: {:?}, selected ids: {:?}", ids, selected_ids);

        ids.push(0);
        let selected_ids = get_selected_ids(&items, &ids, false);
        assert!(selected_ids.iter().all(|id| ids.contains(id)), "Doesn't return valid ids: {:?}, returned {:?}", ids, selected_ids);

        let selected_ids = get_selected_ids(&items, &ids, true);

        ids.extend([1, 2]);
        assert!(selected_ids.iter().all(|id| ids.contains(id)), "Doesn't return valid recursive ids: {:?}, returned {:?}", ids, selected_ids);
    }
}