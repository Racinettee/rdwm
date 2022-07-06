mod drw;
mod xin;

pub mod xconst;

pub use drw::*;
pub use xin::*;

use std::collections::LinkedList;
// Runtime settings for holding the global variables
// we'd see in dwm.c
pub struct Settings<'a> {
    // X display screen geometry width, height
    pub sw: i32, pub sh: i32,

    pub mons: LinkedList<Monitor<'a>>,
}