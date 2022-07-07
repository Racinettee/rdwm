use fontconfig_sys::{FcPatternGetBool, constants::FC_COLOR, FcBool};
use libc::c_void;
use x11::{
    xlib::{Display, Window, Drawable, GC, XCreatePixmap, XDefaultDepth, XCreateGC, XSetLineAttributes, LineSolid, CapButt, JoinMiter},
    xft::{XftFont, FcPattern, XftFontOpenName, XftNameParse, XftFontClose, XftFontOpenPattern}
};

use super::{xin, ScreenInfoExt, Settings, Monitor};

#[derive(Copy, Clone)]
pub struct Layout {
    pub symbol:  &'static str,
    pub arrange: Option<fn (&mut Monitor)>,
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

    pub settings: &'a mut Settings<'a>,
}

impl<'a> Drw<'a> {
    pub fn create(display: *mut Display, settings: &'a mut Settings<'a>, screen: i32, root: Window, w: u32, h: u32) -> Self {
        let result = Drw {
            dpy: display,
            w, h, root, screen,
            drawable: unsafe { XCreatePixmap(display, root, w, h, XDefaultDepth(display, screen) as u32) },
            gc: unsafe { XCreateGC(display, root, 0, std::ptr::null_mut()) },
            fonts: Default::default(),

            settings,
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
        let mut unique = Vec::new();
        if xin::is_active(self.dpy) {
            let n = self.settings.mons.len();
            xin::Screens::get_screen_info(self.dpy).iter()
            .for_each(|si| if si.is_unique_geom(&unique) { unique.push(*si) });
            // If there are more screens in unique than were in monitor we will create some new monitors
            if unique.len() > n {
                for _ in 0..(unique.len() - n) {
                    self.settings.mons.push_back(Monitor::create());
                }
                for (i, m) in self.settings.mons.iter_mut().enumerate() {
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
                        m.updatebarpos(self.settings.bh);
                    }
                }
            } else { // less monitors available
                for _ in 0..(unique.len() - n) {
                    let first_mon_num = self.settings.mons.front().unwrap().num;
                    let last_monitor = self.settings.mons.back_mut().unwrap();
                    for c in &mut last_monitor.clients {
                        dirty = true;
                        Self::detach_stack(c);
                        c.mon = first_mon_num;
                        Self::attach(c);
                        Self::attachstack(c);
                    }
                    unsafe {
                        if last_monitor.num == SELMON {
                            SELMON = first_mon_num;
                        }
                    }
                    Self::cleanup_mon(last_monitor);
                }
            }
            drop(unique);
        } else {
            if self.settings.mons.is_empty() {
                self.settings.mons.push_back(Monitor::create());
            }
            let first_mon = self.settings.mons.front_mut().unwrap();
            if unsafe { first_mon.mw != SW || first_mon.mh != SH } {
                dirty = true;
                unsafe {
                    first_mon.mw = SW;
                    first_mon.ww = SW;
                    first_mon.mh = SH;
                    first_mon.wh = SH;
                }
                first_mon.updatebarpos(self.settings.bh);
            }
        }
        if dirty {
            unsafe {
                SELMON = self.settings.mons.front().unwrap().num;
                SELMON = Self::wintomon(self.root).num;
            }
        }
        dirty
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
    fn wintomon(_w: Window) -> &'a Monitor<'a> {
        todo!()
    }
}

pub fn tile(_m: &mut Monitor) {
    todo!()
}

pub fn monocle(_m: &mut Monitor) {
    todo!()
}