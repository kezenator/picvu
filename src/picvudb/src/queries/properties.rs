use diesel::prelude::*;

use diesel;
use diesel::dsl::Select;

use crate::schema::*;

type AllColumns = (
    db_properties::name,
    db_properties::value,
);

const ALL_COLUMNS: AllColumns = (
    db_properties::name,
    db_properties::value,
);

type All = Select<db_properties::table, AllColumns>;

pub fn all() -> All
{
    db_properties::table.select(ALL_COLUMNS)
}