pub trait Handler<Message>: Send + Sync {
    type Output;
}
