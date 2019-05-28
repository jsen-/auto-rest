#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

mod error;
mod prep;

use error::{Error, Result};

use itertools::Itertools as _;
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

fn table_columns(conn: &Connection, table_name: &str) -> sqlite::Result<Option<Vec<ColumnDesc>>> {
    println!("2");
    let count = conn.query_row(
        "SELECT count(*)
           FROM sqlite_master
          WHERE type = 'table'
            AND name = ?",
        &[table_name],
        |row| row.get::<_, isize>(0),
    )?;
    if count == 0 {
        return Ok(None);
    }
    println!("3");
    let columns = conn
        .prepare(&format!("PRAGMA table_info({})", table_name))?
        .query_map(sqlite::NO_PARAMS, |row| {
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
        .collect::<sqlite::Result<Vec<_>>>()?;
    Ok(Some(columns))
}

struct DatabaseRow(Map<String, Value>);
impl<'r> response::Responder<'r> for DatabaseRow {
    fn respond_to(self, _request: &Request) -> response::Result<'r> {
        response::Response::build().ok()
    }
}

use std::fmt;

struct ColumnsOf<'a>(&'a Vec<&'a String>);
impl<'a> fmt::Display for ColumnsOf<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut it = self.0.iter().peekable();
        while let Some(key) = it.next() {
            write!(f, "{}", key)?;
            if let Some(_) = it.peek() {
                write!(f, ", ")?;
            }
        }
        Ok(())
    }
}

union Num {
    f: f64,
    i: i64,
}

struct ToSqlIter<'a, T> {
    columns: T,
    values: &'a Map<String, Value>,
    num: Num,
}
impl<'a, T> ToSqlIter<'a, T> {
    pub fn new<I>(columns: I, values: &'a Map<String, Value>) -> Self
    where
        I: IntoIterator<Item = &'a String, IntoIter = T>,
        T: Iterator<Item = &'a String>,
    {
        Self {
            columns: columns.into_iter(),
            values: values,
            num: Num { i: 0 },
        }
    }
}

impl<'a, T> Iterator for ToSqlIter<'a, T>
where
    T: Iterator<Item = &'a String>,
{
    type Item = &'a dyn sqlite::ToSql;
    fn next(&mut self) -> Option<Self::Item> {
        self.columns
            .next()
            .and_then(|column_name| -> Option<&dyn sqlite::ToSql> {
                match &self.values[column_name] {
                    Value::Bool(b) => Some(b),
                    Value::Null => Some(&sqlite::types::Null),
                    Value::String(s) => Some(s),
                    Value::Number(n) => {
                        if n.is_f64() {
                            self.num = Num {
                                f: n.as_f64().unwrap(),
                            };
                            let x: &f64 = unsafe { std::mem::transmute(&self.num.f) };
                            Some(x)
                        } else if n.is_u64() {
                            self.num = Num {
                                i: n.as_u64().unwrap() as i64,
                            };
                            let x: &i64 = unsafe { std::mem::transmute(&self.num.i) };
                            Some(x)
                        } else if n.is_i64() {
                            self.num = Num {
                                i: n.as_i64().unwrap(),
                            };
                            let x: &i64 = unsafe { std::mem::transmute(&self.num.i) };
                            Some(x)
                        } else {
                            panic!("Number in json_value is not f64, neither u64, nor i64");
                        }
                    }
                    Value::Object(_) | Value::Array(_) => {
                        panic!("structured types cannot be inserted")
                    }
                }
            })
    }
}

fn sql_to_json(row: &sqlite::Row) -> sqlite::Result<Value> {
    use serde_json::{to_value, Number};
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
    Ok(Value::Object(whole_row))
}

#[post("/api/v1/<table_name>", data = "<data>")]
fn table_add(pool: State<ConnPool>, table_name: String, data: Json<Value>) -> Result<Json<Value>> {
    let db = pool.get().unwrap();
    drop(pool);

    let values = data
        .as_object()
        .ok_or_else(|| Error::ExpectingObject)?;

    let cols = table_columns(&db, &table_name).map_err(Error::from)?;
    let (cols, columns) = match cols {
        None => return Err(Error::TableNotFound(table_name).into()),
        Some(ref columns) => (
            columns,
            columns
                .iter()
                .filter_map(|col| {
                    let name = &col.name;
                    if values.contains_key(name) {
                        Some(Ok(&col.name))
                    } else {
                        if col.has_dflt_value || col.pk {
                            None
                        } else {
                            Some(Err(Error::MissingValue(name.to_string())))
                        }
                    }
                })
                .collect::<Result<Vec<&String>>>()?,
        ),
    };

    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table_name,
        ColumnsOf(&columns),
        std::iter::repeat("?")
            .take(columns.len())
            .intersperse(", ")
            .collect::<String>()
    );

    let id_column = cols
        .iter()
        .find(|&col| col.pk)
        .map(|col| &col.name)
        .unwrap();
    let id = db.prepare(&sql).map_err(Error::from)?.insert(ToSqlIter::new(columns, values)).map_err(Error::from)?;
    let obj = db
        .prepare(&format!(
            "SELECT * FROM {} WHERE {} = ?",
            table_name, id_column
        ))
        .map_err(Error::from)?
        .query_row(&[id], sql_to_json)
        .map_err(Error::from)?;
    Ok(Json(obj))
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
