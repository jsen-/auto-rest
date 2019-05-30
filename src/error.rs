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
    DbPool(String),
    ConstraintViolation(Option<String>),
    TableNotFound(String),
    NoPrimaryKey(String),
    CompositePrimaryKey(String),
    MissingValue(String),
    ExpectingObject,
}

#[derive(Serialize)]
struct JsonErr {
    error: String,
    field: Option<String>,
}
impl From<String> for JsonErr {
    fn from(error: String) -> Self {
        JsonErr { error, field: None }
    }
}

impl<'r> response::Responder<'r> for Error {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        let error: JsonErr = match self {
            Error::Db(inner_error) => format!("{}", inner_error).into(),
            Error::ConstraintViolation(field) => JsonErr {
                error: "Constraint violation".to_string(),
                field,
            },
            Error::TableNotFound(table_name) => format!(r#"Table "{}" not found"#, table_name).into(),
            Error::MissingValue(field_name) => {
                format!(r#"Value for field "{}" is missing"#, field_name).into()
            }
            Error::NoPrimaryKey(table_name) => format!(
                r#"Table "{}" does not have a primary key column"#,
                table_name
            ).into(),
            Error::CompositePrimaryKey(table_name) => format!(
                r#"Table "{}" has composite primary key, which is not supported"#,
                table_name
            ).into(),
            Error::DbPool(msg) => msg.into(),
            Error::ExpectingObject => format!("Input format should be a Json object").into(),
        };
        let res = Json(serde_json::to_string(&error).unwrap())
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
impl From<r2d2::Error> for Error {
    fn from(val: r2d2::Error) -> Error {
        Error::DbPool(format!("{}", val))
    }
}

fn get_fieldname_from_constraint_violation_message(msg: String) -> Option<String> {
    const PREFIX: &str = "UNIQUE constraint failed: ";
    if msg.starts_with(PREFIX) {
        msg[PREFIX.len()..]
            .split('.')
            .skip(1)
            .map(String::from)
            .next()
    } else {
        None
    }
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
