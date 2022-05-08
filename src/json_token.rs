use crate::json_value::JsonValue;

#[derive(Debug, PartialEq)]
pub enum JsonToken {
    ObjBeg,
    ObjEnd,
    ArrBeg,
    ArrEnd,
    Key(String),
    Val(JsonValue),
}
