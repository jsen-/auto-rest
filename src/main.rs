#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

mod prep;
use prep::PrepareAndFetch;

use pulser::SerdeAdapter;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rocket::{http, response, response::Stream, routes, Request, State};
use rocket_codegen::get;
use rusqlite as sqlite;
use sqlite::types::ValueRef;
use std::io;
use std::path::{Path, PathBuf};

#[get("/api/v1/<table_name>")]
fn table(
    pool: State<Pool<SqliteConnectionManager>>,
    table_name: String,
) -> Stream<impl io::Read + 'static> {
    use serde_json::{to_value, Map, Number, Value};

    let db = pool.get().unwrap();
    drop(pool);
    let it = db
        .prepare_and_fetch(&format!("SELECT * FROM {}", table_name), |row| {
            let whole_row = row
                .columns()
                .into_iter()
                .enumerate()
                .map(|(index, column)| {
                    let value = match row.get_raw(index) {
                        ValueRef::Null => Value::Null,
                        ValueRef::Text(s) => Value::String(s.to_string()),
                        ValueRef::Integer(i) => Value::Number(Number::from(i)),
                        ValueRef::Real(f) => to_value(f).unwrap(),
                        ValueRef::Blob(_bytes) => Value::String("<blob>".into()),
                    };
                    (column.name().to_string(), value)
                })
                .collect::<Map<_, _>>();
            Ok(whole_row)
        })
        .unwrap()
        .map(Result::unwrap);
    Stream::from(SerdeAdapter::new(it))
}

#[get("/test")]
fn test(pool: State<Pool<SqliteConnectionManager>>) -> Stream<impl io::Read + 'static> {
    use serde_json::{to_value, Map, Number, Value};

    let db = pool.get().unwrap();
    drop(pool);
    let it = db
        .prepare_and_fetch("select * from om_server join om_environment join om_admin", |row| {
            let whole_row = row
                .columns()
                .into_iter()
                .enumerate()
                .map(|(index, column)| {
                    let value = match row.get_raw(index) {
                        ValueRef::Null => Value::Null,
                        ValueRef::Text(s) => Value::String(s.to_string()),
                        ValueRef::Integer(i) => Value::Number(Number::from(i)),
                        ValueRef::Real(f) => to_value(f).unwrap(),
                        ValueRef::Blob(_bytes) => Value::String("<blob>".into()),
                    };
                    (column.name().to_string(), value)
                })
                .collect::<Map<_, _>>();
            Ok(whole_row)
        })
        .unwrap()
        .map(Result::unwrap);
    Stream::from(SerdeAdapter::new(it))
}

#[get("/test2")]
fn test2(pool: State<Pool<SqliteConnectionManager>>) -> Stream<impl io::Read + 'static> {
    use serde_json::{to_value, Map, Number, Value};

    let db = pool.get().unwrap();
    drop(pool);
    let mut stmt = db.prepare("select * from om_server join om_environment join om_admin").unwrap();
    let vec = stmt.query_map(sqlite::NO_PARAMS, |row| {
        let whole_row = row
                .columns()
                .into_iter()
                .enumerate()
                .map(|(index, column)| {
                    let value = match row.get_raw(index) {
                        ValueRef::Null => Value::Null,
                        ValueRef::Text(s) => Value::String(s.to_string()),
                        ValueRef::Integer(i) => Value::Number(Number::from(i)),
                        ValueRef::Real(f) => to_value(f).unwrap(),
                        ValueRef::Blob(_bytes) => Value::String("<blob>".into()),
                    };
                    (column.name().to_string(), value)
                })
                .collect::<Map<_, _>>();
            Ok(whole_row)
    }).unwrap()
    .collect::<Result<Vec<_>, _>>().unwrap();
    Stream::from(io::Cursor::new(serde_json::to_vec(&vec).unwrap()))
}

#[derive(rust_embed::RustEmbed)]
#[folder = "target/frontend/"]
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

#[derive(serde::Serialize, Debug)]
struct DbRow {
    id: isize,
    name: String,
    age: u8,
}

fn main() {
    // let vec = vec![DbRow {id: 1, name: "Jozo".into(), age: 72}, DbRow {id: 2, name: "Fero".into(), age: 58}];
    // let mut adapter = SerdeAdapter::new(vec);
    // println!("{:#?}", adapter);
    // io::copy(&mut adapter, &mut io::stdout()).unwrap();

    let conn_mgr = SqliteConnectionManager::file("db.sqlite").with_init(|conn| {
        conn.pragma_update_and_check(None, "journal_mode", &"wal".to_string(), |row| {
            let wal: String = row.get(0)?;
            if wal == "wal" {
                Ok(())
            } else {
                Err(sqlite::Error::SqliteSingleThreadedMode)
            }
        })
    });
    let pool = Pool::new(conn_mgr).unwrap();

    rocket::ignite()
        .manage(pool)
        .mount("/", routes![table, files, slash, test, test2])
        .mount("/products", routes![slash])
        .launch();
}
