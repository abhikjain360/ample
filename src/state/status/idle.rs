use std::sync::{Arc, RwLock};

#[derive(Debug)]
#[expect(dead_code)]
pub(crate) struct Idle {
    pub(crate) settings: crate::Settings,
    pub(crate) library: Arc<RwLock<crate::Library>>,
    // pub(crate) engine: crate::Engine,
}
