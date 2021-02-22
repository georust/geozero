//! PostGIS geometry type encoding/decoding.
#[cfg(feature = "with-postgis-postgres")]
mod postgis_postgres;
#[cfg(feature = "with-postgis-sqlx")]
mod postgis_sqlx;

#[cfg(feature = "with-postgis-postgres")]
pub mod postgres {
    pub use super::postgis_postgres::*;
}
#[cfg(feature = "with-postgis-sqlx")]
pub mod sqlx {
    pub use super::postgis_sqlx::*;
}
