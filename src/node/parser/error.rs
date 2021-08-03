#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    #[inline]
    pub fn new<T: Into<String>>(message: T) -> Error {
        Error {
            message: message.into(),
        }
    }
}
