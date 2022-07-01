use x11::{xlib::{Display, XCloseDisplay, XErrorEvent, XOpenDisplay, XDefaultRootWindow, XSync, SubstructureRedirectMask, BadWindow, BadMatch, BadDrawable, BadAccess, XDefaultScreen, XDisplayWidth, XDisplayHeight, XRootWindow}, xinerama::XineramaIsActive};
use libc::{signal, 
    //setsid, fork, close,
    waitpid, SIGCHLD, SIG_ERR, WNOHANG};

mod config;
mod rxl;

use config::*;

static mut XERRORXLIB: Option<unsafe extern "C" fn(*mut Display, *mut XErrorEvent) -> i32> = None;
fn check_other_wm(display: *mut Display) -> Result<(), &'static str> {
    use x11::xlib::{
        XSetErrorHandler, XSelectInput
    };
    // Get the original error handling function
    unsafe {
        XERRORXLIB = XSetErrorHandler(Some(xerrorstart));
        // this causes an error if another window manager is running
        XSelectInput(display, XDefaultRootWindow(display), SubstructureRedirectMask);
        XSync(display, 0);
        XSetErrorHandler(Some(xerror));
        XSync(display, 0);
    }

    Ok(())
}

fn setup(display: *mut Display) -> Result<(), &'static str> {
    // clean up any zombie processes
    sigchld(0);

    unsafe {
        let screen = default_screen(display);
        let sw = XDisplayWidth(display, screen);
        let sh = XDisplayHeight(display, screen);
        let root = XRootWindow(display, screen);
        let mut drw = rxl::Drw::create(display, screen, root, sw as u32,sh as u32);
        if let None = drw.fontset_create(FONTS) {
            return Err("no founts could be loaded")
        }
        let lrpad = drw.fonts.h;
        let bh = drw.fonts.h + 2;
        drw.updategeom();
    }
    Ok(())
}

fn scan() -> Result<(), &'static str> {

    Ok(())
}

fn run() -> Result<(), &'static str> {

    Ok(())
}

fn cleanup() -> Result<(), &'static str> {

    Ok(())
}

//use std::sync::mpsc::*;
//use std::sync::*;
//static mut RESULT_SENDER: Option<Mutex<Sender<()>>> = None;

fn main() -> Result<(), &'static str> {
    let display = connect_display()?;
    // check command args

    // handle signals
    install_sighandle()?;
    // check other wm
    check_other_wm(display)?;
    // setup
    setup(display)?;
    // scan
    scan()?;
    // run
    run()?;
    // clean up
    cleanup()?;
    // close x display
    disconnect_display(display);
    Ok(())
}

fn install_sighandle() -> Result<(), &'static str> {
    unsafe {
        if signal(SIGCHLD, sigchld as usize) == SIG_ERR {
            return Err("could not install signal handler")
        }
    }
    Ok(())
}
extern "C" fn sigchld(_: i32) {
    let mut nilstatus: libc::c_int = 0;
    while 0 < unsafe { waitpid(-1, &mut nilstatus as *mut libc::c_int, WNOHANG) } {}
}
extern "C" fn xerrorstart(_display: *mut Display, _ee: *mut XErrorEvent) -> i32 {
    eprintln!("another wm is already running");
    std::process::exit(-1);
}

const X_SETINPUTFOCUS: u8 = 42;
const X_POLYTEXT8: u8 = 74;
const X_POLYFILLRECTANGLE: u8 = 70;
const X_POLYSEGMENT: u8 = 66;
const X_CONFIGUREWINDOW: u8 = 12;
const X_GRABBUTTON: u8 = 28;
const X_GRABKEY: u8 = 33;
const X_COPYAREA: u8 = 62;

unsafe extern "C" fn xerror(display: *mut Display, ee: *mut XErrorEvent) -> i32 {
    #[allow(non_upper_case_globals)]
    match ((*ee).request_code, (*ee).error_code) {
        (X_SETINPUTFOCUS,   BadMatch) |
        (X_POLYTEXT8,       BadDrawable) |
        (X_POLYFILLRECTANGLE, BadDrawable) |
        (X_POLYSEGMENT,     BadDrawable) |
        (X_CONFIGUREWINDOW, BadMatch) |
        (X_GRABBUTTON,      BadAccess) |
        (X_GRABKEY,         BadAccess) |
        (X_COPYAREA,        BadDrawable) |
        (_, BadWindow) => return 0,
        (_, _) => ()
    }
    eprintln!("rdwm: fatal error: request code={}, error code={}", (*ee).request_code, (*ee).error_code);
    return XERRORXLIB.unwrap()(display, ee); /* may call exit */
}

fn connect_display() -> Result<*mut Display, &'static str> {
    let display = unsafe { XOpenDisplay(std::ptr::null()) };

    if display == std::ptr::null_mut() {
        return Err("failed to open dipsplay")
    }

    Ok(display)
}
fn disconnect_display(disp: *mut Display) {
    unsafe { XCloseDisplay(disp) };
}

fn default_screen(display: *mut Display) -> i32 {
    unsafe { XDefaultScreen(display) }
}

