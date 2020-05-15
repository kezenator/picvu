use crate::err::Error;
use crate::store::ReadOps;
use crate::store::WriteOps;

pub trait StoreAccess
{
    fn read_transaction<T, E, F>(&self, f: F) -> Result<T, E>
        where F: FnOnce(& dyn ReadOps) -> Result<T, E>,
            E: From<Error>;

    fn write_transaction<T, E, F>(&self, f: F) -> Result<T, E>
            where F: FnOnce(& mut dyn WriteOps) -> Result<T, E>,
                E: From<Error>;
}
