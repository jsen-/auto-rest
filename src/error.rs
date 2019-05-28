use rocket::http::Status;
use rocket::response;
use rocket::response::content::Json;
use rocket::Request;
use rusqlite;
use rusqlite::ffi::ErrorCode as SqliteError;
use serde::Serialize;

#[derive(Debug)]
pub enum Error {
    Db(rusqlite::Error),
    ConstraintViolation(Option<String>),
    TableNotFound(String),
    MissingValue(String),
    ExpectingObject,
}

#[derive(Serialize)]
struct JsonErr {
    error: String,
}

impl<'r> response::Responder<'r> for Error {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        let error = match self {
            Error::Db(inner_error) => format!("{}", inner_error),
            Error::ConstraintViolation(Some(field_name)) => {
                format!(r#"Constraint violation on field "{}""#, field_name)
            }
            Error::ConstraintViolation(None) => "Constraint violation".into(),
            Error::TableNotFound(table_name) => format!(r#"Table "{}" not found"#, table_name),
            Error::MissingValue(field_name) => {
                format!(r#"Value for field "{}" is missing"#, field_name)
            }
            Error::ExpectingObject => format!("Input format should be a Json object"),
        };
        let res = Json(serde_json::to_string(&JsonErr { error }).unwrap())
            .respond_to(req)
            .unwrap();
        response::Response::build()
            .merge(res)
            .status(Status::raw(400))
            .ok()
    }
}

impl From<Error> for (Status, Error) {
    fn from(val: Error) -> (Status, Error) {
        (Status::raw(400), val)
    }
}

fn get_fieldname_from_constraint_violation_message(msg: String) -> Option<String> {
    None
}

pub type Result<T> = std::result::Result<T, Error>;
impl From<rusqlite::Error> for Error {
    fn from(mut val: rusqlite::Error) -> Self {
        match val {
            rusqlite::Error::SqliteFailure(ref error, ref mut maybe_msg) => match error.code {
                SqliteError::ConstraintViolation => Error::ConstraintViolation(
                    maybe_msg
                        .take()
                        .and_then(get_fieldname_from_constraint_violation_message),
                ),
                _ => Error::Db(val),
            },
            _ => Error::Db(val),
        }
    }
}
