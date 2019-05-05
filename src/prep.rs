mod arc;
mod rc;
mod it;

use crate::sqlite::{Result, Row};

pub trait PrepareAndFetch<'conn> {
    fn prepare_and_fetch<R, F>(
        self,
        sql: &str,
        mapper: F,
    ) -> Result<Box<Iterator<Item = Result<R>> + 'conn>>
    where
        F: FnMut(&Row) -> R,
        F: 'conn,
        R: 'conn;
}
