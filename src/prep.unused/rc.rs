use super::PrepareAndFetch;
use crate::sqlite::{Connection, MappedRows, Result, Row, Statement};
use std::rc::Rc;

pub struct ItPrepRc<R, F>
where
    F: FnMut(&Row) -> R,
{
    rs: MappedRows<'static, F>,
    // stmt is never accessed, but we need it with a stable address
    #[allow(dead_code)]
    stmt: Box<Statement<'static>>,
    #[allow(dead_code)]
    conn: Rc<Connection>,
}

impl<R, F> Iterator for ItPrepRc<R, F>
where
    F: FnMut(&Row) -> R,
{
    type Item = Result<R>;
    fn next(&mut self) -> Option<Self::Item> {
        self.rs.next()
    }
}

impl<'conn> PrepareAndFetch<'conn> for Rc<Connection> {
    fn prepare_and_fetch<R, F>(
        self,
        sql: &str,
        mapper: F,
    ) -> Result<Box<Iterator<Item = Result<R>> + 'conn>>
    where
        F: FnMut(&Row) -> R,
        F: 'conn,
        R: 'conn,
    {
        println!("rc");
        unsafe {
            let mut stmt = Box::new(std::mem::transmute(self.prepare(sql)?));
            let stmt_ptr = &mut *stmt as *mut Statement;
            let rs = { &mut *stmt_ptr }.query_map(&[], mapper)?;
            Ok(Box::new(ItPrepRc {
                rs,
                stmt,
                conn: self,
            }))
        }
    }
}
