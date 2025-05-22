use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// All the "contexts" (sections) that Reaper keymaps can live in,
/// with their exact numeric codes.
#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive,
)]
#[repr(u32)]
pub enum ReaperActionSection {
    Main = 0,
    MainAltRecording = 100,
    MainAlt1 = 1,
    MainAlt2 = 2,
    MainAlt3 = 3,
    MainAlt4 = 4,
    MainAlt5 = 5,
    MainAlt6 = 6,
    MainAlt7 = 7,
    MainAlt8 = 8,
    MainAlt9 = 9,
    MainAlt10 = 10,
    MainAlt11 = 11,
    MainAlt12 = 12,
    MainAlt13 = 13,
    MainAlt14 = 14,
    MainAlt15 = 15,
    MainAlt16 = 16,
    MidiEditor = 32060,
    MidiEventList = 32061,
    MidiInline = 32062,
    MediaExplorer = 32063,
}

impl ReaperActionSection {
    /// Try to convert a raw `u32` into one of our `Section` variants.
    pub fn from_u32(n: u32) -> Option<Self> {
        Self::try_from(n).ok()
    }

    /// Convert a `Section` back into the raw `u32` code.
    pub fn as_u32(self) -> u32 {
        self.into()
    }

    /// Get the human-readable display name for comments
    pub fn display_name(self) -> &'static str {
        match self {
            ReaperActionSection::Main => "Main",
            ReaperActionSection::MainAltRecording => "Main (alt recording)",
            ReaperActionSection::MainAlt1 => "Main (alt-1)",
            ReaperActionSection::MainAlt2 => "Main (alt-2)",
            ReaperActionSection::MainAlt3 => "Main (alt-3)",
            ReaperActionSection::MainAlt4 => "Main (alt-4)",
            ReaperActionSection::MainAlt5 => "Main (alt-5)",
            ReaperActionSection::MainAlt6 => "Main (alt-6)",
            ReaperActionSection::MainAlt7 => "Main (alt-7)",
            ReaperActionSection::MainAlt8 => "Main (alt-8)",
            ReaperActionSection::MainAlt9 => "Main (alt-9)",
            ReaperActionSection::MainAlt10 => "Main (alt-10)",
            ReaperActionSection::MainAlt11 => "Main (alt-11)",
            ReaperActionSection::MainAlt12 => "Main (alt-12)",
            ReaperActionSection::MainAlt13 => "Main (alt-13)",
            ReaperActionSection::MainAlt14 => "Main (alt-14)",
            ReaperActionSection::MainAlt15 => "Main (alt-15)",
            ReaperActionSection::MainAlt16 => "Main (alt-16)",
            ReaperActionSection::MidiEditor => "MIDI Editor",
            ReaperActionSection::MidiEventList => "MIDI Event List", 
            ReaperActionSection::MidiInline => "MIDI Inline Editor",
            ReaperActionSection::MediaExplorer => "Media Explorer",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ReaperActionSection;
    use std::convert::TryFrom;

    #[test]
    fn round_trip_known_sections() {
        let cases = &[
            (0, ReaperActionSection::Main),
            (100, ReaperActionSection::MainAltRecording),
            (1, ReaperActionSection::MainAlt1),
            (16, ReaperActionSection::MainAlt16),
            (32060, ReaperActionSection::MidiEditor),
            (32061, ReaperActionSection::MidiEventList),
            (32062, ReaperActionSection::MidiInline),
            (32063, ReaperActionSection::MediaExplorer),
        ];

        for &(raw, expected) in cases {
            // from_u32 returns Some(expected)
            let from_opt = ReaperActionSection::from_u32(raw);
            assert_eq!(
                from_opt,
                Some(expected),
                "from_u32({}) returned {:?}, expected {:?}",
                raw,
                from_opt,
                expected
            );

            // TryFrom primitive also works
            let try_from = ReaperActionSection::try_from(raw).unwrap();
            assert_eq!(
                try_from, expected,
                "TryFrom::try_from({}) returned {:?}, expected {:?}",
                raw, try_from, expected
            );

            // And back-conversion preserves the raw value
            assert_eq!(
                expected.as_u32(),
                raw,
                "{:?}.as_u32() returned {}, expected {}",
                expected,
                expected.as_u32(),
                raw
            );
        }
    }

    #[test]
    fn invalid_section_codes() {
        // Some arbitrary values that aren't in the enum
        for &bad in &[42u32, 9999, 32064, u32::MAX] {
            assert!(
                ReaperActionSection::from_u32(bad).is_none(),
                "from_u32({}) should be None",
                bad
            );
            assert!(
                ReaperActionSection::try_from(bad).is_err(),
                "TryFrom::try_from({}) should Err",
                bad
            );
        }
    }

    #[test]
    fn pattern_matching_on_main_alt_range() {
        // Confirm that MainAlt1..MainAlt16 cover 1–16
        for n in 1..=16 {
            let section = ReaperActionSection::from_u32(n).unwrap();
            match section {
                ReaperActionSection::MainAlt1
                | ReaperActionSection::MainAlt2
                | ReaperActionSection::MainAlt3
                | ReaperActionSection::MainAlt4
                | ReaperActionSection::MainAlt5
                | ReaperActionSection::MainAlt6
                | ReaperActionSection::MainAlt7
                | ReaperActionSection::MainAlt8
                | ReaperActionSection::MainAlt9
                | ReaperActionSection::MainAlt10
                | ReaperActionSection::MainAlt11
                | ReaperActionSection::MainAlt12
                | ReaperActionSection::MainAlt13
                | ReaperActionSection::MainAlt14
                | ReaperActionSection::MainAlt15
                | ReaperActionSection::MainAlt16 => {
                    // good
                }
                other => panic!("Value {} mapped to unexpected {:?}", n, other),
            }
        }
    }
}
