use crate::editor::buff::{Mode, Buffer};

use std::rc::Weak;

pub struct Window {
    buffer: Weak<Buffer>,
    bread_crumbs: String,
    height: f32, // from 0 to 1
    width: f32, // from 0 to 1
    mode: Mode,
}
