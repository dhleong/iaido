pub enum BufferSource {
    /// The Buffer is in-memory only
    None,

    /// The Buffer was read from or has been written to a file
    /// on disk with the given absolute path
    LocalFile(String),

    /// The Buffer receives its content from a network source; such
    /// buffers MUST be read-only
    Connection(String),
}

impl BufferSource {
    pub fn is_none(&self) -> bool {
        match self {
            &BufferSource::None => true,
            _ => false,
        }
    }
}
