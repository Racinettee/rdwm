mod drw;
mod xin;
mod mon;
mod layout;
pub mod xconst;

pub use drw::*;
pub use xin::*;
pub use mon::*;
pub use layout::*;

use x11::xlib::{Display, Window, XQueryPointer};
use std::{collections::LinkedList, cmp::{max, min}};
// Runtime settings for holding the global variables
// we'd see in dwm.c
pub struct Settings {
    // X display screen geometry width, height
    pub sw: i32, pub sh: i32,
    pub bh: i32,
    pub mons: LinkedList<Monitor>,
    pub root: Window,
    pub dpy: *mut Display,
    pub selmon: i32,
}

impl<'a> Settings {
    pub fn win_to_mon(&'a self, w: Window) -> &'a Monitor {
        let mut x = 0;
        let mut y = 0;
        if w == self.root && self.get_root_ptr(&mut x, &mut y) {
            return self.rect_to_mon(x, y, 1, 1);
        }
        todo!()
    }

    pub fn get_root_ptr(&self, x: &mut i32, y: &mut i32) -> bool {
        let mut di = 0;
        let mut diu = 0u32;
        let mut dummy: Window = 0;
        let result = unsafe {
            XQueryPointer(
                self.dpy,
                self.root,
                &mut dummy as *mut Window,
                &mut dummy as *mut Window,
                x as *mut i32,
                y as *mut i32,
                &mut di as *mut i32,
                &mut di as *mut i32,
                &mut diu as *mut u32)
        };
        result != 0
    }

    pub fn rect_to_mon(&'a self, x: i32, y: i32, w: i32, h: i32) -> &'a Monitor {
        let mut area = 0;
        let mut r = self.mons.iter().find(|m| m.num == self.selmon).unwrap();
        for m in &self.mons {
            let a = m.intersect(x, y, w, h);
            if a > area {
                area = a;
                r = m;
            }
        }
        r
    }
}