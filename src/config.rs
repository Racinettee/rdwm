use crate::rxl::{tile, monocle, Layout};

pub static FONTS: &[&str] = &[
    "monospace:size=10",
];
// factor of master area size [0.05..0.95]
pub const MFACT: f32 = 0.55;
// the number of clients allowed in the master area
pub const NMASTER: i32 = 1;
// false means no bar
pub const SHOWBAR: bool = true;
// false means bottom bar
pub const TOPBAR: bool = true;

pub const LAYOUTS: &[Layout] = &[
    // symbol   arrange function
    Layout{ symbol: "[]=", arrange: Some(tile) }, // first entry is default
    Layout{ symbol: "><>", arrange: None }, // no layout fn means float
    Layout{ symbol: "[M]", arrange: Some(monocle) },
];