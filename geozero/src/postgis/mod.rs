//! PostGIS geometry type encoding/decoding.
#[cfg(feature = "postgis-postgres")]
mod postgis_postgres;
#[cfg(feature = "postgis-sqlx")]
mod postgis_sqlx;

#[cfg(feature = "postgis-postgres")]
pub mod postgres {
    pub use super::postgis_postgres::*;
}
#[cfg(feature = "postgis-sqlx")]
pub mod sqlx {
    pub use super::postgis_sqlx::*;
}
