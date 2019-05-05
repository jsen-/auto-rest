#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

use rocket::{http, response, response::Stream, routes, Request};
use rocket_codegen::get;
use rocket_contrib::database;
use rocket_contrib::databases::rusqlite as sqlite;
use rocket_contrib::json::Json;
use serde_json::Value as JsonValue;
use sqlite::types::ValueRef;
use std::io;
use std::path::{Path, PathBuf};

fn from_sql_to_json(v: ValueRef) -> JsonValue {
    match v {
        ValueRef::Null => JsonValue::Null,
        ValueRef::Integer(i) => JsonValue::Number(i.into()),
        ValueRef::Real(r) => JsonValue::Number(serde_json::Number::from_f64(r).unwrap()),
        ValueRef::Text(t) => JsonValue::String(t.into()),
        ValueRef::Blob(b) => JsonValue::Array(
            b.into_iter()
                .map(|&i| JsonValue::Number(i.into()))
                .collect::<Vec<_>>(),
        ),
    }
}

#[database("local")]
struct LocalDb(rocket_contrib::databases::rusqlite::Connection);

impl LocalDb {
    pub fn query_table_all(&self, table_name: &str) -> sqlite::Result<JsonValue> {
        let mut stmt = self.0.prepare(&format!("SELECT * FROM {}", table_name))?;

        let vec = stmt
            .query_map(sqlite::NO_PARAMS, |row| -> sqlite::Result<_> {
                let mut row_values = serde_json::Map::new();
                for (index, column) in row.columns().into_iter().enumerate() {
                    row_values.insert(
                        column.name().to_string(),
                        from_sql_to_json(row.get_raw(index)),
                    );
                }
                Ok(JsonValue::Object(row_values))
            })?
            .collect::<sqlite::Result<Vec<JsonValue>>>()?;
        Ok(JsonValue::Array(vec))
    }
}

#[get("/api/v1/<table_name>")]
fn table(local_db: LocalDb, table_name: String) -> sqlite::Result<Json<JsonValue>> {
    Ok(Json(local_db.query_table_all(&table_name)?))
}

#[derive(rust_embed::RustEmbed)]
#[folder = "target/frontend"]
struct FrontendFiles;
pub struct IncludedFile<T: io::Read> {
    relative_path: String,
    contents: Stream<T>,
}

impl<T: io::Read> IncludedFile<T> {
    pub fn new(relative_path: String, contents: T) -> Self {
        Self {
            relative_path,
            contents: Stream::from(contents),
        }
    }
}

impl<'r, T: io::Read + 'r> response::Responder<'r> for IncludedFile<T> {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        let mut response = self.contents.respond_to(req)?;
        if let Some(ext) = Path::new(&self.relative_path).extension() {
            if let Some(ct) = http::ContentType::from_extension(&ext.to_string_lossy()) {
                response.set_header(ct);
            }
        }

        Ok(response)
    }
}

#[get("/<filename..>", rank = 999)]
fn files(filename: PathBuf) -> Option<IncludedFile<io::Cursor<Vec<u8>>>> {
    match filename.into_os_string().into_string() {
        Ok(filename) => FrontendFiles::get(&filename).map(|contents| {
            let vec = contents.as_ref().to_owned();
            IncludedFile::new(filename, io::Cursor::new(vec))
        }),
        _ => None,
    }
}

#[get("/")]
#[allow(clippy::needless_pass_by_value)]
pub fn slash() -> IncludedFile<io::Cursor<Vec<u8>>> {
    files("index.html".into()).expect("index.html is missing")
}

fn main() {
    rocket::ignite()
        .attach(LocalDb::fairing())
        .mount("/", routes![table, files, slash])
        .mount("/products", routes![slash])
        .launch();
}
