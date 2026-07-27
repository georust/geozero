//! PostGIS geometry type encoding/decoding for Diesel.

mod reader;
mod sql;
mod writer;

pub use writer::PostgisDieselWriter;

pub mod sql_types {
    use diesel::query_builder::QueryId;
    use diesel::sql_types::SqlType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "geometry"))]
    pub struct Geometry;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "geography"))]
    pub struct Geography;
}
