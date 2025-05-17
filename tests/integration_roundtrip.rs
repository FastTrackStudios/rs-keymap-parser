// tests/integration_roundtrip.rs

use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::{env, fs};

// Adjust the import path below to your crate’s root module name:
use rs_keymap_parser::action_list::ReaperActionList;

/// Reads all lines from `path` into a Vec<String>, preserving line order exactly.
fn read_lines<P: AsRef<std::path::Path>>(path: P) -> io::Result<Vec<String>> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

#[test]
fn integration_roundtrip_large_file() {
    // 1) Locate the original keymap in `resources/LargeIntegrationTest.reaperkeymap`
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut orig_path = PathBuf::from(manifest_dir);
    orig_path.push("resources");
    orig_path.push("LargeIntegrationTest.reaperkeymap");
    assert!(
        orig_path.exists(),
        "Resource file not found: {:?}",
        orig_path
    );

    // 2) Prepare target directory under `<manifest>/target/`
    let mut target_dir = PathBuf::from(manifest_dir);
    target_dir.push("target");
    fs::create_dir_all(&target_dir).expect("Failed to create target directory");

    let mut json_path = target_dir.clone();
    json_path.push("largeintegration.json");

    let mut rt_path = target_dir;
    rt_path.push("largeintegration_rt.reaperkeymap");

    // 3) Load and parse original keymap
    let list =
        ReaperActionList::load_from_file(&orig_path).expect("Failed to load original keymap");

    // 4) Serialize to JSON
    let json_file = fs::File::create(&json_path).expect("Failed to create JSON file");
    serde_json::to_writer_pretty(json_file, &list).expect("JSON serialization failed");

    // 5) Deserialize back from JSON
    let json_file = fs::File::open(&json_path).expect("Failed to open JSON file");
    let list2: ReaperActionList =
        serde_json::from_reader(json_file).expect("JSON deserialization failed");

    // 6) Save round-tripped keymap
    list2
        .save_to_file(&rt_path)
        .expect("Failed to save round-trip keymap");

    // 7) Compare files line-by-line, ignoring any trailing comments
    let orig_lines = read_lines(&orig_path).expect("Failed to read original lines");
    let rt_lines = read_lines(&rt_path).expect("Failed to read round-trip lines");

    for (i, (o, r)) in orig_lines.iter().zip(&rt_lines).enumerate() {
        // Strip anything after '#' and trim trailing whitespace
        let o_clean = o.split('#').next().unwrap().trim_end();
        let r_clean = r.split('#').next().unwrap().trim_end();

        assert_eq!(
            o_clean,
            r_clean,
            "Mismatch at line {}: orig=`{}`, rt=`{}`",
            i + 1,
            o_clean,
            r_clean
        );
    }

    // Also ensure no extra lines at end
    assert_eq!(
        orig_lines.len(),
        rt_lines.len(),
        "Line count differs: orig={} vs rt={}",
        orig_lines.len(),
        rt_lines.len()
    );
}
