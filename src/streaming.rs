use std::pin::Pin;
use std::task::{Context, Poll};
use futures_util::Stream;
use tokio::sync::mpsc;

/// A token stream that implements proper async streaming
pub struct TokenStream {
    receiver: mpsc::UnboundedReceiver<TokenEvent>,
}

#[derive(Debug, Clone)]
pub enum TokenEvent {
    Token(String),
    Done,
    Error(String),
}

impl TokenStream {
    pub fn new() -> (TokenSender, Self) {
        let (tx, rx) = mpsc::unbounded_channel();
        (TokenSender { sender: tx }, Self { receiver: rx })
    }
}

impl Stream for TokenStream {
    type Item = TokenEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

/// Thread-safe token sender
#[derive(Clone)]
pub struct TokenSender {
    sender: mpsc::UnboundedSender<TokenEvent>,
}

impl TokenSender {
    pub fn send_token(&self, token: String) -> Result<(), mpsc::error::SendError<TokenEvent>> {
        self.sender.send(TokenEvent::Token(token))
    }
    
    pub fn send_done(&self) -> Result<(), mpsc::error::SendError<TokenEvent>> {
        self.sender.send(TokenEvent::Done)
    }
    
    pub fn send_error(&self, error: String) -> Result<(), mpsc::error::SendError<TokenEvent>> {
        self.sender.send(TokenEvent::Error(error))
    }
}

/// Async-compatible callback wrapper
pub struct AsyncTokenCallback {
    sender: TokenSender,
}

impl AsyncTokenCallback {
    pub fn new(sender: TokenSender) -> Self {
        Self { sender }
    }
    
    pub fn into_callback(self) -> Box<dyn FnMut(String) + Send> {
        let sender = self.sender;
        Box::new(move |token: String| {
            let _ = sender.send_token(token);
        })
    }
}
