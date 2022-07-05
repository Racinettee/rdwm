use std::{slice::{self, SliceIndex}, ops::Index};
use libc::c_void;
use x11::{xinerama::{XineramaScreenInfo, XineramaQueryScreens, XineramaIsActive}, xlib::{Display, XFree}};

pub type ScreenInfo = XineramaScreenInfo;

pub trait ScreenInfoExt {
    // returns true if same
    fn compare_geom(&self, other: Self) -> bool;
}

impl ScreenInfoExt for ScreenInfo {
    fn compare_geom(&self, other: Self) -> bool {
        self.x_org == other.x_org && self.y_org == other.y_org
            && self.width == other.width && self.height == other.height
    }
}

pub struct Screens<'a> {
    pub info: &'a mut [ScreenInfo]
}

impl<'a> Screens<'a> {
    pub fn get_screen_info(dpy: *mut Display) -> Screens<'a> {
        Screens { info: unsafe {
                let mut nn: i32 = 0;
                let info = XineramaQueryScreens(dpy, &mut nn as *mut i32);
                slice::from_raw_parts_mut(info, nn as usize)
            }
        }
    }
    pub fn len(&self) -> usize {
        self.info.len()
    }
}

impl<'a, Idx> Index<Idx> for Screens<'a>
where Idx: SliceIndex<[ScreenInfo], Output = ScreenInfo>, {
    type Output = ScreenInfo;

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

pub const EMPTY_SCREEN_INFO: XineramaScreenInfo = XineramaScreenInfo { screen_number: 0, x_org: 0, y_org: 0, width: 0, height: 0 };

pub fn is_active(dpy: *mut Display) -> bool {
    unsafe { XineramaIsActive(dpy) != 0 }
}