use std::fs;
use std::path::PathBuf;
use crate::database::Item;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum SetupItems {
    RecursiveDepth1,
    RecursiveDepth2,
    Flat
}
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn setup_ids(setup: SetupItems) -> Vec<u32> {
    setup_items(setup).iter().map(|item|item.id()).collect::<Vec<u32>>()
}

pub struct TempDir(PathBuf);

impl TempDir {
    #[allow(dead_code)]
    pub fn new(name: &str) -> Self {
        let path = std::env::temp_dir()
            .join(format!("todo_cli_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        TempDir(path)
    }
    #[allow(dead_code)]
    pub fn path(&self) -> &PathBuf {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}