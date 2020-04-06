use super::settings::Settings;
use super::settings::EmptySetting;
use wasm_bindgen::prelude::*;
use std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;
use web_sys::{
    WebGlRenderingContext as Context,
    HtmlCanvasElement,
};

use super::settings::SettingsCache;

pub(self) struct GlData {
    pub(super) context: Context,
    pub(self) settings_cache: RefCell<SettingsCache>,
}

#[derive(Clone)]
pub struct Gl {
    pub(self) data: Rc<GlData>,
}

impl Gl {
    pub fn new(canvas: &HtmlCanvasElement) -> Gl {
        Gl {
            data: Rc::new(GlData {
                context: Context::from(JsValue::from(canvas.get_context("webgl").unwrap().unwrap())),
                settings_cache: Default::default(),
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
