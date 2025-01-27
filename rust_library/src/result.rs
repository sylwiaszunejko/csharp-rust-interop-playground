use scylla::_macro_internal::ColumnType;
use scylla::deserialize::DeserializeValue;


// You can implement DeserializeValue for your own types
#[derive(PartialEq, Eq, Debug)]
pub (crate) struct WQueryResult(*const u8);

impl<'frame, 'metadata> DeserializeValue<'frame, 'metadata> for WQueryResult {
    fn type_check(
        _: &ColumnType,
    ) -> Result<(), scylla::deserialize::TypeCheckError> {
        Ok(())
    }

    fn deserialize(
        _: &'metadata ColumnType<'metadata>,
        v: Option<scylla::deserialize::FrameSlice<'frame>>,
    ) -> Result<Self, scylla::deserialize::DeserializationError> {
        if v.is_none() {
            return Ok(WQueryResult(std::ptr::null()));
        }
        Ok(WQueryResult(v.unwrap().to_bytes().as_ptr()))
    }
}