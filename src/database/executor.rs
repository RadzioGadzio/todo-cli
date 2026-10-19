use crate::args::Action;
use crate::database::{Item};
pub fn execute_action(items: &mut Vec<Item>, action: &Action) -> Result<bool, String> {
    match action {
        Action::LIST { description } => {
            print_pretty(items, *description);
        }
        Action::ADD { title, parent, description } => {
            if title.trim().is_empty() {
                return Err("Title cannot be empty".to_string());
            }

            let mut max_id = 0;
            for item in items.iter() {
              if item.id() > max_id {
                  max_id = item.id()
              }
            }
            items.push(Item::new(
                max_id + 1,
                title.to_string(),
                description.clone(),
                *parent
            ));
            execute_action(items, &Action::LIST { description: false })?;
            
            return Ok(true);
        }
        _ => todo!("Not implemented yet")
    }

    Ok(false)
}

fn print_pretty(items: &Vec<Item>, show_description: bool) {
    for item in items {
        if item.parent.is_some() {
            continue;
        }
        print_tree(items, item, 0, show_description);
    }
}

fn print_tree(items: &[Item], item: &Item, depth: usize, show_description: bool) {
    let indent = " ".repeat(depth * 4);
    item.pretty(&indent, show_description);

    for child in items {
        if child.parent != Some(item.id) {
            continue;
        }
        print_tree(items, child, depth + 1, show_description);
    }
}