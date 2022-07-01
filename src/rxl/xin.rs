use std::{slice::{self, SliceIndex}, ops::Index};

use libc::c_void;
use x11::{xinerama::{XineramaScreenInfo, XineramaQueryScreens}, xlib::{Display, XFree, Screen}};

pub struct Screens<'a> {
    pub info: &'a mut [XineramaScreenInfo]
}

impl<'a> Screens<'a> {
    pub fn get_screen_info(dpy: *mut Display) -> Screens<'a> {
        let info = unsafe {
            let mut nn: i32 = 0;
            let info = XineramaQueryScreens(dpy, &mut nn as *mut i32);
            slice::from_raw_parts_mut(info, nn as usize)
        };
        Screens { info: info }
    }
    pub fn len(&self) -> usize {
        self.info.len()
    }
}

impl<'a, Idx> Index<Idx> for Screens<'a>
where Idx: SliceIndex<[XineramaScreenInfo], Output = XineramaScreenInfo>, {
    type Output = XineramaScreenInfo;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.info[index]
    }
}

impl Drop for Screens<'_> {
    fn drop(&mut self) {
        unsafe {
            XFree(self.info.as_mut_ptr() as *mut c_void);
        }
    }
}