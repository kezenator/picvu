use actix::prelude::*;
use picvudb::*;

pub struct DbAddr(Addr<DbExecutor>);

impl DbAddr
{
    pub fn new(addr: Addr<DbExecutor>) -> Self
    {
        DbAddr(addr)
    }

    pub fn send<M: picvudb::ApiMessage>(&self, msg: M) -> Request<DbExecutor, WrappedMsg<M>>
    {
        self.0.send(WrappedMsg{ wrapped: msg })
    }
}

pub struct DbExecutor(picvudb::Store);

impl DbExecutor
{
    pub fn new(store: picvudb::Store) -> Self
    {
        DbExecutor(store)
    }
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

pub struct WrappedMsg<M>
    where M: picvudb::ApiMessage
{
    wrapped: M,
}

impl<M> actix::Message for WrappedMsg<M>
    where M: picvudb::ApiMessage
{
    type Result = Result<M::Response, M::Error>;
}

impl<M> Handler<WrappedMsg<M>> for DbExecutor
    where M: picvudb::ApiMessage
{
    type Result = Result<M::Response, M::Error>;

    fn handle(&mut self, msg: WrappedMsg<M>, _: &mut Self::Context) -> Self::Result
    {
        self.0.write_transaction(|ops|
        {
            msg.wrapped.execute(ops)
        })
    }
}
