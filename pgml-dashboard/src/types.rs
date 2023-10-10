use sailfish::runtime::{Buffer, Render};
use std::ops::{Deref, DerefMut};

// This is a custom wrapper around Option so we can implement render for it
pub struct CustomOption<T>(Option<T>);

impl<T> Default for CustomOption<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> CustomOption<T> {
    pub fn new(value: T) -> Self {
        Self(Some(value))
    }
}

impl<T> From<T> for CustomOption<T> {
    fn from(value: T) -> Self {
        Self(Some(value))
    }
}

impl<T> Deref for CustomOption<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for CustomOption<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Render> Render for CustomOption<T> {
    fn render(&self, b: &mut Buffer) -> Result<(), sailfish::RenderError> {
        match &self.0 {
            Some(value) => value.render(b),
            None => Ok(()),
        }
    }
}
