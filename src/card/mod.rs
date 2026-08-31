use crate::login::Transport;

#[derive(Debug, Clone)]
pub struct Card<T> {
    transport: T,
}

impl<T: Transport> Card<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }
}
