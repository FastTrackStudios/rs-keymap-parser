use serde::{Deserialize, Serialize};
use std::fmt;

/// Special input types that use modifier code 255 in Reaper keymap files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialInput {
    /// Normal vertical mousewheel
    Mousewheel,
    /// Mousewheel with Ctrl modifier
    CtrlMousewheel,
    /// Mousewheel with Alt modifier  
    AltMousewheel,
    /// Mousewheel with Ctrl+Alt modifiers
    CtrlAltMousewheel,
    /// Mousewheel with Shift modifier
    ShiftMousewheel,
    /// Mousewheel with Ctrl+Shift modifiers
    CtrlShiftMousewheel,
    /// Mousewheel with Alt+Shift modifiers
    AltShiftMousewheel,
    /// Mousewheel with Ctrl+Alt+Shift modifiers
    CtrlAltShiftMousewheel,
    
    /// Horizontal mousewheel
    HorizWheel,
    /// Horizontal mousewheel with Alt modifier
    AltHorizWheel,
    /// Horizontal mousewheel with Ctrl modifier
    CtrlHorizWheel,
    /// Horizontal mousewheel with Ctrl+Alt modifiers
    CtrlAltHorizWheel,
    /// Horizontal mousewheel with Shift modifier
    ShiftHorizWheel,
    /// Horizontal mousewheel with Ctrl+Shift modifiers
    CtrlShiftHorizWheel,
    /// Horizontal mousewheel with Alt+Shift modifiers
    AltShiftHorizWheel,
    /// Horizontal mousewheel with Ctrl+Alt+Shift modifiers
    CtrlAltShiftHorizWheel,
    
    /// Multitouch zoom
    MultiZoom,
    /// Multitouch zoom with Ctrl
    CtrlMultiZoom,
    /// Multitouch zoom with Alt
    AltMultiZoom,
    /// Multitouch zoom with Ctrl+Alt+Shift
    CtrlAltShiftMultiZoom,
    
    /// Multitouch rotate
    MultiRotate,
    /// Multitouch rotate with Ctrl
    CtrlMultiRotate,
    
    /// Multitouch horizontal swipe
    MultiHorz,
    /// Multitouch vertical swipe
    MultiVert,
    
    /// Media keyboard keys (various values)
    MediaKey(u16),
    
    /// Unknown special input
    Unknown(u16),
}

impl SpecialInput {
    /// Convert a key code (used with modifier 255) to a SpecialInput
    pub fn from_key_code(key_code: u16) -> Self {
        match key_code {
            // Normal mousewheel
            120 | 248 => SpecialInput::Mousewheel,
            121 | 249 => SpecialInput::CtrlMousewheel,
            122 | 250 => SpecialInput::AltMousewheel,
            123 | 251 => SpecialInput::CtrlAltMousewheel,
            125 | 253 => SpecialInput::CtrlShiftMousewheel,
            252 => SpecialInput::ShiftMousewheel,
            254 => SpecialInput::AltShiftMousewheel,
            255 => SpecialInput::CtrlAltShiftMousewheel,
            
            // Horizontal mousewheel
            88 | 216 => SpecialInput::HorizWheel,
            90 | 218 => SpecialInput::AltHorizWheel,
            217 => SpecialInput::CtrlHorizWheel,
            219 => SpecialInput::CtrlAltHorizWheel,
            220 => SpecialInput::ShiftHorizWheel,
            221 => SpecialInput::CtrlShiftHorizWheel,
            222 => SpecialInput::AltShiftHorizWheel,
            223 => SpecialInput::CtrlAltShiftHorizWheel,
            
            // MultiZoom
            72 | 200 => SpecialInput::MultiZoom,
            73 | 201 => SpecialInput::CtrlMultiZoom,
            74 | 202 => SpecialInput::AltMultiZoom,
            207 => SpecialInput::CtrlAltShiftMultiZoom,
            
            // MultiRotate  
            24 | 152 => SpecialInput::MultiRotate,
            25 | 153 => SpecialInput::CtrlMultiRotate,
            
            // MultiSwipe
            40 | 168 => SpecialInput::MultiHorz,
            56 | 184 => SpecialInput::MultiVert,
            
            // Media keyboard keys (start at 232 and continue every 256)
            key if key >= 232 && (key - 232) % 256 == 0 => SpecialInput::MediaKey(key),
            key if key >= 488 => SpecialInput::MediaKey(key),
            
            // Unknown special input
            other => SpecialInput::Unknown(other),
        }
    }
    
    /// Convert back to the key code value
    pub fn to_key_code(self) -> u16 {
        match self {
            SpecialInput::Mousewheel => 248,
            SpecialInput::CtrlMousewheel => 249,
            SpecialInput::AltMousewheel => 250,
            SpecialInput::CtrlAltMousewheel => 251,
            SpecialInput::ShiftMousewheel => 252,
            SpecialInput::CtrlShiftMousewheel => 253,
            SpecialInput::AltShiftMousewheel => 254,
            SpecialInput::CtrlAltShiftMousewheel => 255,
            
            SpecialInput::HorizWheel => 216,
            SpecialInput::AltHorizWheel => 218,
            SpecialInput::CtrlHorizWheel => 217,
            SpecialInput::CtrlAltHorizWheel => 219,
            SpecialInput::ShiftHorizWheel => 220,
            SpecialInput::CtrlShiftHorizWheel => 221,
            SpecialInput::AltShiftHorizWheel => 222,
            SpecialInput::CtrlAltShiftHorizWheel => 223,
            
            SpecialInput::MultiZoom => 200,
            SpecialInput::CtrlMultiZoom => 201,
            SpecialInput::AltMultiZoom => 202,
            SpecialInput::CtrlAltShiftMultiZoom => 207,
            
            SpecialInput::MultiRotate => 152,
            SpecialInput::CtrlMultiRotate => 153,
            
            SpecialInput::MultiHorz => 168,
            SpecialInput::MultiVert => 184,
            
            SpecialInput::MediaKey(key) => key,
            SpecialInput::Unknown(key) => key,
        }
    }
}

impl fmt::Display for SpecialInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            SpecialInput::Mousewheel => "Mousewheel",
            SpecialInput::CtrlMousewheel => "Ctrl+Mousewheel",
            SpecialInput::AltMousewheel => "Alt+Mousewheel",
            SpecialInput::CtrlAltMousewheel => "Ctrl+Alt+Mousewheel",
            SpecialInput::ShiftMousewheel => "Shift+Mousewheel",
            SpecialInput::CtrlShiftMousewheel => "Ctrl+Shift+Mousewheel",
            SpecialInput::AltShiftMousewheel => "Alt+Shift+Mousewheel",
            SpecialInput::CtrlAltShiftMousewheel => "Ctrl+Alt+Shift+Mousewheel",
            
            SpecialInput::HorizWheel => "HorizWheel",
            SpecialInput::AltHorizWheel => "Alt+HorizWheel",
            SpecialInput::CtrlHorizWheel => "Ctrl+HorizWheel",
            SpecialInput::CtrlAltHorizWheel => "Ctrl+Alt+HorizWheel",
            SpecialInput::ShiftHorizWheel => "Shift+HorizWheel",
            SpecialInput::CtrlShiftHorizWheel => "Ctrl+Shift+HorizWheel",
            SpecialInput::AltShiftHorizWheel => "Alt+Shift+HorizWheel",
            SpecialInput::CtrlAltShiftHorizWheel => "Ctrl+Alt+Shift+HorizWheel",
            
            SpecialInput::MultiZoom => "MultiZoom",
            SpecialInput::CtrlMultiZoom => "Ctrl+MultiZoom",
            SpecialInput::AltMultiZoom => "Alt+MultiZoom", 
            SpecialInput::CtrlAltShiftMultiZoom => "Ctrl+Alt+Shift+MultiZoom",
            
            SpecialInput::MultiRotate => "MultiRotate",
            SpecialInput::CtrlMultiRotate => "Ctrl+MultiRotate",
            
            SpecialInput::MultiHorz => "MultiHorz",
            SpecialInput::MultiVert => "MultiVert",
            
            SpecialInput::MediaKey(key) => return write!(f, "MediaKey({})", key),
            SpecialInput::Unknown(key) => return write!(f, "Unknown({})", key),
        };
        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mousewheel_parsing() {
        assert_eq!(SpecialInput::from_key_code(248), SpecialInput::Mousewheel);
        assert_eq!(SpecialInput::from_key_code(120), SpecialInput::Mousewheel);
        assert_eq!(SpecialInput::from_key_code(249), SpecialInput::CtrlMousewheel);
        assert_eq!(SpecialInput::from_key_code(250), SpecialInput::AltMousewheel);
    }
    
    #[test]
    fn test_horizontal_wheel_parsing() {
        assert_eq!(SpecialInput::from_key_code(216), SpecialInput::HorizWheel);
        assert_eq!(SpecialInput::from_key_code(218), SpecialInput::AltHorizWheel);
        assert_eq!(SpecialInput::from_key_code(217), SpecialInput::CtrlHorizWheel);
    }
    
    #[test]
    fn test_round_trip() {
        let inputs = vec![
            SpecialInput::Mousewheel,
            SpecialInput::AltHorizWheel,
            SpecialInput::CtrlMultiZoom,
            SpecialInput::MultiVert,
        ];
        
        for input in inputs {
            let key_code = input.to_key_code();
            let parsed = SpecialInput::from_key_code(key_code);
            assert_eq!(input, parsed);
        }
    }
} 