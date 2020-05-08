use std::rc::Rc;
use std::cell::Cell;
use super::gl::{ Gl };
use super::settings::{ Settings };

use web_sys::{
    HtmlImageElement,
    WebGlTexture,
    WebGlRenderingContext as Context,
};
use num_enum::{
    TryFromPrimitive,
    IntoPrimitive,
};

#[repr(i32)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
pub enum TextureFilter {
    Nearest = Context::NEAREST as i32,
    Linear = Context::LINEAR as i32,
}

impl Default for TextureFilter {
    fn default() -> Self {
        TextureFilter::Linear
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
pub enum TextureType {
    Byte = Context::UNSIGNED_BYTE,
    Float = Context::FLOAT,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
pub enum TextureFormat {
    Alpha = Context::ALPHA,
    Luminance = Context::LUMINANCE,
    LuminanceAlpha = Context::LUMINANCE_ALPHA,
    Rgb = Context::RGB,
    Rgba = Context::RGBA,
}

#[derive(Debug)]
pub enum TextureContent {
    None,
    Image(HtmlImageElement),
    Bytes(Vec<u8>),
}

pub const TEXTURES_COUNT: u32 = 16;

#[derive(Debug)]
pub struct TextureInfo {
    pub(self) gl: Gl,
    pub(super) handle: WebGlTexture,
    pub(self) width: u32,
    pub(self) height: u32,
    pub(self) data_type: TextureType,
    pub(self) format: TextureFormat,
    pub(self) filter: Cell<TextureFilter>,
}

impl PartialEq<TextureInfo> for TextureInfo {
    fn eq(&self, other: &TextureInfo) -> bool {
        self.handle == other.handle
    }
}

impl Eq for TextureInfo {}

impl Drop for TextureInfo {
    fn drop(&mut self) {
        self.gl.context().delete_texture(Some(&self.handle))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Texture {
    pub(super) data: Rc<TextureInfo>,
}

impl Texture {

    pub fn new(gl: Gl, width: u32, height: u32, data_type: TextureType, format: TextureFormat, data: TextureContent) -> Texture {
        let handle = gl.context().create_texture().unwrap();
        let result = Texture {
            data: Rc::new(TextureInfo {
                gl: gl.clone(),
                handle: handle.clone(),
                width: width,
                height: height,
                data_type: data_type,
                format: format,
                filter: Default::default()
            }),
        };

        let format: u32 = format.into();

        gl.apply(
            Gl::settings().texture(0, result.clone()),
            || {
                match data {
                    TextureContent::None => {
                        gl.context().tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                            Context::TEXTURE0,
                            0,
                            format as i32,
                            width as i32,
                            height as i32,
                            0,
                            format,
                            data_type.into(),
                            None,
                        ).unwrap();
                    },
                    TextureContent::Image(image) => {
                        gl.context().tex_image_2d_with_u32_and_u32_and_image(
                            Context::TEXTURE0,
                            0,
                            format as i32,
                            format,
                            data_type.into(),
                            &image
                        ).unwrap();
                    },
                    TextureContent::Bytes(bytes) => {
                        gl.context().tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                            Context::TEXTURE0,
                            0,
                            format as i32,
                            width as i32,
                            height as i32,
                            0,
                            format,
                            data_type.into(),
                            Some(&bytes),
                        ).unwrap();
                    }
                };
            }
        );
        
        return result;
    }

    pub fn gl(&self) -> Gl {
        self.data.gl.clone()
    }

    pub fn width(&self) -> u32 {
        self.data.width
    }
    pub fn height(&self) -> u32 {
        self.data.height
    }
    pub fn data_type(&self) -> TextureType {
        self.data.data_type
    }
    pub fn format(&self) -> TextureFormat {
        self.data.format
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    pub fn filter(&self) -> TextureFilter {
        self.data.filter.get()
    }

    pub fn set_filter(&self, filter: TextureFilter) {
        if self.filter() != filter {
            let ref gl = self.data.gl;
            let context = gl.context();
            gl.apply(
                Gl::settings().texture(0, self.clone()),
                || {
                    context.tex_parameteri(Context::TEXTURE0, Context::TEXTURE_MAG_FILTER, filter.into());
                    context.tex_parameteri(Context::TEXTURE0, Context::TEXTURE_MIN_FILTER, filter.into());
                    self.data.filter.set(filter);
                }
            );
        }
    }
}
