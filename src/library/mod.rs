use crate::login::Transport;

#[derive(Debug, Clone)]
pub struct Library<T> {
    transport: T,
}

impl<T: Transport> Library<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }
}
