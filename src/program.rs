use crate::color::Color;
use std::collections::HashMap;

pub enum ProgramArg {
    FloatArg(f32),
    ColorArg(Color),
}

pub struct Program {
    pub idx: u32,
    pub args: HashMap<String, ProgramArg>,
}

impl Program {
    pub fn new(idx: u32) -> Self {
        Self {
            idx,
            args: HashMap::with_capacity(16),
        }
    }

    pub fn set_arg(&mut self, name: &str, arg: ProgramArg) {
        self.args.insert(name.to_string(), arg);
    }
}