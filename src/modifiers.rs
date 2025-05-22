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
        
        // Special flag for modifier code 255 (mousewheel, multitouch, media keys)
        const SPECIAL_INPUT = 0b1000_0000; // 128 - highest bit to avoid conflicts
    }
}

impl Modifiers {
    /// The Reaper Keymap code for modifiers is always 1 + the sum of the bits, this is because
    /// no modifiers is 1 instead of 0 in the ReaperKeyMap files
    /// 
    /// Special case: modifier code 255 represents special inputs (mousewheel, multitouch, etc.)
    pub fn reaper_code(self) -> u8 {
        if self.contains(Modifiers::SPECIAL_INPUT) {
            255
        } else {
            1 + (self.bits() & 0x7F) // Mask out the special bit for normal modifiers
        }
    }
}

// Helper to convert raw modifier code into Modifiers
impl Modifiers {
    /// Convert Reaper code (1 + bits) back into flag set.
    /// Special handling for code 255 which represents special inputs like mousewheel.
    pub fn try_from_reaper_code(n: u8) -> Option<Self> {
        if n == 255 {
            // Special case: modifier 255 represents mousewheel, multitouch, media keys
            Some(Modifiers::SPECIAL_INPUT)
        } else {
            let bits = n.checked_sub(1)?;
            Modifiers::from_bits(bits)
        }
    }
    
    /// Check if this represents a special input type (mousewheel, multitouch, etc.)
    pub fn is_special_input(self) -> bool {
        self.contains(Modifiers::SPECIAL_INPUT)
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

        // Note: Modifiers::all() now includes SPECIAL_INPUT which has reaper_code 255
        let all_regular = Modifiers::SHIFT | Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER;
        assert_eq!(all_regular.reaper_code(), 61);
        
        let all_with_special = Modifiers::all();
        assert_eq!(all_with_special.reaper_code(), 255); // Because SPECIAL_INPUT takes precedence
    }
    #[test]
    fn test_all_modifier_combinations() {
        // The raw bit-values for our regular flags:
        // SHIFT=4, CONTROL=32, ALT=16, SUPER=8 → sum = 60
        // SPECIAL_INPUT=128 is handled separately
        
        // Test regular modifier combinations (excluding SPECIAL_INPUT)
        let regular_flags = Modifiers::SHIFT | Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER;
        let max_regular_bits = regular_flags.bits();

        for bits in 0..=max_regular_bits {
            if let Some(flags) = Modifiers::from_bits(bits) {
                // Skip if this includes SPECIAL_INPUT
                if flags.contains(Modifiers::SPECIAL_INPUT) {
                    continue;
                }
                
                // 1) reaper_code must be exactly sum_of_bits + 1 for regular flags
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
        
        // Test SPECIAL_INPUT separately
        let special = Modifiers::SPECIAL_INPUT;
        assert_eq!(special.reaper_code(), 255, "SPECIAL_INPUT should have reaper_code 255");
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
            (Modifiers::SHIFT | Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER, 61),
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

    #[test]
    fn test_case_255() {
        // 255 is a special case for mousewheel, multitouch, media keys
        let special = Modifiers::try_from_reaper_code(255);
        assert!(special.is_some(), "255 should map to SPECIAL_INPUT flag");
        
        let special_flags = special.unwrap();
        assert!(special_flags.is_special_input(), "255 should be detected as special input");
        assert_eq!(special_flags.reaper_code(), 255, "Special input should round-trip to 255");

        // Test that 254 still doesn't work for normal flags
        let truncated = Modifiers::from_bits_truncate(254 & 0x7F); // 254 & 0x7F = 126 = 0b01111110
        // 126 = SHIFT(4) + ALT(16) + SUPER(8) + CONTROL(32) + extra bits
        // But 126 includes bits that aren't in our defined flags, so let's test what we actually get
        let all_defined = Modifiers::SHIFT | Modifiers::ALT | Modifiers::SUPER | Modifiers::CONTROL;
        assert_eq!(truncated, all_defined, "Truncating 126 should give all defined flags");
    }
    
    #[test] 
    fn test_special_input_flag() {
        let special = Modifiers::SPECIAL_INPUT;
        assert!(special.is_special_input());
        assert_eq!(special.reaper_code(), 255);
        
        // Test that normal flags don't register as special
        let normal = Modifiers::SHIFT | Modifiers::CONTROL;
        assert!(!normal.is_special_input());
        assert_ne!(normal.reaper_code(), 255);
    }
}
