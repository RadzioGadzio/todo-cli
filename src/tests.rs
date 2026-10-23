use crate::database::Item;

#[derive(Copy, Clone)]
pub enum SetupItems {
    RecursiveDepth1,
    RecursiveDepth2,
    Flat
}
pub fn setup_items(setup: SetupItems) -> Vec<Item> {
    let t = |id: u32| format!("title{id}");
    match setup {
        // 0 -> {1, 2}
        SetupItems::RecursiveDepth1 => vec![
            Item::new(0, t(0), None, None),
            Item::new(1, t(1), None, Some(0)),
            Item::new(2, t(2), None, Some(0)),
        ],
        // 0 -> {1 -> 2, 3}; 4 standalone
        SetupItems::RecursiveDepth2 => vec![
            Item::new(0, t(0), None, None),
            Item::new(1, t(1), None, Some(0)),
            Item::new(2, t(2), None, Some(1)),
            Item::new(3, t(3), None, Some(0)),
            Item::new(4, t(4), None, None),
        ],
        // no parents
        SetupItems::Flat => vec![
            Item::new(0, t(0), None, None),
            Item::new(1, t(1), None, None),
            Item::new(2, t(2), Some("description".to_string()), None),
        ]
    }
}
pub fn setup_ids(setup: SetupItems) -> Vec<u32> {
    setup_items(setup).iter().map(|item|item.id()).collect::<Vec<u32>>()
}