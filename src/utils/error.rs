use std::fmt::Display;

/// Log the error and map it to another error type.
pub fn log_map<E>(msg: impl Display, map_to: E) -> E {
    tracing::error!("{}", msg.to_string());

    map_to
}