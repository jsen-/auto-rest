#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

mod error;
mod prep;

use error::{Error, Result};

use prep::PrepareAndFetch;
use pulser::SerdeAdapter;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rocket::{http, response, response::Stream, routes, Request, State};
use rocket_codegen::{get, post};
use rocket_contrib::json::Json;
use rusqlite as sqlite;
use serde_json::{Map, Value};
use sqlite::types::ValueRef;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct ColumnDesc {
    name: String,
    ty: String,
    not_null: bool,
    has_dflt_value: bool,
    pk: bool,
}

type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;
type ConnPool = Pool<SqliteConnectionManager>;

fn table_columns(conn: &Connection, table_name: &str) -> sqlite::Result<Vec<ColumnDesc>> {
    conn.prepare("PRAGMA table_info(?)")?
        .query_map(&[table_name], |row| {
            let not_null: i32 = row.get_unwrap(3);
            let pk: i32 = row.get_unwrap(5);
            let dflt_value: ValueRef = row.get_raw(4);
            Ok(ColumnDesc {
                name: row.get_unwrap(1),
                ty: row.get_unwrap(2),
                not_null: if not_null == 1 { true } else { false },
                has_dflt_value: dflt_value != ValueRef::Null,
                pk: if pk == 1 { true } else { false },
            })
        })?
        .collect::<sqlite::Result<Vec<_>>>()
}

struct DatabaseRow(Map<String, Value>);
impl<'r> response::Responder<'r> for DatabaseRow {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        response::Response::build().ok()
    }
}

use std::fmt;

struct ColumnsOf<'a>(&'a Map<String, Value>);
impl<'a> fmt::Display for ColumnsOf<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut it = self.0.iter().peekable();
        while let Some((key, _)) = it.next() {
            write!(f, "{}", key)?;
            if let Some(_) = it.peek() {
                write!(f, ", ")?;
            }
        }
        Ok(())
    }
}

struct ValuesOf<'a>(&'a Map<String, Value>);
impl<'a> fmt::Display for ValuesOf<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut it = self.0.iter().peekable();
        while let Some((_, value)) = it.next() {
            write!(f, "{}", value)?;
            if let Some(_) = it.peek() {
                write!(f, ", ")?;
            }
        }
        Ok(())
    }
}

#[post("/api/v1/<table_name>", data = "<data>")]
fn table_add(
    pool: State<ConnPool>,
    table_name: String,
    mut data: Json<Value>,
) -> Result<DatabaseRow> {
    let db = pool.get().unwrap();
    drop(pool);

    let obj = data.as_object_mut().ok_or_else(|| Error::ExpectingObject)?;

    let pairs = table_columns(&db, &table_name)?
        .into_iter()
        .filter_map(|col| {
            let name = col.name;
            match obj.remove(&name) {
                Some(value) => Some(Ok((name, value))),
                None => {
                    if col.has_dflt_value {
                        None
                    } else {
                        Some(Err(Error::MissingValue(name)))
                    }
                }
            }
        })
        .collect::<Result<Map<String, Value>>>()?;

    format!("INSERT INTO {} ({}) VALUES ({})", table_name, ColumnsOf(&pairs), ValuesOf(&pairs));


    Ok(DatabaseRow(Map::new()))
}

#[get("/api/v1/<table_name>")]
fn table(
    pool: State<Pool<SqliteConnectionManager>>,
    table_name: String,
) -> Stream<impl io::Read + 'static> {
    use serde_json::{to_value, Number};
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
        .map(std::result::Result::unwrap);
    Stream::from(SerdeAdapter::new(it))
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
fn files(
    pool: State<Pool<SqliteConnectionManager>>,
    filename: PathBuf,
) -> Option<IncludedFile<io::Cursor<Vec<u8>>>> {
    let db = pool.get().unwrap();
    drop(pool);
    db.prepare_and_fetch(
        &format!(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='{}'",
            filename.display()
        ),
        |_| Ok(()),
    )
    .unwrap()
    .next()
    .map(|_| index())
    .or_else(|| file_response(filename))
}

fn file_response(filename: PathBuf) -> Option<IncludedFile<io::Cursor<Vec<u8>>>> {
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
pub fn index() -> IncludedFile<io::Cursor<Vec<u8>>> {
    file_response("index.html".into()).expect("index.html is missing")
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
        .mount("/", routes![table, table_add, files, index])
        .launch();
}
