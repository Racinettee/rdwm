use super::Monitor;

#[derive(Copy, Clone)]
pub struct Layout {
    pub symbol:  &'static str,
    pub arrange: Option<fn (&mut Monitor)>,
}