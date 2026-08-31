use crate::login::Transport;

#[derive(Debug, Clone)]
pub struct Jwxt<T> {
    transport: T,
}

impl<T: Transport> Jwxt<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }
}
