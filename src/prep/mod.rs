mod arc;
mod it;
mod pooled_connection;
mod rc;

use crate::sqlite::{Result, Row};

pub trait PrepareAndFetch {
    fn prepare_and_fetch<R, F>(
        self,
        sql: &str,
        mapper: F,
    ) -> Result<Box<Iterator<Item = Result<R>> + 'static>>
    where
        F: FnMut(&Row) -> Result<R>,
        F: 'static,
        R: 'static;
}
