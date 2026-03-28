pub mod egui_impl;

use crate::protocol::{A2NInput, A2NOutput};

pub trait Renderer {
    fn run(self, input: A2NInput) -> A2NOutput;
}
