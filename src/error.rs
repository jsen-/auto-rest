use rocket::http::Status;
use rocket::response;
use rocket::Request;
use rusqlite;
use serde::Serialize;
use std::io;

#[derive(Serialize, Debug)]
pub enum Error {
    Db(String),
    TableNotFound(String),
    MissingValue(String),
    ExpectingObject(String),
}

struct W (Vec<u8>, usize);

impl io::Read for W {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let rest = &self.0[self.1..];
        rest.
    }
}

impl<'r> response::Responder<'r> for Error {
    fn respond_to(self, _request: &Request) -> response::Result<'r> {

        let buf = io::BufReader::new();
        response::Response::build()
            .status(Status::raw(400))
            .sized_body(buf)
            .ok()
    }
}

impl From<Error> for (Status, Error) {
    fn from(val: Error) -> (Status, Error) {
        (Status::raw(400), val)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
impl From<rusqlite::Error> for Error {
    fn from(val: rusqlite::Error) -> Self {
        Error::Db(format!("{:?}", val))
    }
}
