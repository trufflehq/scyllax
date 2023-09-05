//! ScyllaX error types

/// An error from ScyllaX
#[derive(thiserror::Error, Debug)]
pub enum ScyllaxError {
    /// A query error from Scylla
    #[error("Scylla Query error: {0}")]
    Scylla(#[from] scylla::transport::errors::QueryError),

    /// An error thrown when trying to parse a single row
    #[error("Scylla single row typed error: {0}")]
    SingleRowTyped(#[from] scylla::transport::query_result::SingleRowTypedError),

    /// No rows were found when trying to parse a single row
    #[error("No rows found")]
    NoRowsFound,

    /// There was an error when building an upsert query.
    #[error("Failed to build query: {0}")]
    BuildUpsertQueryError(#[from] BuildUpsertQueryError),
}

/// An error when building an upsert query
#[derive(thiserror::Error, Debug)]
pub enum BuildUpsertQueryError {
    /// There were too many values (usually ignored since we don't set a capacity on [`scylla::frame::value::SerializedValues`]])
    #[error("Too many values when adding {field}")]
    TooManyValues {
        /// The field being added when the error was thrown
        field: String,
    },

    /// You cant mix named and unnamed values (trating SerializedValues as both a HashMap and a Vec).
    /// This is only added for `scylla`-error parity
    #[error("Can't mix named and unnamed values")]
    MixingNamedAndNotNamedValues,

    /// The value for a field was too big
    #[error("Value for {field} is too big")]
    ValueTooBig {
        /// The field being added when the error was thrown
        field: String,
    },

    /// Failed to parse a value
    #[error("Failed to serialize value for {field}")]
    ParseError {
        /// The field being added when the error was thrown
        field: String,
    },
}