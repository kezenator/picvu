use std::fmt::Debug;
use crate::store::WriteOps;

pub mod data;
pub mod msgs;

pub trait ApiMessage: Debug + Send + 'static
{
    type Response: Debug + Send + 'static;
    type Error: From<crate::Error> + Debug + Send + 'static;

    fn execute(&self, ops: &dyn WriteOps) -> Result<Self::Response, Self::Error>;
}