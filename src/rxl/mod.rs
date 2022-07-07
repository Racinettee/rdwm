mod drw;
mod xin;
mod mon;
pub mod xconst;

pub use drw::*;
pub use xin::*;
pub use mon::*;

use std::collections::LinkedList;
// Runtime settings for holding the global variables
// we'd see in dwm.c
pub struct Settings<'a> {
    // X display screen geometry width, height
    pub sw: i32, pub sh: i32,
    pub bh: i32,
    pub mons: LinkedList<Monitor<'a>>,
}