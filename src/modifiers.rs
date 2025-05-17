use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Modifiers: u8 {
        const SHIFT   = 0b0000_0100; //  4
        //
        const CONTROL = 0b0010_0000; // 32
        const ALT     = 0b0001_0000; // 16
        const SUPER   = 0b0000_1000; //  8
    }
}

impl Modifiers {
    /// The Reaper Keymap code for modifiers is always 1 + the sum of the bits, this is because
    /// no modifiers is 1 instead of 0 in the ReaperKeyMap files
    pub fn reaper_code(self) -> u8 {
        1 + self.bits()
    }
}

// Helper to convert raw modifier code into Modifiers
impl Modifiers {
    /// Convert Reaper code (1 + bits) back into flag set.
    pub fn try_from_reaper_code(n: u8) -> Option<Self> {
        let bits = n.checked_sub(1)?;
        Modifiers::from_bits(bits)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mods() {
        let m = Modifiers::SHIFT | Modifiers::CONTROL;
        assert_eq!(m.reaper_code(), 37);

        let all = Modifiers::SHIFT | Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER;
        assert_eq!(all.reaper_code(), 61);

        match m {
            m if m.is_empty() => println!("No Modifiers"),
            m if m == Modifiers::SHIFT => println!("Shift"),
            m if m.contains(Modifiers::CONTROL) => println!("Control is pressed"),
            m if m.contains(Modifiers::ALT) => println!("Alt is pressedk"),

            _ => (),
        }

        let all = Modifiers::all();
        assert_eq!(all.reaper_code(), 61);
    }
    #[test]
    fn test_all_modifier_combinations() {
        // The raw bit-values for our flags:
        // SHIFT=4, CONTROL=32, ALT=16, SUPER=8 → sum = 60
        let max_bits = Modifiers::all().bits();

        // Iterate every integer from 0..=60 and pick only those
        // that correspond to a valid combination of our flags.
        for bits in 0..=max_bits {
            if let Some(flags) = Modifiers::from_bits(bits) {
                // 1) reaper_code must be exactly sum_of_bits + 1
                assert_eq!(
                    flags.reaper_code(),
                    bits + 1,
                    "flags={:?} had bits={} but reaper_code() was {}",
                    flags,
                    bits,
                    flags.reaper_code()
                );

                // 2) Round‐trip via bits() → from_bits() → bits()
                let round = Modifiers::from_bits(flags.bits())
                    .expect("round-trip from_bits should succeed");
                assert_eq!(
                    round.bits(),
                    bits,
                    "round-trip bits of {:?} was {}, expected {}",
                    flags,
                    round.bits(),
                    bits
                );
            }
        }
    }

    #[test]
    fn test_specific_known_cases() {
        // A few hand-picked sanity checks
        let cases = &[
            (Modifiers::empty(), 1),  // 0+1
            (Modifiers::SHIFT, 5),    // 4+1
            (Modifiers::CONTROL, 33), // 32+1
            (Modifiers::ALT, 17),     // 16+1
            (Modifiers::SUPER, 9),    // 8+1
            (Modifiers::SHIFT | Modifiers::CONTROL, 37),
            (Modifiers::all(), 61),
        ];

        for &(flags, expected) in cases {
            assert_eq!(
                flags.reaper_code(),
                expected,
                "{:?}.reaper_code() was {}, expected {}",
                flags,
                flags.reaper_code(),
                expected
            );
        }
    }

    //TODO! 255 is a special modifier case that doesn't exist yet
    #[test]
    fn test_case_255() {
        // 255 → bits = 254 (0b1111_1110), no exact match => None
        assert!(
            Modifiers::try_from_reaper_code(255).is_none(),
            "255 should not map to a valid full-bit modifier set"
        );

        // if you instead truncate unknown bits, you get all the known flags:
        let truncated = Modifiers::from_bits_truncate(254);
        assert_eq!(
            truncated,
            Modifiers::all(),
            "254 (0b11111110) truncated should be all flags"
        );
    }
}
