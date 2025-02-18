use std::ops::Deref;

use crate::context::Environment;

pub struct Context {
    env: Environment,
}

impl Context {
    pub fn env(&self) -> &Environment {
        &self.env
    }
}

pub struct SystemContext(pub Context);

impl Deref for SystemContext {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SystemContext {
    pub fn new(env: Environment) -> Self {
        SystemContext(Context { env })
    }
}
