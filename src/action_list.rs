use crate::keycodes::KeyCode;
use crate::modifiers::Modifiers;
use crate::sections::ReaperActionSection;
use crate::special_inputs::SpecialInput;
use bitflags::bitflags;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::num::ParseIntError;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReaperActionInput {
    pub key: KeyCode,
    pub modifiers: Modifiers,
}

pub fn lookup_command_id(list: &ReaperActionList, input: &ReaperActionInput) -> Option<String> {
    list.keys()
        .iter()
        .find(|rk| {
            rk.modifiers == input.modifiers && 
            matches!(&rk.key_input, KeyInputType::Regular(key) if *key == input.key)
        })
        .map(|rk| rk.command_id.clone())
}

/// Errors that can occur while parsing keymap entries.
#[derive(Debug)]
pub enum ParseError {
    IoError(io::Error),
    MissingField {
        tag: &'static str,
        field: &'static str,
    },
    InvalidNumber {
        tag: &'static str,
        field: &'static str,
        err: String,
    },
    InvalidModifierCode(u8),
    InvalidKeyCode(u16),
    InvalidSectionCode(u32),
    InvalidTermination(u32),
    InvalidTag(String),
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        ParseError::IoError(e)
    }
}

impl From<ParseIntError> for ParseError {
    fn from(e: ParseIntError) -> Self {
        ParseError::InvalidNumber {
            tag: "<number>",
            field: "<value>",
            err: e.to_string(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::IoError(e) => write!(f, "I/O error: {}", e),
            ParseError::MissingField { tag, field } => {
                write!(f, "{} entry missing field {}", tag, field)
            }
            ParseError::InvalidNumber { tag, field, err } => {
                write!(f, "{} entry invalid number in {}: {}", tag, field, err)
            }
            ParseError::InvalidModifierCode(b) => write!(f, "invalid modifier code {}", b),
            ParseError::InvalidKeyCode(b) => write!(f, "invalid key code {}", b),
            ParseError::InvalidSectionCode(n) => write!(f, "invalid section code {}", n),
            ParseError::InvalidTermination(n) => write!(f, "invalid termination behavior {}", n),
            ParseError::InvalidTag(t) => write!(f, "invalid entry tag: {}", t),
        }
    }
}

impl std::error::Error for ParseError {}

/// Represents any KEY, SCR, or ACT entry in a Reaper keymap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReaperEntry {
    Key(KeyEntry),
    Script(ScriptEntry),
    Action(ActionEntry),
}

/// The type of input for a KEY entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyInputType {
    /// Regular keyboard key
    Regular(KeyCode),
    /// Special input (mousewheel, multitouch, etc.) used with modifier 255
    Special(SpecialInput),
}

/// Structured representation of a Reaper keymap comment
/// Format: # Section : KeyCombination : [BehaviorFlag] : [ActionDescription]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    /// The section name (e.g., "Main", "MIDI Editor")
    pub section: String,
    /// The key combination (e.g., "Cmd+Shift+M", "Mousewheel")
    pub key_combination: String,
    /// Optional behavior flag (e.g., "OVERRIDE DEFAULT", "DISABLED DEFAULT")
    pub behavior_flag: Option<String>,
    /// Optional action description (e.g., "Track: Toggle mute for selected tracks")
    pub action_description: Option<String>,
    /// Parsed action name from the description (e.g., "View: Scroll vertically")
    pub parsed_action_name: Option<String>,
    /// Whether this action supports MIDI CC relative/mousewheel input
    pub is_midi_relative: bool,
}

impl Comment {
    /// Parse a comment from a line that starts with #
    pub fn from_line(line: &str) -> Option<Self> {
        let line = line.trim();
        if !line.starts_with('#') {
            return None;
        }
        
        // Remove the # and split by :
        let content = line[1..].trim();
        let parts: Vec<&str> = content.split(':').map(|s| s.trim()).collect();
        
        if parts.len() < 2 {
            return None;
        }
        
        let section = parts[0].to_string();
        let key_combination = parts[1].to_string();
        
        let behavior_flag = if parts.len() > 2 && !parts[2].is_empty() {
            // Check if this part looks like a behavior flag or action description
            let part = parts[2];
            if part.contains("OVERRIDE") || part.contains("DISABLED") || part.contains("DEFAULT") {
                Some(part.to_string())
            } else {
                None
            }
        } else {
            None
        };
        
        let action_description = if behavior_flag.is_some() && parts.len() > 3 {
            // If we have a behavior flag, join all remaining parts as the action description
            let remaining_parts: Vec<&str> = parts[3..].iter().cloned().collect();
            if !remaining_parts.is_empty() && !remaining_parts.iter().all(|s| s.is_empty()) {
                Some(remaining_parts.join(": "))
            } else {
                None
            }
        } else if behavior_flag.is_none() && parts.len() > 2 && !parts[2].is_empty() {
            // If no behavior flag, join all parts from index 2 onwards as the action description
            let remaining_parts: Vec<&str> = parts[2..].iter().cloned().collect();
            Some(remaining_parts.join(": "))
        } else {
            None
        };
        
        // Parse action name and check for MIDI relative flag
        let (parsed_action_name, is_midi_relative) = if let Some(ref desc) = action_description {
            let is_midi_rel = desc.contains("(MIDI CC relative/mousewheel)") || 
                             desc.contains("(MIDI relative/mousewheel)");
            
            // Extract the action name (everything before the parentheses if present)
            let action_name = if let Some(paren_pos) = desc.find('(') {
                desc[..paren_pos].trim().to_string()
            } else {
                desc.clone()
            };
            
            (Some(action_name), is_midi_rel)
        } else {
            (None, false)
        };
        
        Some(Comment {
            section,
            key_combination,
            behavior_flag,
            action_description,
            parsed_action_name,
            is_midi_relative,
        })
    }
    
    /// Generate a comment line from this structured comment
    pub fn to_line(&self) -> String {
        let mut parts = vec![self.section.as_str(), self.key_combination.as_str()];
        
        if let Some(ref behavior) = self.behavior_flag {
            parts.push(behavior);
        }
        
        if let Some(ref action) = self.action_description {
            parts.push(action);
        }
        
        format!("# {}", parts.join(" : "))
    }
    
    /// Create a new comment with default behavior for the given key entry
    pub fn from_key_entry(entry: &KeyEntry) -> Self {
        let section = entry.section.display_name().to_string();
        let key_combination = entry.generate_key_description();
        let behavior_flag = if entry.command_id == "0" {
            Some("DISABLED DEFAULT".to_string())
        } else {
            Some("OVERRIDE DEFAULT".to_string())
        };
        
        Comment {
            section,
            key_combination,
            behavior_flag,
            action_description: None, // Could be enhanced to look up actual action names
            parsed_action_name: None,
            is_midi_relative: false,
        }
    }
}

/// A 'KEY' entry: modifiers, key input, command ID, section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyEntry {
    pub modifiers: Modifiers,
    pub key_input: KeyInputType,
    pub command_id: String,
    pub section: ReaperActionSection,
    pub comment: Option<Comment>,
}

impl KeyEntry {
    /// Get the legacy key_code for compatibility (returns None for special inputs)
    pub fn key_code(&self) -> Option<KeyCode> {
        match &self.key_input {
            KeyInputType::Regular(key_code) => Some(*key_code),
            KeyInputType::Special(_) => None,
        }
    }

    /// Generate a comment for this key entry
    pub fn generate_comment(&self) -> Comment {
        Comment::from_key_entry(self)
    }

    /// Generate the key combination description (e.g., "Cmd+Shift+M", "Mousewheel")
    pub fn generate_key_description(&self) -> String {
        let mut parts = Vec::new();
        
        // Add modifier descriptions
        if self.modifiers.contains(Modifiers::SUPER) {
            parts.push("Cmd".to_string());
        }
        if self.modifiers.contains(Modifiers::ALT) {
            parts.push("Opt".to_string());
        }
        if self.modifiers.contains(Modifiers::SHIFT) {
            parts.push("Shift".to_string());
        }
        if self.modifiers.contains(Modifiers::CONTROL) {
            parts.push("Control".to_string());
        }
        
        // Add key description
        let key_desc = match &self.key_input {
            KeyInputType::Regular(key_code) => key_code.display_name().to_string(),
            KeyInputType::Special(special_input) => special_input.to_string(),
        };
        
        if !key_desc.is_empty() {
            parts.push(key_desc);
        }
        
        if parts.is_empty() {
            String::new()
        } else {
            parts.join("+")
        }
    }
}

/// A 'SCR' entry: termination behavior, section, command ID, description, path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptEntry {
    pub termination_behavior: TerminationBehavior,
    pub section: ReaperActionSection,
    pub command_id: String,
    pub description: String,
    pub path: String,
}

/// Termination behaviors for scripts.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, IntoPrimitive, TryFromPrimitive,
)]
#[repr(u32)]
pub enum TerminationBehavior {
    Prompt = 4,
    TerminateExisting = 260,
    AlwaysNewInstance = 516,
}

bitflags! {
    /// Flags controlling custom actions.
    #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
    #[serde(transparent)]
    pub struct ActionFlags: u32 {
        const CONSOLIDATE_UNDO = 0b0000_0001;
        const SHOW_IN_MENUS    = 0b0000_0010;
        const ACTIVE_IF_ALL    = 0b0001_0000;
        const ACTIVE_IF_ANY    = 0b0010_0000;
    }
}

/// An 'ACT' entry: flags, section, command ID, description, action IDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionEntry {
    pub action_flags: ActionFlags,
    pub section: ReaperActionSection,
    pub command_id: String,
    pub description: String,
    pub action_ids: Vec<String>,
}

// Helper to escape fields for serialization
fn escape_field(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

impl ReaperEntry {
    /// Serialize this entry back to a keymap line.
    pub fn to_line(&self) -> String {
        match self {
            ReaperEntry::Key(k) => {
                let key_value = match &k.key_input {
                    KeyInputType::Regular(key_code) => key_code.as_u8() as u16,
                    KeyInputType::Special(special_input) => special_input.to_key_code(),
                };
                let base_line = format!(
                    "KEY {} {} {} {}",
                    k.modifiers.reaper_code(),
                    key_value,
                    k.command_id,
                    k.section.as_u32(),
                );
                
                // Add comment if present
                if let Some(ref comment) = k.comment {
                    format!("{} {}", base_line, comment.to_line())
                } else {
                    // Generate a default comment
                    let default_comment = k.generate_comment();
                    format!("{} {}", base_line, default_comment.to_line())
                }
            },
            ReaperEntry::Script(s) => {
                let desc = escape_field(&s.description);
                // Don't escape paths - they should be stored raw and only quoted if they contain spaces
                let path = &s.path;
                let cmd = escape_field(&s.command_id);
                
                // Quote command_id if it contains spaces or special characters
                let cmd_q = if cmd.chars().any(|c| c.is_whitespace()) {
                    format!("\"{}\"", cmd)
                } else {
                    cmd
                };
                
                // Quote path if it contains spaces
                let path_q = if path.chars().any(|c| c.is_whitespace()) {
                    format!("\"{}\"", path)
                } else {
                    path.to_string()
                };
                
                format!(
                    "SCR {} {} {} \"{}\" {}",
                    u32::from(s.termination_behavior),
                    s.section.as_u32(),
                    cmd_q,
                    desc,
                    path_q,
                )
            }
            ReaperEntry::Action(a) => {
                let cmd = escape_field(&a.command_id);
                let desc = escape_field(&a.description);
                let ids = a.action_ids.join(" ");
                if ids.is_empty() {
                    format!(
                        "ACT {} {} \"{}\" \"{}\"",
                        a.action_flags.bits(),
                        a.section.as_u32(),
                        cmd,
                        desc,
                    )
                } else {
                    format!(
                        "ACT {} {} \"{}\" \"{}\" {}",
                        a.action_flags.bits(),
                        a.section.as_u32(),
                        cmd,
                        desc,
                        ids,
                    )
                }
            }
        }
    }

    /// Parse a line into an entry, returning detailed errors.
    pub fn from_line(line: &str) -> Result<Self, ParseError> {
        // Split line into entry part and comment part
        let parts_split: Vec<&str> = line.splitn(2, '#').collect();
        let before = parts_split[0].trim();
        let comment_part = if parts_split.len() > 1 { 
            Some(format!("#{}", parts_split[1])) 
        } else { 
            None 
        };
        
        let mut parts = before.split_whitespace();
        let tag = parts.next().ok_or(ParseError::MissingField {
            tag: "<line>",
            field: "tag",
        })?;
        match tag {
            "KEY" => {
                let mods_str = parts.next().ok_or(ParseError::MissingField {
                    tag: "KEY",
                    field: "modifiers",
                })?;
                let mods = mods_str
                    .parse::<u8>()
                    .map_err(|e| ParseError::InvalidNumber {
                        tag: "KEY",
                        field: "modifiers",
                        err: e.to_string(),
                    })?;
                let modifiers = Modifiers::try_from_reaper_code(mods)
                    .ok_or(ParseError::InvalidModifierCode(mods))?;
                let code_str = parts.next().ok_or(ParseError::MissingField {
                    tag: "KEY",
                    field: "key_code",
                })?;
                let code = code_str
                    .parse::<u16>()
                    .map_err(|e| ParseError::InvalidNumber {
                        tag: "KEY",
                        field: "key_code",
                        err: e.to_string(),
                    })?;
                
                // Determine the key input type based on modifier
                let key_input = if modifiers.is_special_input() {
                    // For modifier 255, use special input parsing
                    KeyInputType::Special(SpecialInput::from_key_code(code))
                } else {
                    // For normal modifiers, use regular key code parsing
                    let key_code = KeyCode::from_u16(code).ok_or(ParseError::InvalidKeyCode(code))?;
                    KeyInputType::Regular(key_code)
                };
                let cmd = parts.next().ok_or(ParseError::MissingField {
                    tag: "KEY",
                    field: "command_id",
                })?;
                let sec_str = parts.next().ok_or(ParseError::MissingField {
                    tag: "KEY",
                    field: "section",
                })?;
                let sec = sec_str
                    .parse::<u32>()
                    .map_err(|e| ParseError::InvalidNumber {
                        tag: "KEY",
                        field: "section",
                        err: e.to_string(),
                    })?;
                let section = ReaperActionSection::from_u32(sec)
                    .ok_or(ParseError::InvalidSectionCode(sec))?;
                
                // Parse comment if present
                let comment = comment_part.and_then(|c| Comment::from_line(&c));
                
                Ok(ReaperEntry::Key(KeyEntry {
                    modifiers,
                    key_input,
                    command_id: cmd.to_string(),
                    section,
                    comment,
                }))
            }
            "SCR" => {
                // 1) parse termination
                let term_str = parts.next().ok_or(ParseError::MissingField {
                    tag: "SCR",
                    field: "termination",
                })?;
                let term = term_str
                    .parse::<u32>()
                    .map_err(|e| ParseError::InvalidNumber {
                        tag: "SCR",
                        field: "termination",
                        err: e.to_string(),
                    })?;
                let termination_behavior = TerminationBehavior::try_from(term)
                    .map_err(|_| ParseError::InvalidTermination(term))?;

                // 2) parse section
                let sec_str = parts.next().ok_or(ParseError::MissingField {
                    tag: "SCR",
                    field: "section",
                })?;
                let sec = sec_str
                    .parse::<u32>()
                    .map_err(|e| ParseError::InvalidNumber {
                        tag: "SCR",
                        field: "section",
                        err: e.to_string(),
                    })?;
                let section = ReaperActionSection::from_u32(sec)
                    .ok_or(ParseError::InvalidSectionCode(sec))?;

                // 3) Parse command_id and description carefully from quoted fields
                let quote_parts: Vec<&str> = before.split('"').collect();
                
                // Check if command_id is quoted or unquoted
                let (command_id, description, path) = if before.contains('"') {
                    // There are quotes, need to figure out the structure
                    if quote_parts.len() < 3 {
                        return Err(ParseError::MissingField {
                            tag: "SCR",
                            field: "description",
                        });
                    }
                    
                    // Check if the first quote comes before the command_id position
                    let before_first_quote = quote_parts[0];
                    let parts_before_quote: Vec<&str> = before_first_quote.split_whitespace().collect();
                    
                    if parts_before_quote.len() == 3 {
                        // Command ID is quoted: SCR term section "command_id" "description" path
                        if quote_parts.len() < 5 {
                            return Err(ParseError::MissingField {
                                tag: "SCR", 
                                field: "description",
                            });
                        }
                        let cmd_id = quote_parts[1].to_string();
                        let desc = quote_parts[3].to_string();
                        let path_part = if quote_parts.len() > 5 {
                            // Path is quoted
                            quote_parts[5].to_string()
                        } else {
                            // Path is unquoted, get remainder after last quote
                            quote_parts[4].trim().to_string()
                        };
                        (cmd_id, desc, path_part)
                    } else {
                        // Command ID is unquoted: SCR term section command_id "description" path
                        let cmd = parts.next().ok_or(ParseError::MissingField {
                            tag: "SCR",
                            field: "command_id",
                        })?;
                        let desc = quote_parts[1].to_string();
                        let path_part = if quote_parts.len() > 3 {
                            // Path is quoted
                            quote_parts[3].to_string()
                        } else {
                            // Path is unquoted
                            quote_parts[2].trim().to_string()
                        };
                        (cmd.to_string(), desc, path_part)
                    }
                } else {
                    // No quotes at all - this would be malformed for SCR
                    return Err(ParseError::MissingField {
                        tag: "SCR",
                        field: "description",
                    });
                };

                Ok(ReaperEntry::Script(ScriptEntry {
                    termination_behavior,
                    section,
                    command_id,
                    description,
                    path,
                }))
            }
            "ACT" => {
                // 1) parse flags and section
                let flags_str = parts.next().ok_or(ParseError::MissingField {
                    tag: "ACT",
                    field: "flags",
                })?;
                let flags = flags_str
                    .parse::<u32>()
                    .map_err(|e| ParseError::InvalidNumber {
                        tag: "ACT",
                        field: "flags",
                        err: e.to_string(),
                    })?;
                let action_flags = ActionFlags::from_bits_truncate(flags);

                let sec_str = parts.next().ok_or(ParseError::MissingField {
                    tag: "ACT",
                    field: "section",
                })?;
                let sec = sec_str
                    .parse::<u32>()
                    .map_err(|e| ParseError::InvalidNumber {
                        tag: "ACT",
                        field: "section",
                        err: e.to_string(),
                    })?;
                let section = ReaperActionSection::from_u32(sec)
                    .ok_or(ParseError::InvalidSectionCode(sec))?;

                // 2) reliably extract the two quoted fields
                let quote_parts: Vec<&str> = before.split('"').collect();
                if quote_parts.len() < 4 {
                    return Err(ParseError::MissingField {
                        tag: "ACT",
                        field: "command_id/description",
                    });
                }
                let command_id = quote_parts[1].to_string();
                let description = quote_parts[3].to_string();

                // 3) everything after the second closing quote is the list of IDs
                let ids_part = quote_parts.get(4).unwrap_or(&"");
                let action_ids = ids_part.split_whitespace().map(String::from).collect();

                Ok(ReaperEntry::Action(ActionEntry {
                    action_flags,
                    section,
                    command_id,
                    description,
                    action_ids,
                }))
            }
            other => Err(ParseError::InvalidTag(other.to_string())),
        }
    }
}

fn do_nothing() {}

/// Collection of Reaper entries with I/O methods.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReaperActionList(pub Vec<ReaperEntry>);

impl ReaperActionList {
    /// Load all entries from a file, skipping malformed lines.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        for (i, line) in reader.lines().enumerate() {
            let text = line?;
            match ReaperEntry::from_line(&text) {
                Ok(entry) => entries.push(entry),
                Err(e) => do_nothing(),
            }
        }
        Ok(ReaperActionList(entries))
    }

    /// Save all entries back to a file.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = fs::File::create(path)?;
        for entry in &self.0 {
            writeln!(file, "{}", entry.to_line())?;
        }
        Ok(())
    }

    pub fn keys(&self) -> Vec<KeyEntry> {
        self.0
            .iter()
            .filter_map(|e| {
                if let ReaperEntry::Key(k) = e {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

pub fn get_action_list_from_current_config() -> ReaperActionList {
    
    ReaperActionList(Vec::new())
}

pub fn make_test_action_list() -> ReaperActionList {
    let mut list = ReaperActionList(Vec::new());

    // 1) push a no-modifier entry for "A"
    list.0.push(ReaperEntry::Key(KeyEntry {
        modifiers: Modifiers::empty(),
        key_input: KeyInputType::Regular(KeyCode::A),
        command_id: "40044".to_string(),
        section: ReaperActionSection::Main,
        comment: None,
    }));

    list.0.push(ReaperEntry::Key(KeyEntry {
        modifiers: Modifiers::CONTROL,
        key_input: KeyInputType::Regular(KeyCode::A),
        command_id: "shifted command id".to_string(),
        section: ReaperActionSection::Main,
        comment: None,
    }));

    // 2) push a Ctrl+B entry
    list.0.push(ReaperEntry::Key(KeyEntry {
        modifiers: Modifiers::CONTROL,
        key_input: KeyInputType::Regular(KeyCode::B),
        command_id: "SWS_ACTION".to_string(),
        section: ReaperActionSection::Main,
        comment: None,
    }));

    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_existing_command() {
        let list = make_test_action_list();

        // lookup the existing Ctrl+B
        let input = ReaperActionInput {
            modifiers: Modifiers::CONTROL,
            key: KeyCode::B,
        };
        assert_eq!(lookup_command_id(&list, &input), Some("SWS_ACTION".to_string()));

        // lookup a missing combo (Shift+C)
        let missing = ReaperActionInput {
            modifiers: Modifiers::SHIFT,
            key: KeyCode::C,
        };
        assert_eq!(lookup_command_id(&list, &missing), None);
    }

    #[test]
    fn test_parse_individual_lines() {
        // Test parsing different types of lines
        
        // Test KEY entry (33 = CONTROL + 1, 65 = KeyCode::A)
        let key_line = "KEY 33 65 40044 0";
        let key_entry = ReaperEntry::from_line(key_line).unwrap();
        if let ReaperEntry::Key(k) = key_entry {
            assert_eq!(k.modifiers, Modifiers::CONTROL);
            assert_eq!(k.key_input, KeyInputType::Regular(KeyCode::A));
            assert_eq!(k.command_id, "40044");
        } else {
            panic!("Expected Key entry");
        }

        // Test SCR entry with quoted command_id
        let scr_line = r#"SCR 4 0 "_Script: Test script" "Some description" /path/to/script.lua"#;
        let scr_entry = ReaperEntry::from_line(scr_line).unwrap();
        if let ReaperEntry::Script(s) = scr_entry {
            assert_eq!(s.command_id, "_Script: Test script");
            assert_eq!(s.description, "Some description");
            assert_eq!(s.path, "/path/to/script.lua");
        } else {
            panic!("Expected Script entry");
        }
        
        // Test SCR entry with unquoted command_id
        let scr_line2 = r#"SCR 4 0 _Script_Test "My Test Script" "/path with spaces/script.lua""#;
        let scr_entry2 = ReaperEntry::from_line(scr_line2).unwrap();
        if let ReaperEntry::Script(s) = scr_entry2 {
            assert_eq!(s.command_id, "_Script_Test");
            assert_eq!(s.description, "My Test Script");
            assert_eq!(s.path, "/path with spaces/script.lua");
        } else {
            panic!("Expected Script entry");
        }

        // Test ACT entry
        let act_line = r#"ACT 0 0 "_Custom_Action" "My Custom Action" 40044 40045"#;
        let act_entry = ReaperEntry::from_line(act_line).unwrap();
        if let ReaperEntry::Action(a) = act_entry {
            assert_eq!(a.command_id, "_Custom_Action");
            assert_eq!(a.description, "My Custom Action");
            assert_eq!(a.action_ids, vec!["40044", "40045"]);
        } else {
            panic!("Expected Action entry");
        }
    }

    #[test]
    fn test_round_trip_serialization() {
        // Test that parsing and serializing gives consistent functional results
        let lines = vec![
            "KEY 33 65 40044 0", // 33 = CONTROL + 1
            r#"SCR 4 0 "_Script" "Test script" /path/script.lua"#,
            r#"ACT 0 0 "_Action" "Test action" 40044 40045"#,
        ];

        for line in lines {
            let entry = ReaperEntry::from_line(line).unwrap();
            let serialized = entry.to_line();
            let reparsed = ReaperEntry::from_line(&serialized).unwrap();
            
            // For KEY entries, we now auto-generate comments, so we need to compare the functional parts
            match (&entry, &reparsed) {
                (ReaperEntry::Key(original), ReaperEntry::Key(reparsed_key)) => {
                    assert_eq!(original.modifiers, reparsed_key.modifiers);
                    assert_eq!(original.key_input, reparsed_key.key_input);
                    assert_eq!(original.command_id, reparsed_key.command_id);
                    assert_eq!(original.section, reparsed_key.section);
                    // Comment should be auto-generated for reparsed entry
                    assert!(reparsed_key.comment.is_some(), "Reparsed KEY entry should have auto-generated comment");
                }
                // For SCR and ACT entries, they should be exactly equal
                _ => {
                    assert_eq!(entry, reparsed);
                }
            }
        }
    }

    #[test]
    fn test_load_sample_keymap_file() {
        // Test loading from a sample keymap file
        use std::fs;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let sample_keymap = r#"
# This is a comment
KEY 1 32 40044 0
KEY 33 65 40001 0  
KEY 9 66 40002 0
SCR 4 0 "_Script_Test" "My Test Script" /path/to/test.lua
ACT 0 0 "_Custom_Test" "Test Custom Action" 40044 40045 40046
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(sample_keymap.as_bytes()).unwrap();
        
        let result = ReaperActionList::load_from_file(temp_file.path());
        assert!(result.is_ok());
        
        let action_list = result.unwrap();
        assert_eq!(action_list.0.len(), 5); // Should parse 5 entries (ignore comments and empty lines)
        
        // Test that we can find keys
        let keys = action_list.keys();
        assert_eq!(keys.len(), 3); // Should have 3 KEY entries
        
        // Test looking up a specific key
        let input = ReaperActionInput {
            modifiers: Modifiers::CONTROL,
            key: KeyCode::A,
        };
        assert_eq!(lookup_command_id(&action_list, &input), Some("40001".to_string()));
    }

    #[test]
    fn test_load_real_keymap_file() {
        // Test loading the actual test keymap file from resources
        let keymap_path = std::path::Path::new("resources/test-file.reaperkeymap");
        
        let result = ReaperActionList::load_from_file(keymap_path);
        assert!(result.is_ok(), "Failed to load real keymap file: {:?}", result.err());
        
        let action_list = result.unwrap();
        
        // Should have a significant number of entries (the file has 916 lines, but some are comments)
        // We now successfully parse 734 entries (a great improvement!)
        assert!(action_list.0.len() > 700, "Expected more than 700 entries, got {}", action_list.0.len());
        assert!(action_list.0.len() < 916, "Expected less than 916 entries (some lines are comments), got {}", action_list.0.len());
        
        // Test that we can find keys
        let keys = action_list.keys();
        assert!(keys.len() > 700, "Expected more than 700 KEY entries, got {}", keys.len());
        
        // Test looking up some specific real entries from the file
        
        // Test entry: KEY 1 82 1013 0 # Main : R : OVERRIDE DEFAULT : Transport: Record
        let record_input = ReaperActionInput {
            modifiers: Modifiers::empty(), // 1 = no modifiers (0+1)
            key: KeyCode::R,
        };
        assert_eq!(lookup_command_id(&action_list, &record_input), Some("1013".to_string()));
        
        // Test entry: KEY 9 78 40023 0 # Main : Cmd+N : OVERRIDE DEFAULT : File: New project  
        let new_project_input = ReaperActionInput {
            modifiers: Modifiers::SUPER, // 9 = SUPER (8+1)
            key: KeyCode::N,
        };
        assert_eq!(lookup_command_id(&action_list, &new_project_input), Some("40023".to_string()));
        
        // Test entry: KEY 33 70 8 0 # Main : Control+F : Track: Toggle FX bypass for selected tracks
        let fx_bypass_input = ReaperActionInput {
            modifiers: Modifiers::CONTROL, // 33 = CONTROL (32+1)
            key: KeyCode::F,
        };
        assert_eq!(lookup_command_id(&action_list, &fx_bypass_input), Some("8".to_string()));
    }

    #[test]
    fn test_get_midi_editor_scroll_commands_from_real_file() {
        // Test finding MIDI editor scroll commands from the real keymap file
        let keymap_path = std::path::Path::new("resources/test-file.reaperkeymap");
        let action_list = ReaperActionList::load_from_file(keymap_path).unwrap();
        
        // Find MIDI editor scroll commands (section 32060)
        let midi_scroll_commands: Vec<_> = action_list.0
            .iter()
            .filter_map(|entry| {
                if let ReaperEntry::Key(k) = entry {
                    if k.section == ReaperActionSection::MidiEditor {
                        Some((k.key_input.clone(), k.modifiers, k.command_id.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
            
        // Should find many MIDI editor commands  
        // We now successfully parse 47 MIDI editor commands (great improvement!)
        assert!(midi_scroll_commands.len() > 40, "Expected many MIDI editor commands, got {}", midi_scroll_commands.len());
        
        // Look for specific scroll-related commands we care about
        // KEY 255 248 40432 32060 # MIDI Editor : Mousewheel : OVERRIDE DEFAULT : View: Scroll vertically (MIDI relative/mousewheel)
        let vertical_scroll = midi_scroll_commands.iter()
            .find(|(_, _, cmd)| cmd == "40432");
        assert!(vertical_scroll.is_some(), "Should find command 40432 (vertical scroll) in MIDI editor");
        
        // KEY 255 250 40431 32060 # MIDI Editor : Opt+Mousewheel : OVERRIDE DEFAULT : View: Zoom horizontally (MIDI relative/mousewheel)  
        let horizontal_zoom = midi_scroll_commands.iter()
            .find(|(_, _, cmd)| cmd == "40431");
        assert!(horizontal_zoom.is_some(), "Should find command 40431 (horizontal zoom) in MIDI editor");
        
        // KEY 255 220 40660 32060 # MIDI Editor : Shift+HorizWheel : OVERRIDE DEFAULT : View: Scroll horizontally reversed (MIDI relative/mousewheel)
        let horizontal_scroll = midi_scroll_commands.iter()
            .find(|(_, _, cmd)| cmd == "40660");
        assert!(horizontal_scroll.is_some(), "Should find command 40660 (horizontal scroll) in MIDI editor");
    }

    #[test]
    fn test_parse_complex_modifier_codes_from_real_file() {
        // Test parsing complex modifier codes like 255 from the real file
        let keymap_path = std::path::Path::new("resources/test-file.reaperkeymap");
        let action_list = ReaperActionList::load_from_file(keymap_path).unwrap();
        
        // Find entries with modifier code 255 (these appear in the real file)
        let complex_modifiers: Vec<_> = action_list.0
            .iter()
            .filter_map(|entry| {
                if let ReaperEntry::Key(k) = entry {
                    // Check if this uses a complex modifier (like 255)
                    let reaper_code = k.modifiers.reaper_code();
                    if reaper_code == 255 {
                        Some((k.key_input.clone(), k.modifiers, k.command_id.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
            
        // The real file has many entries with modifier 255
        // KEY 255 218 0 0 # Main : Opt+HorizWheel : DISABLED DEFAULT
        // KEY 255 248 989 0 # Main : Mousewheel : OVERRIDE DEFAULT : View: Scroll vertically (MIDI CC relative/mousewheel)
        assert!(complex_modifiers.len() > 10, "Expected many entries with modifier 255, got {}", complex_modifiers.len());
    }

    #[test]
    fn test_get_scroll_commands() {
        // Test finding scroll-related commands from the real keymap
        let keymap_path = std::path::Path::new("resources/test-file.reaperkeymap");
        let action_list = ReaperActionList::load_from_file(keymap_path).unwrap();
        
        // Find all scroll-related commands across all sections
        let scroll_commands: Vec<_> = action_list.0
            .iter()
            .filter_map(|entry| {
                if let ReaperEntry::Key(k) = entry {
                    // Look for scroll-related command IDs
                    if k.command_id == "989" || k.command_id == "40432" || k.command_id == "40431" || k.command_id == "40660" {
                        Some((k.section, k.key_input.clone(), k.modifiers, k.command_id.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
            
        // Should find scroll commands in both main window and MIDI editor
        assert!(scroll_commands.len() > 5, "Expected several scroll commands, got {}", scroll_commands.len());
        
        // Verify we have scroll commands in different sections
        let main_scrolls = scroll_commands.iter().filter(|(section, _, _, _)| *section == ReaperActionSection::Main).count();
        let midi_scrolls = scroll_commands.iter().filter(|(section, _, _, _)| *section == ReaperActionSection::MidiEditor).count();
        
        assert!(main_scrolls > 0, "Should find scroll commands in main section");
        assert!(midi_scrolls > 0, "Should find scroll commands in MIDI editor section");
    }

    #[test]
    fn test_parse_error_handling() {
        // Test malformed lines
        let bad_lines = vec![
            "INVALID_TAG 1 2 3",
            "KEY", // missing fields
            "KEY abc 65 40044 0", // invalid number
            "SCR 999 0 test desc path", // invalid termination
        ];

        for line in bad_lines {
            assert!(ReaperEntry::from_line(line).is_err());
        }
    }
}
