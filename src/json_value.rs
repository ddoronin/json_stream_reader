#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    String(String),
    Number(String),
}
