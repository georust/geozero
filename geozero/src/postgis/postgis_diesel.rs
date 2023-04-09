use crate::postgis::postgis_diesel::sql_types::{Geography, Geometry};
use crate::wkb::Ewkb;
use std::io::Write as _;

use diesel::deserialize::{self, FromSql};
use diesel::pg::{self, Pg};
use diesel::serialize::{self, IsNull, Output, ToSql};

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

impl ToSql<Geometry, Pg> for Ewkb {
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        out.write_all(&self.0)?;
        Ok(IsNull::No)
    }
}

impl ToSql<Geography, Pg> for Ewkb {
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        out.write_all(&self.0)?;
        Ok(IsNull::No)
    }
}

impl FromSql<Geometry, Pg> for Ewkb {
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        Ok(Self(bytes.as_bytes().to_vec()))
    }
}

impl FromSql<Geography, Pg> for Ewkb {
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        Ok(Self(bytes.as_bytes().to_vec()))
    }
}
