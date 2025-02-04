use scylla::_macro_internal::{
    ColumnIterator, ColumnSpec, ColumnType, DeserializationError, TypeCheckError,
};
use scylla::deserialize::{DeserializeRow, DeserializeValue};
use std::ptr::null;

// You can implement DeserializeValue for your own types
#[derive(PartialEq, Eq, Debug)]
pub(crate) struct WQueryResult(*const [*const u8]);

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
    fn deserialize(mut row: ColumnIterator<'frame, 'metadata>) -> Result<Self, DeserializationError> {
        // let mut data: [*const u8] =;
        while let Some(column) = row
            .next()
            .transpose()
            .map_err(DeserializationError::new)? {
            let tmp  = column.slice.unwrap().to_bytes().as_ptr();
        }
        Ok(WQueryResult(null()))
    }
}
