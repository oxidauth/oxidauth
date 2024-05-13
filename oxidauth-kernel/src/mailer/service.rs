use super::{Message, SendError, Sender};

#[derive(Clone)]
pub struct SenderService<T>
where
    T: Sender,
{
    sender: T,
}

impl<T> SenderService<T>
where
    T: Sender,
{
    pub fn new(sender: T) -> SenderService<T> {
        Self { sender }
    }
}

impl<T> SenderService<T>
where
    T: Sender,
{
    pub async fn send(&self, msg: &Message) -> Result<T::Value, SendError> {
        self.sender.send(msg).await
    }
}

pub trait IntoSenderService<T>
where
    T: Sender,
{
    fn into_sender_service(self) -> SenderService<T>;
}
