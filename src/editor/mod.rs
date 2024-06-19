mod buff;
mod window;
mod tab;
mod command_buffer;

use crate::editor::buff::Buffer;
use crate::editor::tab::Tab;
use crate::editor::command_buffer::CommandBuffer;

use std::rc::Weak;

enum HybridTab {
    Tab(Weak<Tab>),
    CommandBuffer,
}

pub struct Editor {
    buffers: Vec<Buffer>,
    tabs: Vec<Tab>,
    command_buffer: CommandBuffer,
    focused_tab: HybridTab,
}

impl Editor {
    // TODO: Display tabs, windows, breadcrumbs, on the screen using methods
}
