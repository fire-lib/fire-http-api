//! Stream protocol based on websocket
//! ```ignore
//! // the client want's to start a sender stream
//! Client > Kind: SenderRequest Action: "MyAction" data: request
//! // the server acknowledge the request / or sends a SenderClose
//! Server > Kind: SenderRequest Action: "MyAction" data: null
//! // the client now start the send messages
//! Client > Kind: SenderMessage Action: "MyAction" data: message
//! // either the client or the server can send a SenderClose
//! // which will indicate that the stream should be terminated
//! Server > Kind: SenderClose Action: "MyAction" data: null|error
//! ```


mod macros;
pub mod message;
pub mod server;
mod stream;
pub mod streamer;
pub mod error;
mod poll_fn;

// pub use message::MessageData;
pub use server::StreamServer;
pub use stream::{StreamKind, Stream};
pub use error::StreamError;
pub use streamer::Streamer;