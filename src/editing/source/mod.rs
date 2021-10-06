use super::Id;

#[derive(Debug, Clone)]
pub enum BufferSource {
    /// The Buffer is in-memory only
    None,

    Help,
    Log,

    /// The Buffer was read from or has been written to a file
    /// on disk with the given absolute path
    LocalFile(String),

    /// The Buffer receives its content from a network source; such
    /// buffers MUST be read-only
    Connection(String),

    /// The Buffer is in-memory only, as None, but serves to provide
    /// input to the Connection in the buffer with the given Id
    ConnectionInputForBuffer(Id),
}

impl BufferSource {
    pub fn is_none(&self) -> bool {
        match self {
            &BufferSource::None => true,
            _ => false,
        }
    }
}
