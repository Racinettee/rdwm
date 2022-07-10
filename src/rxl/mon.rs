use std::{collections::LinkedList, cmp::{max, min}};

use x11::xlib::Window;

use super::{Client, Layout};

pub struct Monitor {
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
	pub clients: LinkedList<Client>,
	pub sel:     *mut Client,
	pub stack:   *mut Client,
	//pub next:    *mut Monitor,
	pub barwin:  Window,
	pub lt:      [Layout; 2]
}

impl<'a> Monitor {
    pub fn create() -> Monitor {
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

	pub fn intersect(&self, x: i32, y: i32, w: i32, h: i32) -> i32 {
        max(0, min(x + w, self.wx + self.ww) - max(x, self.wx))
        * max(0, min(y + h, self.wy + self.wh) - max(y, self.wy))
    }
}

impl Default for Monitor {
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