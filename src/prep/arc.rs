use super::PrepareAndFetch;
use crate::sqlite::{Connection, MappedRows, Result, Row, Statement, NO_PARAMS};
use std::sync::Arc;

pub struct ItPrepArc<R, F>
where
    F: FnMut(&Row) -> Result<R>,
{
    rs: MappedRows<'static, F>,
    // stmt is never accessed, but we need it with a stable address
    #[allow(dead_code)]
    stmt: Box<Statement<'static>>,
    #[allow(dead_code)]
    conn: Arc<Connection>,
}

impl<R, F> Iterator for ItPrepArc<R, F>
where
    F: FnMut(&Row) -> Result<R>,
{
    type Item = Result<R>;
    fn next(&mut self) -> Option<Self::Item> {
        self.rs.next()
    }
}

impl PrepareAndFetch for Arc<Connection> {
    fn prepare_and_fetch<R, F>(
        self,
        sql: &str,
        mapper: F,
    ) -> Result<Box<Iterator<Item = Result<R>> + 'static>>
    where
        F: FnMut(&Row) -> Result<R>,
        F: 'static,
        R: 'static,
    {
        unsafe {
            let mut stmt = Box::new(std::mem::transmute(self.prepare(sql)?));
            let stmt_ptr = &mut *stmt as *mut Statement;
            let rs = { &mut *stmt_ptr }.query_map(NO_PARAMS, mapper)?;
            Ok(Box::new(ItPrepArc {
                rs,
                stmt,
                conn: self,
            }))
        }
    }
}
