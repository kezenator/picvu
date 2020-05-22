use crate::connection::*;

use crate::err::*;
use crate::store::access::*;
use crate::store::ops::*;
use crate::store::trans::*;

use diesel::Connection;

pub struct Store
{
    db_connection: DbConnection,
}

impl Store
{
    pub fn new(path: &str) -> Result<Store, DbConnectionError>
    {
        let db_connection = DbConnection::new(path)?;

        Ok(Store { db_connection })
    }
}

impl StoreAccess for Store
{
    fn read_transaction<T, E, F>(&self, f: F) -> Result<T, E>
        where F: FnOnce(& dyn ReadOps) -> Result<T, E>,
            E: From<Error>
    {
        let mut opt_result: Option<Result<T, E>> = None;

        self.db_connection.connection.transaction(||
            {
                let trans = Transaction{ connection: &self.db_connection.connection };

                opt_result = Some(f(&trans));

                Ok(())
            })?;

        assert!(opt_result.is_some());
        opt_result.unwrap()
    }

    fn write_transaction<T, E, F>(&self, f: F) -> Result<T, E>
            where F: FnOnce(& mut dyn WriteOps) -> Result<T, E>,
                E: From<Error>
    {
        let mut opt_f_result: Option<Result<T, E>> = None;

        let db_result = self.db_connection.connection.transaction(||
            {
                let mut trans = Transaction{ connection: &self.db_connection.connection };

                match f(&mut trans)
                {
                    Ok(val) =>
                    {
                        opt_f_result = Some(Ok(val));
                        Ok(())
                    },
                    Err(e) =>
                    {
                        opt_f_result = Some(Err(e));
                        Err(diesel::result::Error::RollbackTransaction)
                    },
                }
            });

        if let Some(Err(f_err)) = opt_f_result
        {
            Err(f_err)
        }
        else if db_result.is_err()
        {
            let conv_db_err: Error = db_result.unwrap_err().into();
            Err(conv_db_err.into())
        }
        else if let Some(Ok(f_val)) = opt_f_result
        {
            Ok(f_val)
        }
        else
        {
            assert!(false);
            let last_chance_err: Error = diesel::result::Error::RollbackTransaction.into();
            Err(last_chance_err.into())
        }
    }
}