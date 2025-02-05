use scylla::_macro_internal::{
    ColumnIterator, ColumnSpec, ColumnType, DeserializationError, TypeCheckError,
};
use scylla::deserialize::{DeserializeRow, DeserializeValue};
use std::ffi::c_int;
use std::ptr::null;

// You can implement DeserializeValue for your own types
#[repr(C)]
#[derive(PartialEq, Eq, Debug)]
pub(crate) struct WQueryResult {
    rows: *const Row,
    len: c_int,
}

#[repr(C)]
pub(crate) struct Row {
    buffer: *const u8,
    len: c_int,
    offset: c_int,
}

impl From<Vec<Row>> for WQueryResult {
    fn from(value: Vec<Row>) -> Self {
        WQueryResult {
            rows: value.as_slice().as_ptr(),
            len: value.len() as c_int,
        }
    }
}

impl<'frame, 'metadata> DeserializeRow<'frame, 'metadata> for WQueryResult {
    fn type_check(specs: &[ColumnSpec]) -> Result<(), TypeCheckError> {
        Ok(())
    }

    /// Deserializes a row from given column iterator.
    ///
    /// This function can assume that the driver called `type_check` to verify
    /// the row's type. Note that `deserialize` is not an unsafe function,
    /// so it should not use the assumption about `type_check` being called
    /// as an excuse to run `unsafe` code.
    fn deserialize(
        mut row: ColumnIterator<'frame, 'metadata>,
    ) -> Result<Self, DeserializationError> {
        let mut rows: Vec<Row> = vec![];

        while let Some(column) = row.next().transpose().map_err(DeserializationError::new)? {
            match column.slice {
                Some(slice) => {
                    let slice = slice.as_slice();
                    rows.push(Row {
                        buffer: slice.as_ptr(),
                        offset: 0,
                        len: slice.len() as c_int,
                    });
                }
                None => {
                    rows.push(Row {
                        buffer: null(),
                        offset: 0,
                        len: 0,
                    });
                }
            }
        }

        //         object Deserialize(ProtocolVersion version, byte[] buffer, int offset, int length, ColumnTypeCode typeCode, IColumnInfo typeInfo);
        Ok(rows.into())
    }
}
