use std::fmt::Display;

use redis::RedisError;
use tokio_tungstenite::tungstenite;

#[derive(Debug)]
pub enum Error {
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
            Error::WebSocketError(error) => f.write_str(
                format!("error with a player web socket: {}", error.to_string()).as_str(),
            ),
            Error::NothingToRead => f.write_str("read returned None"),
            Error::PlayerDeserializationError(json_err) => {
                f.write_str(format!("error while deserializing a player: {json_err}").as_str())
            }
            Error::ClientMessageDeserializeError(json_err) => f.write_str(
                format!("error while deserializing a client message: {json_err}").as_str(),
            ),
            Error::TickInfoSerializationError(json_err) => {
                f.write_str(format!("error while serializing a tick info: {json_err}").as_str())
            }
            Error::NormalClose => f.write_str(format!("client closed gracefuly").as_str()),
            Error::UnexpectedNonTextMessageError => {
                f.write_str(format!("received non text or close web socket message").as_str())
            }
            Error::RedisOpenError => f.write_str("error when opening Redis"),
            Error::RedisGetConnError(err) => {
                f.write_str(format!("Error while trying to connect to Redis: {}", err).as_str())
            }
            Error::NoPlayerForUuid => f.write_str("player not found for uuid"),
            Error::NoPlayerForNickname => f.write_str("player not found for nickname"),
            Error::KeysQueryError => f.write_str("error while querying 'keys' Redis command"),
            Error::DeletionQueryError => f.write_str("error while querying Redis key deletion"),
            Error::WriterFeedError => f.write_str("error while calling feed on a player's writer"),
            Error::UuidError => f.write_str("error while manipulating a uuid"),
            Error::ErrorFromMain => f.write_str("error catched by main"),
            Error::TracingError => f.write_str("error with tracing system initialization"),
            Error::UnexpectedNonLoginMessage => {
                f.write_str("non login message received during login phase")
            }
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
