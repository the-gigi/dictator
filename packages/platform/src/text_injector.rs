use std::sync::Mutex;
use object_model::error::DictatorError;
use object_model::Injector;

pub struct TextInjector {
}

impl TextInjector {
    pub fn new() -> TextInjector {
        TextInjector {}
    }

    fn get_focused_window() {

    }
}

impl Injector for TextInjector {
    fn inject(&self, text: &str) -> Option<DictatorError> {


        None
    }
}
