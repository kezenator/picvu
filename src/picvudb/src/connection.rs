use diesel::{Connection, RunQueryDsl, SqliteConnection};
use diesel_migrations::RunMigrationsError;
use snafu::{ResultExt, Snafu};

use crate::models;
use crate::schema;

embed_migrations!("./migrations");

#[derive(Debug, Snafu)]
pub enum DbConnectionError{
    #[snafu(display("Unable to open database {:?}: {}", path, source))]
    LowerDbConnectionError { source: diesel::ConnectionError, path: String },
    #[snafu(display("Database contains invalid properties: {}", source))]
    LowerDbPropertiesError { source: diesel::result::Error },
    #[snafu(display("Unable to apply initial database setup: {}", source))]
    LowerDbMigrationError { source: RunMigrationsError },
    #[snafu(display("Unsupported schema version: {}", version))]
    UnsupportedVersionError { version: String },
}

pub type DbConnectionResult = Result<DbConnection, DbConnectionError>;

pub struct DbConnection
{
    pub connection: SqliteConnection,
}

impl DbConnection
{
    pub fn new(path: &str) -> DbConnectionResult
    {
        let db_connection = SqliteConnection::establish(path)
            .context(LowerDbConnectionError{path: path.to_owned() })?;

        if schema::db_properties::table
            .load::<models::DbProperty>(&db_connection)
            .context(LowerDbPropertiesError{})
            .is_err()
        {
            embedded_migrations::run(&db_connection)
                .context(LowerDbMigrationError{})?;

            let name = "version".to_owned();
            let value = "2020-06-14".to_owned();
            
            diesel::insert_into(schema::db_properties::table)
                .values(&models::DbProperty{name, value})
                .execute(&db_connection)
                .context(LowerDbPropertiesError{})?;
        }
        
        let mut properties = schema::db_properties::table
            .load::<models::DbProperty>(&db_connection)
            .context(LowerDbPropertiesError{})?;

        let versions = properties.drain(..)
            .filter(|prop| prop.name == "version")
            .map(|prop| prop.value)
            .collect::<Vec<String>>();

        let mut version = String::new();

        if versions.len() == 1
        {
            version = versions[0].clone();
        }

        ensure!(version == "2020-06-14", UnsupportedVersionError{ version });

        Ok(Self{ connection: db_connection })
    }
}