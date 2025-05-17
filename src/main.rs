use rs_keymap_parser::parse::*;

fn main() {
    let lines = [
        "KEY 1 85 40760 4    # Main (alt-4) : U : OVERRIDE DEFAULT : Edit: Dynamic split items...",
        "KEY 37 71 40771 4  # Main (alt-4) : Shift+Control+G : Track: Toggle all track grouping enabled",
        "KEY 255 216 977 0  # Main : HorizWheel : OVERRIDE DEFAULT : View: Scroll horizontally reversed",
    ];
}
