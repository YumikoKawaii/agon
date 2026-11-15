use std::fmt;

#[derive(Debug)]
pub enum AgonError {
    Texture { path: String, source: macroquad::Error },
}

impl fmt::Display for AgonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AgonError::Texture { path, source } => write!(f, "can't load texture '{path}': {source}"),
        }
    }
}

impl std::error::Error for AgonError {}

pub type AResult<T = ()> = Result<T, AgonError>;
