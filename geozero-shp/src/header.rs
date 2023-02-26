use crate::point_z::BBoxZ;
use crate::Error;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::fmt;
use std::io::Read;

pub(crate) const HEADER_SIZE: i32 = 100;
const FILE_CODE: i32 = 9994;
/// Size of reserved bytes in the header, that have do defined use
const SIZE_OF_SKIP: usize = std::mem::size_of::<i32>() * 5;

/// struct representing the Header of a shapefile
/// can be retrieved via the reader used to read
#[derive(Copy, Clone, PartialEq)]
pub struct Header {
    /// Total file length (Header + Shapes) in 16bit word
    pub file_length: i32,
    /// The bbox contained all the shapes in this shapefile
    ///
    /// For shapefiles where the shapes do not have `m` or `z` values
    /// the associated min and max will be `0`s.
    pub bbox: BBoxZ,
    /// Type of all the shapes in the file
    /// (as mixing shapes is not allowed)
    pub shape_type: ShapeType,
    /// Version of the shapefile specification
    pub version: i32,
}

impl Default for Header {
    fn default() -> Self {
        Header {
            bbox: BBoxZ::default(),
            shape_type: ShapeType::NullShape,
            file_length: HEADER_SIZE / 2,
            version: 1000,
        }
    }
}

impl Header {
    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Header, Error> {
        let file_code = source.read_i32::<BigEndian>()?;

        if file_code != FILE_CODE {
            return Err(Error::InvalidFileCode(file_code));
        }

        let mut skip: [u8; SIZE_OF_SKIP] = [0; SIZE_OF_SKIP];
        source.read_exact(&mut skip)?;

        let file_length_16_bit = source.read_i32::<BigEndian>()?;
        let version = source.read_i32::<LittleEndian>()?;
        let shape_type = ShapeType::read_from(&mut source)?;

        let mut hdr = Header {
            shape_type,
            version,
            file_length: file_length_16_bit,
            ..Default::default()
        };

        hdr.bbox.min.x = source.read_f64::<LittleEndian>()?;
        hdr.bbox.min.y = source.read_f64::<LittleEndian>()?;
        hdr.bbox.max.x = source.read_f64::<LittleEndian>()?;
        hdr.bbox.max.y = source.read_f64::<LittleEndian>()?;
        hdr.bbox.min.z = source.read_f64::<LittleEndian>()?;
        hdr.bbox.max.z = source.read_f64::<LittleEndian>()?;
        hdr.bbox.min.m = source.read_f64::<LittleEndian>()?;
        hdr.bbox.max.m = source.read_f64::<LittleEndian>()?;

        Ok(hdr)
    }
}

/// The enum for the ShapeType as defined in the
/// specification
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ShapeType {
    NullShape = 0,
    Point = 1,
    Polyline = 3,
    Polygon = 5,
    Multipoint = 8,

    PointZ = 11,
    PolylineZ = 13,
    PolygonZ = 15,
    MultipointZ = 18,

    PointM = 21,
    PolylineM = 23,
    PolygonM = 25,
    MultipointM = 28,

    Multipatch = 31,
}

impl ShapeType {
    pub(crate) fn read_from<T: Read>(source: &mut T) -> Result<ShapeType, Error> {
        let code = source.read_i32::<LittleEndian>()?;
        Self::from(code).ok_or_else(|| Error::InvalidShapeType(code))
    }

    /// Returns the ShapeType corresponding to the input code
    /// if the code is valid
    /// ```
    /// use geozero_shp::ShapeType;
    ///
    /// assert_eq!(ShapeType::from(25), Some(ShapeType::PolygonM));
    /// assert_eq!(ShapeType::from(60), None);
    /// ```
    pub fn from(code: i32) -> Option<ShapeType> {
        match code {
            0 => Some(ShapeType::NullShape),
            1 => Some(ShapeType::Point),
            3 => Some(ShapeType::Polyline),
            5 => Some(ShapeType::Polygon),
            8 => Some(ShapeType::Multipoint),
            11 => Some(ShapeType::PointZ),
            13 => Some(ShapeType::PolylineZ),
            15 => Some(ShapeType::PolygonZ),
            18 => Some(ShapeType::MultipointZ),
            21 => Some(ShapeType::PointM),
            23 => Some(ShapeType::PolylineM),
            25 => Some(ShapeType::PolygonM),
            28 => Some(ShapeType::MultipointM),
            31 => Some(ShapeType::Multipatch),
            _ => None,
        }
    }

    /// Returns whether the ShapeType has the third dimension Z
    pub fn has_z(self) -> bool {
        matches!(
            self,
            ShapeType::PointZ | ShapeType::PolylineZ | ShapeType::PolygonZ | ShapeType::MultipointZ
        )
    }

    /// Returns whether the ShapeType has the optional measure dimension
    pub fn has_m(self) -> bool {
        matches!(
            self,
            ShapeType::PointZ
                | ShapeType::PolylineZ
                | ShapeType::PolygonZ
                | ShapeType::MultipointZ
                | ShapeType::PointM
                | ShapeType::PolylineM
                | ShapeType::PolygonM
                | ShapeType::MultipointM
        )
    }

    /// Returns true if the shape may have multiple parts
    pub fn is_multipart(self) -> bool {
        !matches!(
            self,
            ShapeType::Point
                | ShapeType::PointM
                | ShapeType::PointZ
                | ShapeType::Multipoint
                | ShapeType::MultipointM
                | ShapeType::MultipointZ
        )
    }
}

impl fmt::Display for ShapeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShapeType::NullShape => write!(f, "NullShape"),
            ShapeType::Point => write!(f, "Point"),
            ShapeType::Polyline => write!(f, "Polyline"),
            ShapeType::Polygon => write!(f, "Polygon"),
            ShapeType::Multipoint => write!(f, "Multipoint"),
            ShapeType::PointZ => write!(f, "PointZ"),
            ShapeType::PolylineZ => write!(f, "PolylineZ"),
            ShapeType::PolygonZ => write!(f, "PolygonZ"),
            ShapeType::MultipointZ => write!(f, "MultipointZ"),
            ShapeType::PointM => write!(f, "PointM"),
            ShapeType::PolylineM => write!(f, "PolylineM"),
            ShapeType::PolygonM => write!(f, "PolygonM"),
            ShapeType::MultipointM => write!(f, "MultipointM"),
            ShapeType::Multipatch => write!(f, "Multipatch"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use byteorder::WriteBytesExt;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn wrong_file_code() {
        use std::io::Cursor;

        let mut src = Cursor::new(vec![]);
        src.write_i32::<BigEndian>(42).unwrap();

        src.seek(SeekFrom::Start(0)).unwrap();
        assert!(Header::read_from(&mut src).is_err());
    }
}
