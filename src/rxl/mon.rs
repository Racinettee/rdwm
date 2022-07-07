use std::{collections::LinkedList};

use x11::xlib::Window;

use super::{Client, Layout, Settings};

pub struct Monitor<'a> {
	pub ltsymbol: &'static str,
	pub mfact:    f32,
	pub nmaster:  i32,
	pub num:      i32,
	pub by:       i32,               /* bar geometry */
	pub mx: i32, pub my: i32,
    pub mw: i32, pub mh: i32,   /* screen size */
	pub wx: i32, pub wy: i32,
    pub ww: i32, pub wh: i32,   /* window area  */
	pub seltags: u32,
	pub sellt:   u32,
	pub tagset:  [u32; 2],
	pub showbar: bool,
	pub topbar:  bool,
	pub clients: LinkedList<Client<'a>>,
	pub sel:     *mut Client<'a>,
	pub stack:   *mut Client<'a>,
	//pub next:    *mut Monitor,
	pub barwin:  Window,
	pub lt:      [Layout; 2]
}

impl<'a> Monitor<'a> {
    pub fn create() -> Monitor<'a> {
		Self::default()
    }
	pub fn updatebarpos(&mut self, barheight: i32) {
        self.wy = self.my;
		self.wh = self.mh;
		if self.showbar {
			self.wh -= barheight;
			self.by = if self.topbar { self.wy } else { self.wy + self.wh };
			self.wy = if self.topbar { self.wy + barheight } else { self.wy };
		} else {
			self.by = -barheight;
		}
    }
}

impl Default for Monitor<'_> {
    fn default() -> Self {
        Self { 
			tagset: [1, 1],
			mfact: crate::config::MFACT,
			nmaster: crate::config::NMASTER,
			showbar: crate::config::SHOWBAR,
			topbar: crate::config::TOPBAR,
			lt: [crate::config::LAYOUTS[0],
				crate::config::LAYOUTS[1 & crate::config::LAYOUTS.len()]],
			ltsymbol: crate::config::LAYOUTS[0].symbol,
			clients: LinkedList::new(), 
			num: Default::default(), 
			by: Default::default(),
			mx: Default::default(), my: Default::default(),
			mw: Default::default(), mh: Default::default(),
			wx: Default::default(), wy: Default::default(),
			ww: Default::default(), wh: Default::default(), 
			seltags: Default::default(),
			sellt: Default::default(),
			sel: std::ptr::null_mut(),
			stack: std::ptr::null_mut(),
			barwin: Default::default(),
		}
    }
}