use object_model::Injector;
use object_model::error::DictatorError;

use enigo::{Enigo, Keyboard, Settings};

pub struct TextInjector {}

impl TextInjector {
    pub fn new() -> TextInjector {
        TextInjector {}
    }
}

impl Injector for TextInjector {
    fn inject(&self, text: &str) -> Result<(), DictatorError> {
        let mut injector = Enigo::new(&Settings::default()).unwrap();
        injector
            .text(text)
            .map_err(|e| DictatorError::Injection(e.to_string()))
    }
}
