use crate::arrow::process_geoarrow_wkb_feature_chunk;
use crate::error::{GeozeroError, Result};
use arrow2::io::parquet::read::{infer_schema, read_metadata, FileReader};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io};

#[derive(Serialize, Deserialize, Debug)]
pub struct GeoParquetFileMetadata {
    /// The version of the GeoParquet metadata standard used when writing.
    version: String,

    /// The name of the 'primary' geometry column.
    primary_column: String,

    /// "Metadata about geometry columns, where each key is the name of a geometry column in the
    /// table.
    columns: HashMap<String, GeoParquetColumnMetadata>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeoParquetColumnMetadata {
    /// Name of the geometry encoding format. Currently only 'WKB' is supported.
    encoding: String,

    /// The geometry types of all geometries, or an empty array if they are not known.
    geometry_types: Vec<String>,

    // JSON object representing the Coordinate Reference System (CRS) of the geometry. If the crs
    // field is not included then the data in this column must be stored in longitude, latitude
    // based on the WGS84 datum, and CRS-aware implementations should assume a default value of
    // OGC:CRS84.
    crs: Option<String>,

    /// Winding order of exterior ring of polygons; interior rings are wound in opposite order. If
    /// absent, no assertions are made regarding the winding order.
    orientation: Option<String>,

    /// Name of the coordinate system for the edges. Must be one of 'planar' or 'spherical'. The
    /// default value is 'planar'.
    edges: Option<String>,

    /// Bounding Box of the geometries in the file, formatted according to RFC 7946, section 5.
    bbox: Option<Vec<f64>>,

    /// Coordinate epoch in case of a dynamic CRS, expressed as a decimal year.
    epoch: Option<f64>,
}

pub struct GeoParquetReader<'a, R: io::Read + io::Seek>(pub &'a mut R);

impl<'a, R: io::Read + io::Seek> crate::GeozeroDatasource for GeoParquetReader<'a, R> {
    fn process<P: crate::FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        process_geoparquet_features(&mut self.0, processor)
    }
}

// Can't implement GeozeroGeometry because parquet reader needs mutable reference and process_geom
// trait only provides non-mutable reference

pub fn process_geoparquet_features<R: io::Read + io::Seek, P: crate::FeatureProcessor>(
    reader: &mut R,
    processor: &mut P,
) -> Result<()> {
    let metadata = read_metadata(reader)?;
    let schema = infer_schema(&metadata)?;

    // TODO: is the geo metadata in the arrow metadata or the parquet metadata?
    let geo_metadata_string = schema
        .metadata
        .get("geo")
        .ok_or_else(|| GeozeroError::Dataset("Metadata missing geo key.".to_string()))?;

    let geo_metadata: GeoParquetFileMetadata = serde_json::from_str(geo_metadata_string)
        .map_err(|e| GeozeroError::Dataset(e.to_string()))?;

    let file_reader = FileReader::new(
        reader,
        metadata.row_groups,
        schema.clone(),
        None,
        None,
        None,
    );

    let mut dataset_idx = 0;
    // Iterate over row groups in the parquet file
    for maybe_chunk in file_reader {
        let chunk = maybe_chunk.unwrap();
        process_geoarrow_wkb_feature_chunk(
            &chunk,
            &schema,
            processor,
            dataset_idx,
            &geo_metadata.primary_column,
        )?;
        dataset_idx += &chunk.len();
    }

    Ok(())
}

impl From<arrow2::error::Error> for GeozeroError {
    fn from(error: arrow2::error::Error) -> Self {
        match error {
            arrow2::error::Error::Io(io_err) => GeozeroError::IoError(io_err),
            _ => GeozeroError::Dataset(error.to_string()),
        }
    }
}

// impl From<dyn std::error::Error> for GeozeroError {
//     fn from(error: dyn std::error::Error) -> Self {
//         GeozeroError::Dataset(error.to_string())
//     }
// }
