use std::collections::BTreeMap;
use num_enum::{
    TryFromPrimitive,
    IntoPrimitive,
};
use web_sys::{
    WebGlShader,
    WebGlProgram,
    WebGlRenderingContext as Context,
    console,
};

use super::gl::Gl;

#[derive(Clone, Copy, Debug)]
struct AttributeInfo {
    location: i32,
    size_in_floats: u32,
}

#[derive(Debug, Clone)]
pub struct ProgramData {
    pub(self) gl: Gl,
    pub(self) handle: WebGlProgram,
    pub(self) attributes: BTreeMap<String, AttributeInfo>,
}