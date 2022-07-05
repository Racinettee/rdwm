mod drw;
mod xin;
use std::collections::LinkedList;

pub use drw::*;
pub use xin::*;

// Runtime settings for holding the global variables
// we'd see in dwm.c
pub struct Settings<'a> {
    // X display screen geometry width, height
    sw: i32, sh: i32,

    mons: LinkedList<Monitor<'a>>,
}