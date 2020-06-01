use geozero_core::wkb;
use geozero_core::wkt::WktWriter;
use sqlx::postgres::{PgPool, PgQueryAs};
use tokio::runtime::Runtime;

async fn sqlx() -> Result<(), sqlx::Error> {
    let pool = PgPool::builder()
        .max_size(5)
        .build(&std::env::var("DATABASE_URL").unwrap())
        .await?;

    let row: (Vec<u8>,) =
        sqlx::query_as("SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry::bytea")
            .fetch_one(&pool)
            .await?;

    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::new(&mut wkt_data);
    assert!(wkb::process_wkb_geom(&mut row.0.as_slice(), &mut writer).is_ok());
    assert_eq!(
        std::str::from_utf8(&wkt_data).unwrap(),
        "POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))"
    );

    Ok(())
}

#[test]
#[ignore]
fn postgis_sqlx() {
    assert!(Runtime::new().unwrap().block_on(sqlx()).is_ok());
}

mod register_type {
    use super::*;
    use sqlx::decode::Decode;
    use sqlx::postgres::{PgData, PgTypeInfo, PgValue, Postgres};

    struct WkbToWkt {
        wkt: String,
    }

    impl sqlx::Type<Postgres> for WkbToWkt {
        fn type_info() -> PgTypeInfo {
            PgTypeInfo::with_name("geometry")
        }
    }

    impl<'de> Decode<'de, Postgres> for WkbToWkt {
        fn decode(value: PgValue<'de>) -> sqlx::Result<Self> {
            match value.get() {
                Some(PgData::Binary(mut buf)) => {
                    let mut wkt_data: Vec<u8> = Vec::new();
                    let mut writer = WktWriter::new(&mut wkt_data);
                    wkb::process_wkb_geom(&mut buf, &mut writer).unwrap();
                    let wkt = WkbToWkt {
                        wkt: std::str::from_utf8(&wkt_data).unwrap().to_string(),
                    };
                    Ok(wkt)
                }
                Some(PgData::Text(_s)) => Err(sqlx::Error::Decode(
                    "supporting binary geometry format only".into(),
                )),
                None => Ok(WkbToWkt {
                    wkt: "EMPTY".to_string(),
                }),
            }
        }
    }

    async fn sqlx() -> Result<(), sqlx::Error> {
        let pool = PgPool::builder()
            .max_size(5)
            .build(&std::env::var("DATABASE_URL").unwrap())
            .await?;

        let row: (WkbToWkt,) =
            sqlx::query_as("SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry")
                .fetch_one(&pool)
                .await?;
        assert_eq!(row.0.wkt, "POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))");

        let row: (WkbToWkt,) = sqlx::query_as("SELECT NULL::geometry")
            .fetch_one(&pool)
            .await?;
        assert_eq!(row.0.wkt, "EMPTY");

        Ok(())
    }

    #[test]
    #[ignore]
    fn postgis_sqlx() {
        assert!(Runtime::new().unwrap().block_on(sqlx()).is_ok());
    }
}
