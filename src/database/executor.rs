use crate::args::Action;
use crate::database::{Item};
use crate::database::item::{check_id, print_pretty, check_ids, get_selected_ids};

pub fn execute_action(items: &mut Vec<Item>, action: &Action) -> Result<bool, String> {
    match action {
        Action::List => {
            print_pretty(items);
        }
        Action::Add { title, parent_id, description } => {
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
                check_id(items, *parent_id)?;
            }

            items.push(Item::new(
                max_id + 1,
                title.to_string(),
                description.clone(),
                *parent_id
            ));
            execute_action(items, &Action::List)?;

            return Ok(true);
        },
        Action::Remove { ids, recursive, all } => {
            if *all {
                let items_count = items.len();
                items.clear();
                println!("Removed {} items", items_count);
                return Ok(true);
            }
            let mut ids = ids.clone();
            if ids.is_empty() {
                // Retrieve todos marked as done
                ids = items.iter().filter(|item| item.done).map(|item| item.id()).collect::<Vec<u32>>();
            }

            if ids.is_empty() {
                return Err("No items found".to_string());
            }
            check_ids(items, &ids)?;
            let targets: Vec<u32> = get_selected_ids(items, &ids, *recursive);

            // Remove items also defined in `targets`
            items.retain(|item| !targets.contains(&item.id()));
            execute_action(items, &Action::List)?;

            return Ok(true);
        },
        Action::Done { ids , recursive, inverted} => {
            if ids.is_empty() {
                return Err("Empty IDs not allowed".to_string());
            }
            check_ids(items, ids)?;
            let targets: Vec<u32> = get_selected_ids(items, ids, *recursive);

            // Update each `done` variable if its id is specified
            for item in items.iter_mut() {
                if !targets.contains(&item.id()) { continue; }
                item.done = !inverted;
            }

            execute_action(items, &Action::List)?;

            return Ok(true);
        },
        Action::Reset => {
            for item in items.iter_mut() {
                item.done = false;
            }
            execute_action(items, &Action::List)?;
            return Ok(true);
        }
        Action::Edit { id, title, description, parent_id} => {
            let mut parent_id: Option<u32> = *parent_id;

            // Check if `parent_id` is valid
            if let Some(id) = parent_id {
                check_id(items, id)?;
                parent_id = Some(id);
            }

            // Item to edit
            let item = match items.iter_mut().find(|item| &item.id() == id) {
                Some(item) => item,
                None => return Err(format!("Item with id '{id}' not found")),
            };

            if let Some(title) = title && !title.is_empty() { item.title = title.to_string(); }
            if let Some(description) = description && !description.is_empty() { item.description = Some(description.to_string()); }
            if let Some(parent_id) = parent_id { item.parent = Some(parent_id); }

            execute_action(items, &Action::List)?;

            return Ok(true);
        }
        _ => {}
    }

    Ok(false)
}


