#[cfg(all(feature = "sea-orm-internal", feature = "sqlx-dep"))]
pub use sqlx::error::Error as SqlxError;

#[cfg(all(feature = "sea-orm-internal", feature = "sqlx-mysql"))]
pub use sqlx::mysql::MySqlDatabaseError as SqlxMySqlError;

#[cfg(all(feature = "sea-orm-internal", feature = "sqlx-postgres"))]
pub use sqlx::postgres::PgDatabaseError as SqlxPostgresError;

#[cfg(all(feature = "sea-orm-internal", feature = "sqlx-sqlite"))]
pub use sqlx::sqlite::SqliteError as SqlxSqliteError;

use thiserror::Error;

/// An error from unsuccessful database operations
#[derive(Error, Debug)]
pub enum DbErr {
    /// This error can happen when the connection pool is fully-utilized
    #[error("Failed to acquire connection from pool")]
    ConnectionAcquire,
    /// Runtime type conversion error
    #[error("Error converting `{from}` into `{into}`: {source}")]
    TryIntoErr {
        /// From type
        from: &'static str,
        /// Into type
        into: &'static str,
        /// TryError
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// There was a problem with the database connection
    #[error("Connection Error: {0}")]
    Conn(#[source] RuntimeErr),
    /// An operation did not execute successfully
    #[error("Execution Error: {0}")]
    Exec(#[source] RuntimeErr),
    /// An error occurred while performing a query
    #[error("Query Error: {0}")]
    Query(#[source] RuntimeErr),
    /// Type error: the specified type cannot be converted from u64. This is not a runtime error.
    #[error("Type '{0}' cannot be converted from u64")]
    ConvertFromU64(&'static str),
    /// After an insert statement it was impossible to retrieve the last_insert_id
    #[error("Failed to unpack last_insert_id")]
    UnpackInsertId,
    /// When updating, a model should know its primary key to check
    /// if the record has been correctly updated, otherwise this error will occur
    #[error("Failed to get primary key from model")]
    UpdateGetPrimaryKey,
    /// The record was not found in the database
    #[error("RecordNotFound Error: {0}")]
    RecordNotFound(String),
    /// Thrown by `TryFrom<ActiveModel>`, which assumes all attributes are set/unchanged
    #[error("Attribute {0} is NotSet")]
    AttrNotSet(String),
    /// A custom error
    #[error("Custom Error: {0}")]
    Custom(String),
    /// Error occurred while parsing value as target type
    #[error("Type Error: {0}")]
    Type(String),
    /// Error occurred while parsing json value as target type
    #[error("Json Error: {0}")]
    Json(String),
    /// A migration error
    #[error("Migration Error: {0}")]
    Migration(String),
    /// None of the records are inserted,
    /// that probably means all of them conflict with existing records in the table
    #[error("None of the records are inserted")]
    RecordNotInserted,
    /// None of the records are updated, that means a WHERE condition has no matches.
    /// May be the table is empty or the record does not exist
    #[error("None of the records are updated")]
    RecordNotUpdated,
}

/// Runtime error
#[derive(Error, Debug)]
pub enum RuntimeErr {
    /// SQLx Error
    #[cfg(feature = "sqlx-dep")]
    #[error("{0}")]
    SqlxError(sqlx::error::Error),
    /// Error generated from within SeaORM
    #[error("{0}")]
    Internal(String),
}

impl PartialEq for DbErr {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for DbErr {}

/// Error during `impl FromStr for Entity::Column`
#[derive(Error, Debug)]
#[error("Failed to match \"{0}\" as Column")]
pub struct ColumnFromStrErr(pub String);

#[allow(dead_code)]
pub(crate) fn conn_err<T>(s: T) -> DbErr
where
    T: ToString,
{
    DbErr::Conn(RuntimeErr::Internal(s.to_string()))
}

#[allow(dead_code)]
pub(crate) fn exec_err<T>(s: T) -> DbErr
where
    T: ToString,
{
    DbErr::Exec(RuntimeErr::Internal(s.to_string()))
}

#[allow(dead_code)]
pub(crate) fn query_err<T>(s: T) -> DbErr
where
    T: ToString,
{
    DbErr::Query(RuntimeErr::Internal(s.to_string()))
}

#[allow(dead_code)]
pub(crate) fn type_err<T>(s: T) -> DbErr
where
    T: ToString,
{
    DbErr::Type(s.to_string())
}

#[allow(dead_code)]
pub(crate) fn json_err<T>(s: T) -> DbErr
where
    T: ToString,
{
    DbErr::Json(s.to_string())
}

#[cfg(all(feature = "sqlx-dep"))]
impl DbErr {
    /// Returns if the error indicates that a column is not found in the query.
    pub fn is_column_not_found_error(&self) -> bool {
        match self {
            Self::Type(err) if err.starts_with("A null value was encountered") => true,
            Self::Query(RuntimeErr::SqlxError(sqlx::error::Error::ColumnNotFound(_))) => true,
            _ => false,
        }
    }
}

#[cfg(not(all(feature = "sqlx-dep")))]
impl DbErr {
    /// Returns if the error indicates that a column is not found in the query.
    pub fn is_column_not_found_error(&self) -> bool {
        matches!(self, Self::Type(err) if err.starts_with("A null value was encountered"))
    }
}
