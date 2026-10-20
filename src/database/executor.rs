use crate::args::Action;
use crate::database::{Item};
pub fn execute_action(items: &mut Vec<Item>, action: &Action) -> Result<bool, String> {
    match action {
        Action::LIST => {
            print_pretty(items);
        }
        Action::ADD { title, parent_id, description } => {
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
                parent_id.clone()
            ));
            execute_action(items, &Action::LIST)?;

            return Ok(true);
        }
        Action::DONE { ids , recursive, inverted} => {
            let found = items.iter().any(|item| ids.contains(&item.id()));
            if !found {
                return Err(
                    format!(
                        "Items with id not found: {:?}",
                        ids.iter().map(|item| item.to_string()).collect::<Vec<String>>()
                    )
                );
            }

            let mut targets: Vec<u32> = Vec::new();
            let mut children: Vec<(u32, usize)> = Vec::new();
            let marked_items = items.iter().filter(|item| ids.contains(&item.id())).collect::<Vec<&Item>>();
            for item in marked_items {
                children.clear();
                item.collect_children(items, &mut children);

                if ids.contains(&item.id()) {
                    targets.push(item.id());
                }
                if !!recursive && !children.is_empty() {
                    targets.extend(children.iter().map(|(id, _)| id));
                }

            }

            for item in items.iter_mut() {
                if !targets.contains(&item.id()) { continue;}
                item.done = !inverted;
            }

            execute_action(items, &Action::LIST)?;

            return Ok(true);
        }
        _ => todo!("Not implemented yet")
    }

    Ok(false)
}

fn print_pretty(items: &[Item]) {
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