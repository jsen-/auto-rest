use super::PrepareAndFetch;
use crate::sqlite::{Connection, MappedRows, Result, Row, Statement};

pub struct ItPrep<'conn, R, F>
where
    F: FnMut(&Row) -> R,
{
    rs: MappedRows<'conn, F>,
    // stmt is never accessed, but we need it with a stable address
    #[allow(dead_code)]
    stmt: Box<Statement<'conn>>,
}

impl<'conn, R, F> Iterator for ItPrep<'conn, R, F>
where
    F: FnMut(&Row) -> R,
{
    type Item = Result<R>;
    fn next(&mut self) -> Option<Self::Item> {
        self.rs.next()
    }
}

impl<'conn> PrepareAndFetch<'conn> for &'conn Connection {
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
        println!("ref");
        unsafe {
            let mut stmt = Box::new(std::mem::transmute(self.prepare(sql)?));
            let stmt_ptr = &mut *stmt as *mut Statement;
            let rs = { &mut *stmt_ptr }.query_map(&[], mapper)?;
            Ok(Box::new(ItPrep { rs, stmt }))
        }
    }
}
