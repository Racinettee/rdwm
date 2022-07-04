use std::{collections::LinkedList, sync::Mutex};

use fontconfig_sys::{FcPatternGetBool, constants::FC_COLOR, FcBool};
use libc::c_void;
use x11::{
    xlib::{Display, Window, Drawable, GC, XCreatePixmap, XDefaultDepth, XCreateGC, XSetLineAttributes, LineSolid, CapButt, JoinMiter},
    xft::{XftFont, FcPattern, XftFontOpenName, XftNameParse, XftFontClose, XftFontOpenPattern}
};

use super::{xin, EMPTY_SCREEN_INFO, ScreenInfoExt};

pub struct Layout {
    pub symbol:  String,
    pub arrange: fn (&mut Monitor),
}

pub struct Client<'a> {
    pub name: [i8; 256],
    pub mina: f32, pub maxa: f32,
    pub x: i32, pub y: i32, pub w: i32, pub h: i32,
    pub oldx: i32, pub oldy: i32, pub oldw: i32, pub oldh: i32,
    pub basew: i32, pub baseh: i32,
    pub incw: i32, pub inch: i32,
    pub maxw: i32, pub maxh: i32,
    pub minw: i32, pub minh: i32,
    pub bw: i32, pub oldbw: i32,
    pub tags: u32,
    pub isfixed: i32, pub isfloating: i32, pub isurgent: i32,
    pub neverfocus: i32, pub oldstate: i32, pub isfullscreen: i32,
	pub next: *mut Self,
	pub snext: *mut Self,
	pub mon: i32,
	pub win: Window,
}

pub struct Monitor<'a> {
	pub ltsymbol: [u8; 16],
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
	pub showbar: i32,
	pub topbar:  i32,
	pub clients: LinkedList<Client<'a>>,
	pub sel:     *mut Client<'a>,
	pub stack:   *mut Client<'a>,
	//pub next:    *mut Monitor,
	pub barwin:  Window,
	pub lt:      [*const Layout; 2]
}

pub static mut SELMON: i32 = 0;
pub static mut SW: i32 = 0;
pub static mut SH: i32 = 0;

pub struct Fnt {
    pub dpy: *mut Display,
    pub h: u32,
    pub xfont: *mut XftFont,
    pub pattern: *mut FcPattern,
    pub next: Option<Box<Self>>,
}

impl Default for Fnt {
    fn default() -> Self {
        Fnt {
            dpy: std::ptr::null_mut(),
            xfont: std::ptr::null_mut(),
            pattern: std::ptr::null_mut(),
            h: 0, next: None
        }
    }
}

pub struct Drw<'a> {
    pub dpy: *mut Display,
    pub w: u32, pub h: u32,
    pub screen: i32,
    pub root: Window,
    pub drawable: Drawable,
    pub gc: GC,
    //pub scheme: *mut Clr,
    pub fonts: Box<Fnt>,

    pub mons: &'a mut LinkedList<Monitor<'a>>,
}

impl<'a> Drw<'a> {
    pub fn create(display: *mut Display, mons: &'a mut LinkedList<Monitor<'a>>, screen: i32, root: Window, w: u32, h: u32) -> Self {
        let result = Drw {
            dpy: display,
            w: w,
            h: h,
            root: root,
            screen: screen,
            drawable: unsafe { XCreatePixmap(display, root, w, h, XDefaultDepth(display, screen) as u32) },
            gc: unsafe { XCreateGC(display, root, 0, std::ptr::null_mut()) },
            fonts: Default::default(),

            mons: mons,
        };
        //scheme: todo!(),
        //fonts: Default::default(),
        unsafe { XSetLineAttributes(display, result.gc, 1, LineSolid, CapButt, JoinMiter) };
        result
    }
    pub fn fontset_create(&mut self, fonts: &[&str]) -> Option<&Fnt> {
        if fonts.is_empty() {
            return None
        }
        let mut cur: Box<Fnt>;
        let mut ret: Box<Fnt> = Box::default();
        for font in fonts.iter().rev() {
            if let Some(fnt) = self.xfont_create(font, std::ptr::null_mut()) {
                cur = fnt;
                cur.next = Some(ret);
                ret = cur;
            }
        }
        self.fonts = ret;
        Some(self.fonts.as_ref())
    }
    fn xfont_create(&mut self, fontname: &str, fontpattern: *mut FcPattern) -> Option<Box<Fnt>> {
        let (xfont, pattern) = unsafe { 
            let xfont: *mut XftFont;
            let mut pattern: *mut FcPattern = std::ptr::null_mut();
            if !fontname.is_empty() {
                let mut fontname0:Vec<_> = fontname.as_bytes().into();
                fontname0.push(0);
                xfont = XftFontOpenName(self.dpy, self.screen, fontname0.as_ptr() as *const i8);
                if xfont == std::ptr::null_mut() {
                    eprintln!("error cannot load font from {}", fontname);
                    return None
                }
                pattern = XftNameParse(fontname0.as_ptr() as *const i8);
                if pattern == std::ptr::null_mut() {
                    eprintln!("error cannot parse font name to pattern: {}", fontname);
                    XftFontClose(self.dpy, xfont);
                    return None
                }
            } else if fontpattern != std::ptr::null_mut() {
                xfont = XftFontOpenPattern(self.dpy, fontpattern);
                if xfont == std::ptr::null_mut() {
                    eprintln!("error, cannot load font from pattern");
                    return None
                }
            } else {
                eprintln!("error, no font specified");
                std::process::exit(-1);
            }
    
            /* Do not allow using color fonts. This is a workaround for a BadLength
             * error from Xft with color glyphs. Modelled on the Xterm workaround. See
             * https://bugzilla.redhat.com/show_bug.cgi?id=1498269
             * https://lists.suckless.org/dev/1701/30932.html
             * https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=916349
             * and lots more all over the internet.
             */
            let mut iscol: FcBool = 0;
            if FcPatternGetBool((*xfont).pattern as *mut c_void, FC_COLOR.as_ptr(), 0, &mut iscol as *mut FcBool) != 0 && iscol != 0 {
                XftFontClose(self.dpy, xfont);
                return None
            }
            (xfont, pattern)
        };
        Some(Box::new(Fnt {
            xfont: xfont,
            pattern: pattern,
            h: (unsafe {(*xfont).ascent + (*xfont).descent}) as u32,
            dpy: self.dpy,
            next: None
        }))
    }

    pub fn updategeom(&mut self) -> bool {
        let mut dirty = false;
        if xin::is_active(self.dpy) {
            let screen_info = xin::Screens::get_screen_info(self.dpy);
            let mut unique = Vec::with_capacity(screen_info.len() as usize);
            unique.resize(screen_info.len(), EMPTY_SCREEN_INFO);
            let n = self.mons.len();
            let mut j = 0;
            unsafe {
                // only consider unique geometries as seperate screens
                for i in 0..screen_info.len() {
                    if Self::is_unique_geom(&unique[0..j], screen_info[i as usize]) {
                        unique[j] = screen_info[i as usize];
                        j += 1;
                    }
                }
                drop(screen_info);
                let nn = j;
                if n < nn { // new monitors available
                    for _ in 0..(nn - n) {
                        self.mons.push_back(Self::createmon());
                    }
                    for (i, m) in self.mons.iter_mut().enumerate() {
                        if i >= n
                        || unique[i].x_org as i32 != m.mx || unique[i].y_org as i32 != m.my
                        || unique[i].width as i32 != m.mw || unique[i].height as i32 != m.mh {
                            dirty = true;
                            m.num = i as i32;
                            m.mx = unique[i].x_org as i32;
                            m.wx = unique[i].x_org as i32;
                            m.my = unique[i].y_org as i32;
                            m.wy = unique[i].y_org as i32;
                            m.mw = unique[i].width as i32;
                            m.mh = unique[i].height as i32;
                            m.ww = unique[i].width as i32;
                            m.wh = unique[i].height as i32;
                            Self::updatebarpos(m);
                        }
                    }
                } else { // less monitors available
                    for _ in nn..n {
                        let last_monitor = self.mons.back_mut().unwrap();
                        for c in &mut last_monitor.clients {
                            dirty = true;
                            Self::detach_stack(c);
                            c.mon = self.mons.front().unwrap().num;
                            Self::attach(c);
                            Self::attachstack(c);
                        }
                        if last_monitor.num == SELMON {
                            SELMON = self.mons.front().unwrap().num;
                        }
                        Self::cleanup_mon(last_monitor);
                    }
                }
                drop(unique);
            }
        } else {
            if self.mons.is_empty() {
                self.mons.push_back(Self::createmon());
            }
            let first_mon = self.mons.front_mut().unwrap();
            if unsafe { first_mon.mw != SW || first_mon.mh != SH } {
                dirty = true;
                unsafe {
                    first_mon.mw = SW;
                    first_mon.ww = SW;
                    first_mon.mh = SH;
                    first_mon.wh = SH;
                }
                Self::updatebarpos(first_mon);
            }
        }
        if dirty {
            unsafe {
                SELMON = self.mons.front().unwrap().num;
                SELMON = Self::wintomon(self.root).num;
            }
        }
        dirty
    }

    fn is_unique_geom(unique: &[xin::ScreenInfo], info: xin::ScreenInfo) -> bool {
        for screen in unique.iter().rev() {
            if screen.compare_geom(info) {
                return false
            }
        }
        true
    }
    fn createmon() -> Monitor<'a> {
        todo!()
    }
    fn updatebarpos(_m: &mut Monitor) {
        todo!()
    }
    fn detach_stack(_c: &Client) {
        todo!()
    }
    fn attach(_c: &Client) {
        todo!()
    }
    fn attachstack(_c: &Client) {
        todo!()
    }
    fn cleanup_mon(_m: &Monitor) {
        todo!()
    }
    fn wintomon(w: Window) -> &'a Monitor<'a> {
        todo!()
    }
}