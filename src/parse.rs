use regex::Regex;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[allow(unused)]
#[derive(Debug, Eq, PartialEq)]
pub struct KeyBinding {
    device: u32,
    key_code: u32,
    command_id: u32,
    flags: u32,
    context: String,
    shortcut: String,
    override_default: bool,
    description: String,
}
impl KeyBinding {
    /// Serialize back into a single REAPER keymap line.
    fn to_line(&self) -> String {
        let comment = if self.override_default {
            format!(
                "{} : {} : OVERRIDE DEFAULT : {}",
                self.context, self.shortcut, self.description
            )
        } else {
            format!(
                "{} : {} : {}",
                self.context, self.shortcut, self.description
            )
        };
        format!(
            "KEY {} {} {} {} # {}",
            self.device, self.key_code, self.command_id, self.flags, comment
        )
    }
}

pub fn parse_line(line: &str) -> Option<KeyBinding> {
    // Build a regex with named groups.
    // - (?P<device>\d+) etc.
    // - override_default is captured if present
    let re = Regex::new(
        r"(?x)^
        KEY \s+
        (?P<device>\d+) \s+
        (?P<key_code>\d+) \s+
        (?P<command>\d+) \s+
        (?P<flags>\d+) \s*
        \# \s*
        (?P<context>[^:]+?) \s* : \s*           # everything up to first colon
        (?P<shortcut>[^:]*?) \s* (?: : \s*      # up to second colon
        (?P<override>OVERRIDE\ DEFAULT))?       # optional “OVERRIDE DEFAULT”
        \s* : \s*
        (?P<desc>.+)                            # rest of the description
    $",
    )
    .unwrap();

    let caps = re.captures(line)?;
    Some(KeyBinding {
        device: caps.name("device")?.as_str().parse().ok()?,
        key_code: caps.name("key_code")?.as_str().parse().ok()?,
        command_id: caps.name("command")?.as_str().parse().ok()?,
        flags: caps.name("flags")?.as_str().parse().ok()?,
        context: caps.name("context")?.as_str().trim().to_string(),
        shortcut: caps.name("shortcut")?.as_str().trim().to_string(),
        override_default: caps.name("override").is_some(),
        description: caps.name("desc")?.as_str().trim().to_string(),
    })
}
/// Read a `.reaperkeymap` file and parse every valid line into a Vec<KeyBinding>
pub fn parse_keymap_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<KeyBinding>> {
    let content = fs::read_to_string(path)?;
    let bindings = content.lines().filter_map(parse_line).collect();
    Ok(bindings)
}

/// Serialize a Vec<KeyBinding> back out to a file
pub fn write_keymap_file<P: AsRef<Path>>(path: P, bindings: &[KeyBinding]) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    for b in bindings {
        writeln!(file, "{}", b.to_line())?;
    }
    Ok(())
}

/// Parse `input`, write to `input` with extension replaced by `.reaperkeymap`,
/// then compare the raw bytes to ensure they’re identical.
pub fn round_trip_compare<P: AsRef<Path>>(input: P) -> io::Result<bool> {
    let input = input.as_ref();
    let bindings = parse_keymap_file(input)?;
    let output = Path::new("roundtrip.reaperkeymap");
    write_keymap_file(output, &bindings)?;
    let orig = fs::read(input)?;
    let new = fs::read(output)?;
    Ok(orig == new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_round_trip_file() {
        // Put a sample file at tests/fixtures/sample.reaperkeymap
        let input = Path::new("resources/test-file.reaperkeymap");
        assert!(
            round_trip_compare(input).unwrap(),
            "Round-trip output did not match original!"
        );
    }
    #[test]
    fn test_write_from_struct() {
        // 1) Construct a few KeyBinding instances by hand
        let bindings = vec![
            KeyBinding {
                device: 1,
                key_code: 85,
                command_id: 40760,
                flags: 4,
                context: "Main (alt-4)".into(),
                shortcut: "U".into(),
                override_default: true,
                description: "Edit: Dynamic split items...".into(),
            },
            KeyBinding {
                device: 37,
                key_code: 71,
                command_id: 40771,
                flags: 4,
                context: "Main (alt-4)".into(),
                shortcut: "T".into(),
                override_default: false,
                description: "Track: Toggle all track grouping enabled".into(),
            },
            KeyBinding {
                device: 255,
                key_code: 12520,
                command_id: 1013,
                flags: 0,
                context: "Main".into(),
                shortcut: "A".into(),
                override_default: false,
                description: "Transport: Record".into(),
            },
        ];

        // 2) Write them out to `test-from-struct.reaperkeymap` in the crate root
        let output = Path::new("test-from-struct.reaperkeymap");
        write_keymap_file(output, &bindings).expect("failed to write keymap file");

        // 3) Read it back in as a string
        let generated = fs::read_to_string(output).expect("failed to read generated file");

        // 4) Build the expected content
        let expected = [
            "KEY 1 85 40760 4 # Main (alt-4) : U : OVERRIDE DEFAULT : Edit: Dynamic split items...",
            "KEY 37 71 40771 4 # Main (alt-4) : T : Track: Toggle all track grouping enabled",
            "KEY 255 12520 1013 0 # Main : A : Transport: Record",
            "", // final newline
        ]
        .join("\n");

        // 5) Compare
        assert_eq!(
            generated, expected,
            "Generated keymap did not match expected"
        );
    }

    #[test]
    fn parse_line_with_override() {
        let line = "KEY 1 85 40760 4    # Main (alt-4) : U : OVERRIDE DEFAULT : Edit: Dynamic split items...";
        let kb = parse_line(line).expect("should parse successfully");

        assert_eq!(kb.device, 1);
        assert_eq!(kb.key_code, 85);
        assert_eq!(kb.command_id, 40760);
        assert_eq!(kb.flags, 4);

        assert_eq!(kb.context, "Main (alt-4)");
        assert_eq!(kb.shortcut, "U");
        assert!(kb.override_default);
        assert_eq!(kb.description, "Edit: Dynamic split items...");
    }

    #[test]
    fn parse_line_without_override() {
        let line = "KEY 37 71 40771 4  # Main (alt-4) : Shift+Control+G : Track: Toggle all track grouping enabled";
        let kb = parse_line(line).expect("should parse successfully");

        assert_eq!(kb.device, 37);
        assert_eq!(kb.key_code, 71);
        assert_eq!(kb.command_id, 40771);
        assert_eq!(kb.flags, 4);

        assert_eq!(kb.context, "Main (alt-4)");
        assert_eq!(kb.shortcut, "Shift+Control+G");
        assert!(!kb.override_default);
        assert_eq!(kb.description, "Track: Toggle all track grouping enabled");
    }

    #[test]
    fn parse_line_empty_shortcut() {
        let line = "KEY 255 12520 1013 0  # Main :  : Transport: Record";
        let kb = parse_line(line).expect("should parse successfully");

        assert_eq!(kb.device, 255);
        assert_eq!(kb.key_code, 12520);
        assert_eq!(kb.command_id, 1013);
        assert_eq!(kb.flags, 0);

        assert_eq!(kb.context, "Main");
        assert_eq!(kb.shortcut, "");
        assert!(!kb.override_default);
        assert_eq!(kb.description, "Transport: Record");
    }

    #[test]
    fn parse_line_fails_on_malformed() {
        let bad = "NOT_A_KEY_LINE";
        assert!(parse_line(bad).is_none());
    }
    #[test]
    fn round_trip_parse_and_serialize() {
        let lines = [
            "KEY 1 85 40760 4    # Main (alt-4) : U : OVERRIDE DEFAULT : Edit: Dynamic split items...",
            "KEY 37 71 40771 4  # Main (alt-4) : Shift+Control+G : Track: Toggle all track grouping enabled",
            "KEY 255 12520 1013 0  # Main :  : Transport: Record",
        ];

        // parse into structs
        let original: Vec<KeyBinding> = lines
            .iter()
            .map(|&l| parse_line(l).expect("parse_line failed"))
            .collect();

        // serialize back into strings
        let serialized: Vec<String> = original.iter().map(KeyBinding::to_line).collect();

        // re-parse the serialized strings
        let reparsed: Vec<KeyBinding> = serialized
            .iter()
            .map(|l| parse_line(l).expect("reparse failed"))
            .collect();

        assert_eq!(original, reparsed);
    }
}
