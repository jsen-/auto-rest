use rocket::response::Responder;
use rocket::{response, Request, Response};
use rusqlite;

#[derive(Debug)]
pub enum Error {
    Db(rusqlite::Error),
    MissingValue(String),
    ExpectingObject,
}
pub type Result<T> = std::result::Result<T, Error>;
impl From<rusqlite::Error> for Error {
    fn from(val: rusqlite::Error) -> Self {
        Error::Db(val)
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        Response::build().ok()
    }
}
