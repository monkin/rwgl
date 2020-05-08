use std::rc::Rc;
use web_sys::{
    WebGlRenderingContext as Context,
    WebGlBuffer,
};
use num_enum::{
    TryFromPrimitive,
    IntoPrimitive,
};

use super::Gl;
use super::settings::Settings;

#[repr(u32)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
pub enum BufferUsage {
    /**
     * The data store contents will be modified once and used at most a few times.
     */
    Stream = Context::STREAM_DRAW,
    /**
     * The data store contents will be modified once and used many times.
     */
    Static = Context::STATIC_DRAW,
    /**
     * The data store contents will be modified repeatedly and used many times.
     */
    Dynamic = Context::DYNAMIC_DRAW,
}

#[derive(Debug, Clone)]
pub struct ArrayBufferData {
    pub(self) gl: Gl,
    pub(self) handle: WebGlBuffer,
}

impl Drop for ArrayBufferData {
    fn drop(&mut self) {
        self.gl.context().delete_buffer(Some(&self.handle));
    }
}

#[derive(Debug, Clone)]
pub struct ArrayBuffer {
    pub(self) data: Rc<ArrayBufferData>
}

impl PartialEq<ArrayBuffer> for ArrayBuffer {
    fn eq(&self, other: &ArrayBuffer) -> bool {
        self.data.handle == other.data.handle
    }
}

impl Eq for ArrayBuffer {}

impl ArrayBuffer {
    pub fn new<T: Sized>(gl: Gl, data: &[T], usage: BufferUsage) -> ArrayBuffer {
        let ref context = gl.context();
        let buffer = context.create_buffer().unwrap();

        let result = ArrayBuffer {
            data: Rc::new(ArrayBufferData {
                gl: gl.clone(),
                handle: buffer,
            })
        };

        result.write(data, usage);

        return result;
    }

    pub(super) fn handle(&self) -> WebGlBuffer {
        self.data.handle.clone()
    }

    pub fn write<T: Sized>(&self, data: &[T], usage: BufferUsage) {
        self.data.gl.apply(
            Gl::settings().array_buffer(self.clone()),
            || {
                let bytes = unsafe {
                    std::slice::from_raw_parts(data as *const [T] as *const u8, std::mem::size_of_val(data))
                };
                self.data.gl.context().buffer_data_with_u8_array(
                    Context::ARRAY_BUFFER,
                    &bytes,
                    usage.into(),
                );
            }
        );
    }
}