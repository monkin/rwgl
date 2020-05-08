use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use std::cell::RefCell;
use web_sys::{
    WebGlRenderingContext as Context,
    HtmlCanvasElement,
    AngleInstancedArrays,
};

use super::settings::Settings;
use super::settings::EmptySetting;
use super::settings::SettingsCache;

#[derive(Debug)]
pub(self) struct GlInfo {
    pub(super) context: Context,
    pub(self) settings_cache: RefCell<SettingsCache>,
    pub(super) ex_instanced_arrays: AngleInstancedArrays,
}

#[derive(Clone, Debug)]
pub struct Gl {
    pub(self) data: Rc<GlInfo>,
}

impl Gl {
    pub(self) fn get_extension<Ex: JsCast>(context: &Context, name: &str) -> Ex {
        context.get_extension(name).unwrap().unwrap().unchecked_into()
    }

    pub fn new(canvas: &HtmlCanvasElement) -> Gl {
        let context = Context::from(JsValue::from(canvas.get_context("webgl").unwrap().unwrap()));
        Gl {
            data: Rc::new(GlInfo {
                ex_instanced_arrays: Gl::get_extension(&context, "ANGLE_instanced_arrays"),
                settings_cache: Default::default(),
                context: context,
            })
        }
    }

    pub fn context(&self) -> &Context {
        &self.data.context
    }

    pub fn settings() -> impl Settings {
        EmptySetting {}
    }

    pub fn apply<R>(&self, settings: impl Settings, callback: impl FnOnce() -> R) -> R {
        settings.apply(self, &self.data.settings_cache, callback)
    }
}
