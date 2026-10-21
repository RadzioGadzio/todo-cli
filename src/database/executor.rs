use crate::args::Action;
use crate::database::{Item};
use crate::database::item::{check_parent, print_pretty, check_ids, get_selected_ids};

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

            if let Some(parent_id) = parent_id {
                check_parent(items, parent_id.clone())?;
            }

            items.push(Item::new(
                max_id + 1,
                title.to_string(),
                description.clone(),
                parent_id.clone()
            ));
            execute_action(items, &Action::LIST)?;

            return Ok(true);
        },
        Action::REMOVE { ids, recursive, all } => {
            if *all {
                items.clear();
                execute_action(items, &Action::LIST)?;
                return Ok(true);
            }
            let mut ids = ids.clone();
            if ids.is_empty() {
                ids = items.iter().filter(|item| item.done).map(|item| item.id()).collect::<Vec<u32>>();
            }

            if ids.is_empty() {
                execute_action(items, &Action::LIST)?;
                return Ok(false);
            }
            check_ids(items, &ids)?;
            let targets: Vec<u32> = get_selected_ids(items, &ids, *recursive);

            items.retain(|item| !targets.contains(&item.id()));
            execute_action(items, &Action::LIST)?;

            return Ok(true);
        },
        Action::DONE { ids , recursive, inverted} => {
            if ids.is_empty() {
                return Err("Empty IDs not allowed".to_string());
            }
            check_ids(items, ids)?;
            let targets: Vec<u32> = get_selected_ids(items, ids, *recursive);

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


