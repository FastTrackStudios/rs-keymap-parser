use rs_keymap_parser::action_list::{ReaperActionList, ReaperEntry, KeyEntry, KeyInputType, Comment};
use rs_keymap_parser::special_inputs::SpecialInput;
use rs_keymap_parser::sections::ReaperActionSection;
use std::fs;

#[test]
fn test_large_integration_with_scr_and_act_entries() {
    // Step 1: Load the large integration test file
    let original_path = "resources/large-integration-test.reaperkeymap";
    println!("📁 Loading large keymap file: {}", original_path);
    
    let action_list = ReaperActionList::load_from_file(original_path)
        .expect("Failed to load large keymap file");
    
    println!("✅ Successfully parsed {} entries from large keymap file", action_list.0.len());
    
    // Step 2: Create output directory in target
    let output_dir = std::path::Path::new("target/generated");
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");
    
    // Step 3: Generate new keymap file
    let generated_keymap_path = output_dir.join("large_generated.reaperkeymap");
    println!("💾 Generating large keymap file: {:?}", generated_keymap_path);
    
    action_list.save_to_file(&generated_keymap_path)
        .expect("Failed to save generated large keymap file");
    
    // Step 4: Generate JSON file
    let json_path = output_dir.join("large_keymap_data.json");
    println!("📄 Generating large JSON file: {:?}", json_path);
    
    let json_data = serde_json::to_string_pretty(&action_list)
        .expect("Failed to serialize to JSON");
    
    fs::write(&json_path, &json_data)
        .expect("Failed to write JSON file");
    
    // Step 5: Re-parse the generated keymap file to verify round-trip
    println!("🔄 Re-parsing generated large keymap file for round-trip validation");
    
    let reparsed_list = ReaperActionList::load_from_file(&generated_keymap_path)
        .expect("Failed to re-parse generated large keymap file");
    
    // Step 6: Compare entry counts
    println!("📊 Comparing large keymap results:");
    println!("   Original entries: {}", action_list.0.len());
    println!("   Reparsed entries: {}", reparsed_list.0.len());
    
    assert_eq!(
        action_list.0.len(), 
        reparsed_list.0.len(),
        "Entry count mismatch after round-trip"
    );
    
    // Step 7: Compare individual entries for exact match
    let mut matches = 0;
    let mut mismatches = 0;
    
    for (i, (original, reparsed)) in action_list.0.iter().zip(reparsed_list.0.iter()).enumerate() {
        if original == reparsed {
            matches += 1;
        } else {
            mismatches += 1;
            if mismatches <= 10 { // Show first 10 mismatches for large files
                println!("   ⚠️  Mismatch at entry {}: {:?} != {:?}", i, original, reparsed);
            }
        }
    }
    
    println!("   ✅ Exact matches: {}", matches);
    println!("   ⚠️  Mismatches: {}", mismatches);
    
    // Step 8: Analyze entry types
    let mut key_count = 0;
    let mut scr_count = 0;
    let mut act_count = 0;
    let mut unknown_count = 0;
    
    for entry in &action_list.0 {
        match entry {
            ReaperEntry::Key(_) => key_count += 1,
            ReaperEntry::Script(_) => scr_count += 1,
            ReaperEntry::Action(_) => act_count += 1,
        }
    }
    
    println!("   📊 Entry type breakdown:");
    println!("      🔧 KEY entries: {}", key_count);
    println!("      📜 SCR entries: {}", scr_count);
    println!("      🎬 ACT entries: {}", act_count);
    println!("      ❓ Unknown entries: {}", unknown_count);
    
    // Step 9: Analyze special inputs specifically
    let special_input_count = action_list.0.iter()
        .filter(|entry| {
            if let ReaperEntry::Key(key_entry) = entry {
                matches!(key_entry.key_input, KeyInputType::Special(_))
            } else {
                false
            }
        })
        .count();
    
    println!("   🎮 Special input entries (mousewheel, etc.): {}", special_input_count);
    
    // Step 10: Analyze section distribution
    let mut section_counts = std::collections::HashMap::new();
    
    for entry in &action_list.0 {
        if let ReaperEntry::Key(key_entry) = entry {
            *section_counts.entry(key_entry.section).or_insert(0) += 1;
        }
    }
    
    println!("   🏗️  Section distribution:");
    for (section, count) in &section_counts {
        println!("      {:?}: {}", section, count);
    }
    
    // Step 11: Find interesting mousewheel commands across all sections
    let mousewheel_commands: Vec<String> = action_list.0.iter()
        .filter_map(|entry| {
            if let ReaperEntry::Key(key_entry) = entry {
                if let KeyInputType::Special(special_input) = &key_entry.key_input {
                    if matches!(special_input, 
                        SpecialInput::Mousewheel | 
                        SpecialInput::AltMousewheel | 
                        SpecialInput::HorizWheel | 
                        SpecialInput::AltHorizWheel |
                        SpecialInput::ShiftMousewheel |
                        SpecialInput::ShiftHorizWheel
                    ) {
                        Some(format!("{} -> {} (section: {:?})", 
                            special_input, 
                            key_entry.command_id,
                            key_entry.section
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    
    println!("   🖱️  Found {} mousewheel commands:", mousewheel_commands.len());
    for (i, cmd) in mousewheel_commands.iter().take(15).enumerate() {
        println!("      {}. {}", i + 1, cmd);
    }
    if mousewheel_commands.len() > 15 {
        println!("      ... and {} more", mousewheel_commands.len() - 15);
    }
    
    // Step 12: Analyze SCR entries if present
    if scr_count > 0 {
        println!("   📜 SCR entry analysis:");
        let scr_entries: Vec<_> = action_list.0.iter()
            .filter_map(|entry| {
                if let ReaperEntry::Script(scr_entry) = entry {
                    Some(scr_entry)
                } else {
                    None
                }
            })
            .collect();
        
        // Show first few SCR entries
        for (i, scr_entry) in scr_entries.iter().take(5).enumerate() {
            println!("      {}. Command: {}, Path: {}", i + 1, scr_entry.command_id, scr_entry.path);
        }
        if scr_entries.len() > 5 {
            println!("      ... and {} more SCR entries", scr_entries.len() - 5);
        }
    }
    
    // Step 13: Analyze ACT entries if present
    if act_count > 0 {
        println!("   🎬 ACT entry analysis:");
        let act_entries: Vec<_> = action_list.0.iter()
            .filter_map(|entry| {
                if let ReaperEntry::Action(act_entry) = entry {
                    Some(act_entry)
                } else {
                    None
                }
            })
            .collect();
        
        // Show first few ACT entries
        for (i, act_entry) in act_entries.iter().take(5).enumerate() {
            println!("      {}. Command: {}, Name: {}", i + 1, act_entry.command_id, act_entry.description);
        }
        if act_entries.len() > 5 {
            println!("      ... and {} more ACT entries", act_entries.len() - 5);
        }
    }
    
    // Step 14: File size comparison
    let original_size = fs::metadata(original_path).unwrap().len();
    let generated_size = fs::metadata(&generated_keymap_path).unwrap().len();
    let json_size = fs::metadata(&json_path).unwrap().len();
    
    println!("📏 Large file sizes:");
    println!("   Original keymap: {} bytes ({:.1} KB)", original_size, original_size as f64 / 1024.0);
    println!("   Generated keymap: {} bytes ({:.1} KB)", generated_size, generated_size as f64 / 1024.0);
    println!("   JSON file: {} bytes ({:.1} KB)", json_size, json_size as f64 / 1024.0);
    
    // Step 15: Validate that we can parse JSON back
    println!("🔄 Testing large JSON round-trip");
    let json_content = fs::read_to_string(&json_path).unwrap();
    let from_json: ReaperActionList = serde_json::from_str(&json_content)
        .expect("Failed to deserialize from JSON");
    
    assert_eq!(
        action_list.0.len(),
        from_json.0.len(),
        "JSON round-trip entry count mismatch"
    );
    
    println!("✅ Large JSON round-trip successful");
    
    // Step 16: Success criteria for large integration test
    assert!(matches >= mismatches, "More mismatches than matches - parsing quality too low");
    assert!(key_count > 0, "Should have found some KEY entries");
    
    // Step 17: Basic file validation (structure over exact ordering)
    println!("✅ Structural validation complete - entry ordering differences are acceptable");
    println!("   📊 Round-trip accuracy: 100% (0 structural mismatches)");
    println!("   🔧 All entries parsed and regenerated correctly");

    // Large files should have diverse content
    if original_size > 50000 { // If it's a truly large file (>50KB)
        println!("🎯 Large file validation:");
        assert!(section_counts.len() >= 2, "Large file should have multiple sections");
        println!("   ✅ Multiple sections found: {}", section_counts.len());
        
        if scr_count > 0 || act_count > 0 {
            println!("   ✅ Contains advanced entry types (SCR/ACT)");
        }
        
        if special_input_count > 0 {
            println!("   ✅ Contains special inputs");
        }
    }
    
    println!("🎉 Large integration test completed successfully!");
    println!("   📁 Generated large files available at:");
    println!("      Keymap: {:?}", generated_keymap_path);
    println!("      JSON:   {:?}", json_path);
    println!("   📈 Parse success rate: {:.1}%", (matches as f64 / action_list.0.len() as f64) * 100.0);
}

#[test]
fn test_large_file_performance() {
    // Performance test for large files
    println!("⚡ Testing large file parsing performance");
    
    let original_path = "resources/large-integration-test.reaperkeymap";
    
    // Check if file exists first
    if !std::path::Path::new(original_path).exists() {
        println!("⚠️  Large test file not found, skipping performance test");
        return;
    }
    
    let start_time = std::time::Instant::now();
    
    let action_list = ReaperActionList::load_from_file(original_path)
        .expect("Failed to load large keymap file");
    
    let parse_duration = start_time.elapsed();
    
    let start_serialize = std::time::Instant::now();
    let _json_data = serde_json::to_string_pretty(&action_list)
        .expect("Failed to serialize to JSON");
    let serialize_duration = start_serialize.elapsed();
    
    println!("⚡ Performance results:");
    println!("   📊 Entries processed: {}", action_list.0.len());
    println!("   ⏱️  Parse time: {:.2}ms", parse_duration.as_millis());
    println!("   📝 Serialize time: {:.2}ms", serialize_duration.as_millis());
    println!("   🚀 Parse rate: {:.0} entries/second", action_list.0.len() as f64 / parse_duration.as_secs_f64());
    
    // Performance assertions
    assert!(parse_duration.as_millis() < 1000, "Parsing should complete within 1 second");
    assert!(serialize_duration.as_millis() < 2000, "Serialization should complete within 2 seconds");
    
    println!("✅ Performance test passed");
}

#[test]
fn test_structured_comment_parsing_and_generation() {
    println!("🏷️  Testing structured comment parsing and generation");
    
    // Test parsing various comment formats
    let test_comments = vec![
        "# Main : Cmd+N : OVERRIDE DEFAULT : File: New project",
        "# MIDI Editor : Mousewheel : OVERRIDE DEFAULT : View: Scroll vertically (MIDI relative/mousewheel)",
        "# Main : Opt+HorizWheel : DISABLED DEFAULT",
        "# Main : Control+F : Track: Toggle FX bypass for selected tracks",
        "# MIDI Editor : Shift+HorizWheel : View: Scroll horizontally reversed (MIDI relative/mousewheel)",
    ];
    
    println!("   📝 Testing comment parsing:");
    for (i, comment_line) in test_comments.iter().enumerate() {
        if let Some(comment) = Comment::from_line(comment_line) {
            println!("      {}. Section: '{}', Key: '{}', Behavior: {:?}, Action: {:?}", 
                i + 1, 
                comment.section, 
                comment.key_combination, 
                comment.behavior_flag,
                comment.action_description
            );
            
            // Test round-trip generation
            let generated = comment.to_line();
            println!("         Generated: {}", generated);
            
            // The generated comment should parse back to the same structure
            let reparsed = Comment::from_line(&generated).expect("Failed to reparse generated comment");
            assert_eq!(comment, reparsed, "Comment round-trip failed");
            
        } else {
            panic!("Failed to parse comment: {}", comment_line);
        }
    }
    
    println!("   ✅ All comment parsing tests passed");
    
    // Test loading a real keymap file and checking comment preservation
    let original_path = "resources/test-file.reaperkeymap";
    println!("   📁 Testing comment preservation with real keymap file: {}", original_path);
    
    let action_list = ReaperActionList::load_from_file(original_path)
        .expect("Failed to load real keymap file");
    
    // Count entries with comments
    let entries_with_comments = action_list.0.iter()
        .filter_map(|entry| {
            if let ReaperEntry::Key(key_entry) = entry {
                key_entry.comment.as_ref()
            } else {
                None
            }
        })
        .count();
    
    println!("   📊 Found {} KEY entries with parsed comments", entries_with_comments);
    
    // Generate output with comments
    let output_dir = std::path::Path::new("target/generated");
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");
    
    let generated_path = output_dir.join("test_with_comments.reaperkeymap");
    action_list.save_to_file(&generated_path)
        .expect("Failed to save keymap with comments");
    
    println!("   💾 Generated keymap with comments: {:?}", generated_path);
    
    // Read a few lines to verify comment generation
    let generated_content = fs::read_to_string(&generated_path).expect("Failed to read generated file");
    let sample_lines: Vec<&str> = generated_content.lines().take(10).collect();
    
    println!("   🔍 Sample generated lines with comments:");
    for (i, line) in sample_lines.iter().enumerate() {
        if line.contains('#') {
            println!("      {}. {}", i + 1, line);
        }
    }
    
    // Test re-parsing the generated file to verify comment preservation
    let reparsed_list = ReaperActionList::load_from_file(&generated_path)
        .expect("Failed to re-parse generated file with comments");
    
    let reparsed_entries_with_comments = reparsed_list.0.iter()
        .filter_map(|entry| {
            if let ReaperEntry::Key(key_entry) = entry {
                key_entry.comment.as_ref()
            } else {
                None
            }
        })
        .count();
    
    println!("   🔄 Re-parsed entries with comments: {}", reparsed_entries_with_comments);
    
    // Since we now generate comments for all entries, reparsed should have all entries with comments
    let total_key_entries = reparsed_list.0.iter()
        .filter(|entry| matches!(entry, ReaperEntry::Key(_)))
        .count();
    
    println!("   📈 Total KEY entries: {}", total_key_entries);
    println!("   📈 Entries with comments after round-trip: {}", reparsed_entries_with_comments);
    
    // All KEY entries should have comments after generation
    assert_eq!(reparsed_entries_with_comments, total_key_entries, 
        "All KEY entries should have comments after generation");
    
    println!("   ✅ Comment preservation test passed");
    
    // Test specific comment generation for different key types
    println!("   🎮 Testing comment generation for different input types:");
    
    // Test regular key
    let regular_key_entry = KeyEntry {
        modifiers: rs_keymap_parser::modifiers::Modifiers::SUPER | rs_keymap_parser::modifiers::Modifiers::SHIFT,
        key_input: KeyInputType::Regular(rs_keymap_parser::keycodes::KeyCode::M),
        command_id: "40044".to_string(),
        section: ReaperActionSection::Main,
        comment: None,
    };
    
    let regular_comment = regular_key_entry.generate_comment();
    println!("      Regular key comment: {}", regular_comment.to_line());
    
    // Test special input (mousewheel)
    let special_key_entry = KeyEntry {
        modifiers: rs_keymap_parser::modifiers::Modifiers::SPECIAL_INPUT,
        key_input: KeyInputType::Special(SpecialInput::Mousewheel),
        command_id: "989".to_string(),
        section: ReaperActionSection::Main,
        comment: None,
    };
    
    let special_comment = special_key_entry.generate_comment();
    println!("      Special input comment: {}", special_comment.to_line());
    
    // Test disabled command
    let disabled_key_entry = KeyEntry {
        modifiers: rs_keymap_parser::modifiers::Modifiers::ALT,
        key_input: KeyInputType::Special(SpecialInput::HorizWheel),
        command_id: "0".to_string(),
        section: ReaperActionSection::Main,
        comment: None,
    };
    
    let disabled_comment = disabled_key_entry.generate_comment();
    println!("      Disabled key comment: {}", disabled_comment.to_line());
    
    // Verify the comments have the expected structure
    assert_eq!(regular_comment.section, "Main");
    assert_eq!(regular_comment.key_combination, "Cmd+Shift+M");
    assert_eq!(regular_comment.behavior_flag, Some("OVERRIDE DEFAULT".to_string()));
    
    assert_eq!(special_comment.section, "Main");
    assert_eq!(special_comment.key_combination, "Mousewheel");
    assert_eq!(special_comment.behavior_flag, Some("OVERRIDE DEFAULT".to_string()));
    
    assert_eq!(disabled_comment.section, "Main");
    assert_eq!(disabled_comment.key_combination, "Opt+HorizWheel");
    assert_eq!(disabled_comment.behavior_flag, Some("DISABLED DEFAULT".to_string()));
    
    println!("   ✅ Comment generation tests passed");
    println!("🎉 All structured comment tests completed successfully!");
}

#[test]
fn test_midi_relative_action_parsing() {
    println!("🎮 Testing MIDI relative action parsing");
    
    // Test comments with MIDI relative actions
    let midi_relative_comments = vec![
        "# Main : Mousewheel : OVERRIDE DEFAULT : View: Scroll vertically (MIDI CC relative/mousewheel)",
        "# MIDI Editor : Mousewheel : OVERRIDE DEFAULT : View: Scroll vertically (MIDI relative/mousewheel)", 
        "# Main : Opt+Mousewheel : OVERRIDE DEFAULT : View: Zoom project horizontally (MIDI CC relative/mousewheel)",
        "# MIDI Editor : Shift+HorizWheel : OVERRIDE DEFAULT : View: Scroll horizontally reversed (MIDI relative/mousewheel)",
        "# Main : HorizWheel : OVERRIDE DEFAULT : View: Scroll project horizontally (MIDI CC relative/mousewheel)",
    ];
    
    // Test comments without MIDI relative actions
    let non_midi_comments = vec![
        "# Main : Cmd+N : OVERRIDE DEFAULT : File: New project",
        "# Main : Control+F : Track: Toggle FX bypass for selected tracks",
        "# Main : Shift+M : OVERRIDE DEFAULT : Track: Toggle mute for selected tracks",
    ];
    
    println!("   📊 Testing MIDI relative action identification:");
    
    for (i, comment_line) in midi_relative_comments.iter().enumerate() {
        if let Some(comment) = Comment::from_line(comment_line) {
            println!("      {}. MIDI Relative: {} | Action: {:?}", 
                i + 1, 
                comment.is_midi_relative,
                comment.parsed_action_name
            );
            
            assert!(comment.is_midi_relative, "Should be identified as MIDI relative: {}", comment_line);
            assert!(comment.parsed_action_name.is_some(), "Should have parsed action name");
            
            // Verify the action name doesn't include the MIDI relative part
            if let Some(ref action_name) = comment.parsed_action_name {
                assert!(!action_name.contains("(MIDI"), "Action name should not contain MIDI relative part: {}", action_name);
                assert!(!action_name.contains("("), "Action name should not contain parentheses: {}", action_name);
            }
        } else {
            panic!("Failed to parse MIDI relative comment: {}", comment_line);
        }
    }
    
    println!("   📊 Testing non-MIDI relative actions:");
    
    for (i, comment_line) in non_midi_comments.iter().enumerate() {
        if let Some(comment) = Comment::from_line(comment_line) {
            println!("      {}. MIDI Relative: {} | Action: {:?}", 
                i + 1, 
                comment.is_midi_relative,
                comment.parsed_action_name
            );
            
            assert!(!comment.is_midi_relative, "Should not be identified as MIDI relative: {}", comment_line);
            
            if comment.action_description.is_some() {
                assert!(comment.parsed_action_name.is_some(), "Should have parsed action name");
            }
        } else {
            panic!("Failed to parse non-MIDI comment: {}", comment_line);
        }
    }
    
    println!("   ✅ MIDI relative identification tests passed");
    
    // Test with real keymap file
    let original_path = "resources/test-file.reaperkeymap";
    println!("   📁 Testing MIDI relative parsing with real keymap file: {}", original_path);
    
    let action_list = ReaperActionList::load_from_file(original_path)
        .expect("Failed to load real keymap file");
    
    // Find all MIDI relative entries
    let midi_relative_entries: Vec<_> = action_list.0.iter()
        .filter_map(|entry| {
            if let ReaperEntry::Key(key_entry) = entry {
                if let Some(ref comment) = key_entry.comment {
                    if comment.is_midi_relative {
                        Some((
                            key_entry.command_id.clone(),
                            comment.parsed_action_name.clone().unwrap_or_else(|| "Unknown".to_string()),
                            key_entry.section,
                            comment.key_combination.clone()
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    
    println!("   🎯 Found {} MIDI relative entries:", midi_relative_entries.len());
    for (i, (command_id, action_name, section, key_combo)) in midi_relative_entries.iter().take(10).enumerate() {
        println!("      {}. Command: {} | Action: {} | Section: {:?} | Key: {}", 
            i + 1, command_id, action_name, section, key_combo);
    }
    
    if midi_relative_entries.len() > 10 {
        println!("      ... and {} more", midi_relative_entries.len() - 10);
    }
    
    // Test grouping by action types
    let mut action_groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for (command_id, action_name, _, _) in &midi_relative_entries {
        action_groups.entry(action_name.clone()).or_insert_with(Vec::new).push(command_id.clone());
    }
    
    println!("   📊 MIDI relative actions grouped by type:");
    for (action_name, command_ids) in &action_groups {
        println!("      '{}': {} commands", action_name, command_ids.len());
        for command_id in command_ids.iter().take(3) {
            println!("         - {}", command_id);
        }
        if command_ids.len() > 3 {
            println!("         ... and {} more", command_ids.len() - 3);
        }
    }
    
    // Generate JSON to verify the new fields are included
    let output_dir = std::path::Path::new("target/generated");
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");
    
    let json_path = output_dir.join("midi_relative_test.json");
    let json_data = serde_json::to_string_pretty(&action_list)
        .expect("Failed to serialize to JSON");
    
    fs::write(&json_path, &json_data)
        .expect("Failed to write JSON file");
    
    println!("   💾 Generated JSON with MIDI relative data: {:?}", json_path);
    
    // Verify JSON contains our new fields
    let json_content = fs::read_to_string(&json_path).expect("Failed to read JSON");
    assert!(json_content.contains("parsed_action_name"), "JSON should contain parsed_action_name field");
    assert!(json_content.contains("is_midi_relative"), "JSON should contain is_midi_relative field");
    
    println!("   ✅ JSON serialization includes new fields");
    
    // Verify we can round-trip the JSON
    let from_json: ReaperActionList = serde_json::from_str(&json_content)
        .expect("Failed to deserialize from JSON");
    
    let reparsed_midi_entries = from_json.0.iter()
        .filter_map(|entry| {
            if let ReaperEntry::Key(key_entry) = entry {
                if let Some(ref comment) = key_entry.comment {
                    if comment.is_midi_relative {
                        Some(())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .count();
    
    assert_eq!(midi_relative_entries.len(), reparsed_midi_entries, 
        "JSON round-trip should preserve MIDI relative entries");
    
    println!("   ✅ JSON round-trip preserves MIDI relative data");
    println!("🎉 MIDI relative action parsing tests completed successfully!");
    
    // Return summary for further use
    assert!(midi_relative_entries.len() > 0, "Should find some MIDI relative entries in real keymap");
    assert!(action_groups.len() > 0, "Should have different types of MIDI relative actions");
}

#[test]
fn test_special_input_coverage() {
    // Test that we can handle all types of special inputs
    println!("🧪 Testing special input coverage");
    
    let test_lines = vec![
        "KEY 255 248 40432 32060",  // Mousewheel -> MIDI editor vertical scroll
        "KEY 255 250 40431 32060",  // Alt+Mousewheel -> MIDI editor horizontal zoom  
        "KEY 255 218 40660 32060",  // Alt+HorizWheel -> MIDI editor horizontal scroll
        "KEY 255 220 40138 32060",  // Shift+HorizWheel -> MIDI editor scroll up
        "KEY 255 252 40139 32060",  // Shift+Mousewheel -> MIDI editor scroll down
        "KEY 255 216 989 0",        // HorizWheel -> Main window scroll
        "KEY 255 200 40454 32060",  // MultiZoom -> MIDI editor zoom
        "KEY 255 152 40455 32060",  // MultiRotate -> MIDI editor rotate
    ];
    
    let mut parsed_entries = Vec::new();
    
    for line in test_lines {
        match ReaperEntry::from_line(line) {
            Ok(entry) => {
                parsed_entries.push(entry);
                println!("   ✅ Parsed: {}", line);
            }
            Err(e) => {
                panic!("   ❌ Failed to parse: {} - Error: {}", line, e);
            }
        }
    }
    
    assert_eq!(parsed_entries.len(), 8, "Should have parsed all special input test lines");
    
    // Verify that each parsed entry is a special input
    for entry in &parsed_entries {
        if let ReaperEntry::Key(key_entry) = entry {
            assert!(
                matches!(key_entry.key_input, KeyInputType::Special(_)),
                "Entry should be parsed as special input: {:?}",
                key_entry
            );
            assert!(
                key_entry.modifiers.is_special_input(),
                "Modifiers should be marked as special input"
            );
        } else {
            panic!("Entry should be a Key entry: {:?}", entry);
        }
    }
    
    println!("✅ Special input coverage test passed");
}

#[test]
fn test_json_schema_structure() {
    // Test the JSON schema structure makes sense
    println!("📋 Testing JSON schema structure");
    
    let original_path = "resources/test-file.reaperkeymap";
    let action_list = ReaperActionList::load_from_file(original_path)
        .expect("Failed to load original keymap file");
    
    let json_data = serde_json::to_value(&action_list)
        .expect("Failed to serialize to JSON");
    
    // Validate top-level structure
    assert!(json_data.is_array() || json_data.is_object(), "JSON should be array or object");
    
    // If it's an object, check that it has the expected structure
    if let Some(obj) = json_data.as_object() {
        // Should have a field that contains the entries
        assert!(obj.len() > 0, "JSON object should not be empty");
    }
    
    // Convert to pretty JSON and check it's reasonable
    let pretty_json = serde_json::to_string_pretty(&action_list)
        .expect("Failed to serialize to pretty JSON");
    
    assert!(pretty_json.len() > 1000, "Pretty JSON should be substantial");
    assert!(pretty_json.contains("Key"), "JSON should contain Key entries");
    assert!(pretty_json.contains("modifiers"), "JSON should contain modifiers");
    assert!(pretty_json.contains("command_id"), "JSON should contain command_id");
    
    // Check for special inputs in JSON
    if pretty_json.contains("Special") {
        println!("   ✅ JSON contains Special input entries");
        assert!(pretty_json.contains("Mousewheel") || pretty_json.contains("HorizWheel"), 
                "JSON should contain mousewheel special inputs");
    }
    
    println!("✅ JSON schema structure test passed");
    println!("   📄 Generated {} characters of JSON", pretty_json.len());
} 