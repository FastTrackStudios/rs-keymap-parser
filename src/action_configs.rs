use crate::action_list::ReaperActionList;
use camino::{Utf8Path, Utf8PathBuf};
use reaper_high::Reaper;
use std::fs;
use std::fs::File;
use std::io;

/// Load your keymap from  
///   <REAPER_RESOURCE_PATH>/data/FastTrackStudio/keymaps/ReaperKeyMap.conf
pub fn get_action_list_from_current_config(reaper: &Reaper) -> ReaperActionList {
    let reaper = Reaper::get();
    reaper
        .medium_reaper()
        .get_resource_path(|resource_path: &Utf8Path| {
            // 1) Construct: <resource_path>/data/FastTrackStudio/keymaps
            let keymap_dir: Utf8PathBuf = resource_path
                .join("data")
                .join("FastTrackStudio")
                .join("keymaps");

            // 2) Make sure the directory exists
            if let Err(e) = fs::create_dir_all(&keymap_dir) {
                eprintln!(
                    "⚠️  Could not create keymap directory at {:?}: {}",
                    keymap_dir, e
                );
                // Even if mkdir failed, try to proceed to load (it’ll error out below)
            }

            // 3) Append the filename you actually want to load
            let keymap_file = keymap_dir.join("default.reaperkeymap");

            if !keymap_file.exists() {
                match File::create(&keymap_file) {
                    Ok(_) => println!("✨ Created new keymap file at {:?}", keymap_file),
                    Err(e) => eprintln!("⚠️  Failed to create {:?}: {}", keymap_file, e),
                }
            }

            // 4) Try to load it, or fall back to an empty list on any I/O error
            match ReaperActionList::load_from_file(keymap_file.as_std_path()) {
                Ok(list) => {
                    println!("✔️ Loaded {} entries from {:?}", list.0.len(), keymap_file);
                    list
                }
                Err(e) => {
                    eprintln!("⚠️ Failed to load keymap from {:?}: {}", keymap_file, e);
                    ReaperActionList(Vec::new())
                }
            }
        })
}

