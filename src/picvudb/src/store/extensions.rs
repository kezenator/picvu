use diesel::Expression;
use diesel::expression::AsExpression;
use diesel::sql_types::{BigInt, Integer, Nullable};


sql_function!(fn coalesce(x: Nullable<Integer>, y: Integer) -> BigInt);

diesel_infix_operator!(FtsMatch, " MATCH ");

pub trait FtsMatchExpressionMethods: Expression + Sized
{
    fn fts_match<T: AsExpression<Self::SqlType>>(self, other: T) -> FtsMatch<Self, T::Expression>
    {
        FtsMatch::new(self, other.as_expression())
    }
}

impl<T: Expression> FtsMatchExpressionMethods for T
{
}
