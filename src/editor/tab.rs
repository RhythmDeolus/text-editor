use crate::editor::window::Window;

use std::rc::Weak;

pub struct Tab {
    name: String,
    windows: Vec<Window>,
    focused_window: Weak<Window>,
}
