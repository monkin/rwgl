use std::fmt::Debug;
use std::ops::DerefMut;
use std::ops::Deref;
use std::cell::RefCell;
use web_sys::WebGlRenderingContext as Context;

use super::gl::Gl;
use super::texture::Texture;
use super::texture::TextureFilter;
use super::data_buffer::ArrayBuffer;

#[derive(Clone, Debug, Default)]
pub struct SettingsCache {
    blend: BlendSetting,
    depth: DepthTestSetting,
    active_texture: ActiveTextureSetting,
    array_buffer: ArrayBufferSetting,
    textures: [Option<Texture>; 16],
}

pub trait Settings
where
    Self: PartialEq,
    Self: Debug,
    Self: Clone,
{
    fn apply<R, F: FnOnce() -> R>(&self, gl: &Gl, cache: &RefCell<SettingsCache>, callback: F) -> R;

    fn depth_test(self, value: bool) -> ComposedSetting<Self, DepthTestSetting> {
        ComposedSetting(self, DepthTestSetting(value))
    }

    fn blend(self, value: bool) -> ComposedSetting<Self, BlendSetting> {
        ComposedSetting(self, BlendSetting(value))
    }

    fn texture(self, index: u32, texture: Texture) -> ComposedSetting<Self, TextureSetting> {
        ComposedSetting(self, TextureSetting {
            index: index,
            texture: Some(texture),
        })
    }

    fn texture_filter(self, texture: Texture, filter: TextureFilter) -> ComposedSetting<Self, TextureFilterSetting> {
        ComposedSetting(self, TextureFilterSetting {
            texture: texture,
            filter: filter,
        })
    }

    fn array_buffer(self, array_buffer: ArrayBuffer) -> ComposedSetting<Self, ArrayBufferSetting> {
        ComposedSetting(self, ArrayBufferSetting(Some(array_buffer)))
    }
}

pub trait CachedSettings {
    fn set(gl: &Gl, value: &Self);
    fn get_cached(cache: &impl Deref<Target = SettingsCache>) -> Self;
    fn set_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self);
}

impl <T> Settings for T
where
    T: PartialEq,
    T: Debug,
    T: Clone,
    T: CachedSettings,
{
    fn apply<R, F: FnOnce() -> R>(&self, gl: &Gl, cache: &RefCell<SettingsCache>, callback: F) -> R
    {
        let old_value = Self::get_cached(&cache.borrow());
        return if self == &old_value {
            callback()
        } else {
            Self::set_cached(&mut cache.borrow_mut(), self);
            Self::set(gl, self);
            let result = callback();
            Self::set(gl, &old_value);
            Self::set_cached(&mut cache.borrow_mut(), &old_value);
            result
        }
    }
}

#[derive(Default, PartialEq, Debug, Clone)]
pub struct EmptySetting {}

impl Settings for EmptySetting {
    fn apply<R, F: FnOnce() -> R>(&self, _: &Gl, _: &RefCell<SettingsCache>, callback: F) -> R
    {
        callback()
    }
}

#[derive(Default, PartialEq, Debug, Clone)]
pub struct ComposedSetting<S1: Settings, S2: Settings>(S1, S2);

impl <S1: Settings, S2: Settings> Settings for ComposedSetting<S1, S2> {
    fn apply<R, F: FnOnce() -> R>(&self, gl: &Gl, cache: &RefCell<SettingsCache>, callback: F) -> R
    {
        self.0.apply(gl, cache, || {
            self.1.apply(gl, cache, || callback())
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActiveTextureSetting(u32);

impl CachedSettings for ActiveTextureSetting {
    fn set(gl: &Gl, value: &Self) {
        gl.context().active_texture(value.0 + Context::TEXTURE0);
    }
    fn get_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.active_texture
    }
    fn set_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.active_texture = *value;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ArrayBufferSetting(Option<ArrayBuffer>);

impl CachedSettings for ArrayBufferSetting {
    fn set(gl: &Gl, value: &Self) {
        gl.context().bind_buffer(Context::ARRAY_BUFFER, value.0.as_ref().map(|v| v.handle()).as_ref());
    }
    fn get_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.array_buffer.clone()
    }
    fn set_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.array_buffer = value.clone();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlendSetting(bool);

impl CachedSettings for BlendSetting {
    fn set(gl: &Gl, value: &Self) {
        if value.0 {
            gl.context().enable(Context::BLEND)
        } else {
            gl.context().disable(Context::BLEND)
        }
    }
    fn get_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.blend
    }
    fn set_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.blend = *value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DepthTestSetting(bool);

impl CachedSettings for DepthTestSetting {
    fn set(gl: &Gl, value: &Self) {
        if value.0 {
            gl.context().enable(Context::DEPTH_TEST)
        } else {
            gl.context().disable(Context::DEPTH_TEST)
        }
    }
    fn get_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        cache.depth
    }
    fn set_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        cache.depth = *value;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextureSetting {
    index: u32,
    texture: Option<Texture>,
}

impl TextureSetting {
    pub(self) fn set_texture(gl: &Gl, index: u32, texture: Option<&Texture>) {
        gl.context().bind_texture(
            Context::TEXTURE0 + index,
            texture.map(|texture| texture.data.handle.clone()).as_ref()
        );
    }
}

impl Settings for TextureSetting {
    fn apply<R, F: FnOnce() -> R>(&self, gl: &Gl, cache: &RefCell<SettingsCache>, callback: F) -> R {
        let previous = cache.borrow().textures[self.index as usize].clone();
        cache.borrow_mut().textures[self.index as usize] = self.texture.clone();
        Self::set_texture(gl, self.index, self.texture.as_ref());
        let result = callback();
        Self::set_texture(gl, self.index, previous.as_ref());
        cache.borrow_mut().textures[self.index as usize] = previous;
        return result;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureFilterSetting {
    texture: Texture,
    filter: TextureFilter,
}

impl Settings for TextureFilterSetting {
    fn apply<R, F: FnOnce() -> R>(&self, _: &Gl, _: &RefCell<SettingsCache>, callback: F) -> R {
        let previous = self.texture.filter();
        let current = self.filter;
        self.texture.set_filter(current);
        let result = callback();
        self.texture.set_filter(previous);
        return result;
    }
}


