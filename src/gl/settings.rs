use std::fmt::Debug;
use std::ops::DerefMut;
use std::ops::Deref;
use std::cell::RefCell;
use web_sys::WebGlRenderingContext as Context;

use super::gl::Gl;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SettingsCache {
    blend: BlendSetting,
    depth: DepthTestSetting,
}

pub trait Settings
where
    Self: Default,
    Self: PartialEq,
    Self: Default,
    Self: Debug,
    Self: Clone,
{
    fn set(gl: &Gl, value: &Self);
    fn get_cached(cache: &impl Deref<Target = SettingsCache>) -> Self;
    fn set_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self);

    fn depth_test(self, value: bool) -> ComposedSetting<Self, DepthTestSetting> {
        ComposedSetting(self, DepthTestSetting(value))
    }

    fn blend(self, value: bool) -> ComposedSetting<Self, BlendSetting> {
        ComposedSetting(self, BlendSetting(value))
    }

    fn apply<R, F>(&self, gl: &Gl, cache: &RefCell<SettingsCache>, callback: F) -> R
    where
        F: FnOnce() -> R
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
    fn set(_: &Gl, _: &Self) {}
    fn get_cached(_: &impl Deref<Target = SettingsCache>) -> Self {
        EmptySetting {}
    }
    fn set_cached(_: &mut impl DerefMut<Target = SettingsCache>, _: &Self) {}
}

#[derive(Default, PartialEq, Debug, Clone)]
pub struct ComposedSetting<S1: Settings, S2: Settings>(S1, S2);

impl <S1: Settings, S2: Settings> Settings for ComposedSetting<S1, S2> {
    fn set(gl: &Gl, value: &Self) {
        S1::set(gl, &value.0);
        S2::set(gl, &value.1);
    }
    fn get_cached(cache: &impl Deref<Target = SettingsCache>) -> Self {
        ComposedSetting(S1::get_cached(cache), S2::get_cached(cache))
    }
    fn set_cached(cache: &mut impl DerefMut<Target = SettingsCache>, value: &Self) {
        S1::set_cached(cache, &value.0);
        S2::set_cached(cache, &value.1);
    }

    fn apply<R, F>(&self, gl: &Gl, cache: &RefCell<SettingsCache>, callback: F) -> R
    where
        F: FnOnce() -> R
    {
        self.0.apply(gl, cache, || {
            self.1.apply(gl, cache, || callback())
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlendSetting(bool);

impl Settings for BlendSetting {
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

impl Settings for DepthTestSetting {
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
