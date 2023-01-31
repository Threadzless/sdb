use std::ops::{Deref, DerefMut};
use proc_macro::Span;
use syn::LitStr;
use proc_macro_error::{Diagnostic, Level, SpanRange};

pub struct Message {
    pub(crate) inner: Diagnostic,
    pub(crate) error: Option<Box<dyn std::error::Error>>,
}

impl Message {
    pub fn new_err( title: impl ToString ) -> Self {
        Self {
            inner: Diagnostic::new( Level::Error, title.to_string() ),
            error: None,
        }
    }

    pub fn new_warn( title: impl ToString ) -> Self {
        Self {
            inner: Diagnostic::new( Level::Warning, title.to_string() ),
            error: None,
        }
    }

    pub fn set_cause<E: std::error::Error + 'static>( &mut self, err: E ) -> &mut Self {
        self.error = Some( Box::new( err ) );
        self
    }

    // pub fn highlight(&mut self, ) -> &mut Self {

    // }

    pub fn emit( self ) {
        self.inner.emit();
    }
}


impl Deref for Message {
    type Target = Diagnostic;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Message {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}