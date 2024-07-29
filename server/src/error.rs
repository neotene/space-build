use std::fmt::Display;

use redis::RedisError;
use tokio_tungstenite::tungstenite;

#[derive(Debug)]
pub enum Error {
    // TestError(String),
    WebSocketError(tungstenite::Error),

    NothingToRead,

    PlayerDeserializationError(serde_json::Error),
    ClientMessageDeserializeError(serde_json::Error),
    TickInfoSerializationError(serde_json::Error),

    NormalClose,
    UnexpectedNonTextMessageError,

    RedisOpenError,
    RedisGetConnError(RedisError),

    NoPlayerForUuid,
    NoPlayerForNickname,

    KeysQueryError,
    DeletionQueryError,

    WriterFeedError,

    UuidError,

    ErrorFromMain,

    TracingError,

    UnexpectedNonLoginMessage,

    SaveSystemsSetError(RedisError),
    SaveSystemsSerializationError(serde_json::Error),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::WebSocketError(error) => {
                f.write_str(format!("Web socket error: {}", error.to_string()).as_str())
            }

            Error::NothingToRead => f.write_str("NothingToRead"),

            Error::PlayerDeserializationError(json_err) => {
                f.write_str(format!("PlayerDeserializationError: {json_err}").as_str())
            }
            Error::ClientMessageDeserializeError(json_err) => {
                f.write_str(format!("ClientMessageDeserializeError: {json_err}").as_str())
            }
            Error::TickInfoSerializationError(json_err) => {
                f.write_str(format!("TickInfoSerializationError: {json_err}").as_str())
            }

            Error::NormalClose => f.write_str(format!("Client closed gracefuly").as_str()),
            Error::UnexpectedNonTextMessageError => {
                f.write_str(format!("Received non text or close ws message").as_str())
            }

            Error::RedisOpenError => f.write_str("RedisOpenError"),
            Error::RedisGetConnError(err) => {
                f.write_str(format!("Error while trying to connect to Redis: {}", err).as_str())
            }

            Error::NoPlayerForUuid => f.write_str("NoPlayerForUuid"),
            Error::NoPlayerForNickname => f.write_str("NoPlayerForNickname"),

            Error::KeysQueryError => f.write_str("KeysQueryError"),
            Error::DeletionQueryError => f.write_str("DeletionQueryError"),

            Error::WriterFeedError => f.write_str("WriterFeedError"),

            Error::UuidError => f.write_str("UuidError"),

            Error::ErrorFromMain => f.write_str("ErrorFromMain"),

            Error::TracingError => f.write_str("Error with tracing system initialization"),

            Error::UnexpectedNonLoginMessage => f.write_str("UnexpectedNonLoginMessage"),

            Error::SaveSystemsSetError(redis_err) => f.write_str(
                format!("error while trying to save systems in Redis: {redis_err}").as_str(),
            ),

            Error::SaveSystemsSerializationError(redis_err) => f.write_str(
                format!("error while trying to serialize systems before saving them: {redis_err}")
                    .as_str(),
            ),
        }
    }
}
