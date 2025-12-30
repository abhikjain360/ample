use std::fmt;

use iced::Element;

pub(crate) struct Snackbar {
    ty: SnackbarType,
    #[expect(dead_code)]
    message: Element<'static, crate::Message>,
}

#[derive(Debug)]
pub(crate) enum SnackbarType {
    Info,
    Warning,
    Error,
}

impl Snackbar {
    #[expect(dead_code)]
    pub(crate) fn warning(message: impl Into<Element<'static, crate::Message>>) -> Self {
        Self {
            ty: SnackbarType::Warning,
            message: message.into(),
        }
    }

    #[expect(dead_code)]
    pub(crate) fn info(message: impl Into<Element<'static, crate::Message>>) -> Self {
        Self {
            ty: SnackbarType::Info,
            message: message.into(),
        }
    }

    pub(crate) fn error(message: impl Into<Element<'static, crate::Message>>) -> Self {
        Self {
            ty: SnackbarType::Error,
            message: message.into(),
        }
    }
}

impl fmt::Debug for Snackbar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Snackbar").field("ty", &self.ty).finish()
    }
}
