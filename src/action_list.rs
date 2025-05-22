use crate::keycodes::KeyCode;
use crate::modifiers::Modifiers;
use crate::sections::ReaperActionSection;
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
        .find(|rk| rk.modifiers == input.modifiers && rk.key_code == input.key)
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

/// A 'KEY' entry: modifiers, key code, command ID, section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyEntry {
    pub modifiers: Modifiers,
    pub key_code: KeyCode,
    pub command_id: String,
    pub section: ReaperActionSection,
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
            ReaperEntry::Key(k) => format!(
                "KEY {} {} {} {}",
                k.modifiers.reaper_code(),
                k.key_code.as_u8(),
                k.command_id,
                k.section.as_u32(),
            ),
            ReaperEntry::Script(s) => {
                let desc = escape_field(&s.description);
                let path = escape_field(&s.path);
                let path_q = if path.chars().any(|c| c.is_whitespace()) {
                    format!("\"{}\"", path)
                } else {
                    path
                };
                format!(
                    "SCR {} {} {} \"{}\" {}",
                    u32::from(s.termination_behavior),
                    s.section.as_u32(),
                    s.command_id,
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
        let before = line.split('#').next().unwrap_or("").trim();
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
                let key_code = KeyCode::from_u16(code).ok_or(ParseError::InvalidKeyCode(code))?;
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
                Ok(ReaperEntry::Key(KeyEntry {
                    modifiers,
                    key_code,
                    command_id: cmd.to_string(),
                    section,
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

                // 3) parse command_id
                let cmd = parts.next().ok_or(ParseError::MissingField {
                    tag: "SCR",
                    field: "command_id",
                })?;

                // 4) split on quotes
                let quote_parts: Vec<&str> = before.split('"').collect();
                // we must have at least one pair of quotes for description
                if quote_parts.len() < 3 {
                    return Err(ParseError::MissingField {
                        tag: "SCR",
                        field: "description",
                    });
                }
                let description = quote_parts[1].to_string();

                // 5) path may be quoted (quote_parts[3]) or unquoted (quote_parts[2])
                let path = if quote_parts.len() > 3 {
                    // quoted case
                    quote_parts[3].to_string()
                } else {
                    // unquoted case: take remainder after the closing quote
                    quote_parts[2].trim().to_string()
                };

                Ok(ReaperEntry::Script(ScriptEntry {
                    termination_behavior,
                    section,
                    command_id: cmd.to_string(),
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
    let mut list = ReaperActionList(Vec::new());
    list
}

pub fn make_test_action_list() -> ReaperActionList {
    let mut list = ReaperActionList(Vec::new());

    // 1) push a no-modifier entry for “A”
    list.0.push(ReaperEntry::Key(KeyEntry {
        modifiers: Modifiers::empty(),
        key_code: KeyCode::A,
        command_id: "40044".to_string(),
        section: ReaperActionSection::Main,
    }));

    list.0.push(ReaperEntry::Key(KeyEntry {
        modifiers: Modifiers::CONTROL,
        key_code: KeyCode::A,
        command_id: "shifted command id".to_string(),
        section: ReaperActionSection::Main,
    }));

    // 2) push a Ctrl+B entry
    list.0.push(ReaperEntry::Key(KeyEntry {
        modifiers: Modifiers::CONTROL,
        key_code: KeyCode::B,
        command_id: "SWS_ACTION".to_string(),
        section: ReaperActionSection::Main,
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
        assert_eq!(lookup_command_id(&list, &input), Some("10004".to_string()));

        // lookup a missing combo (Shift+C)
        let missing = ReaperActionInput {
            modifiers: Modifiers::SHIFT,
            key: KeyCode::C,
        };
    }
}
