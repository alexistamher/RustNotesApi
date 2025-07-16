use std::error::Error;


pub trait MapErrorToString<T> {
    fn map_err_as_str(self) -> Result<T, String>;
}

impl<T, E: Error> MapErrorToString<T> for Result<T, E> {
    fn map_err_as_str(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}
