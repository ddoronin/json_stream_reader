//! Token
//!
//! JSON data can be represented as a stack of tokens.
//!
//! Those tokens can store an intermediate state between re-buffering.
//!
//! Also tokens are useful for a state machine.
#[derive(Debug, PartialEq)]
pub(crate) enum Token {
    Obj,
    Arr,
    Key(Vec<u8>),
    AfterKey,
    Colon,
    Null(Vec<u8>),
    True(Vec<u8>),
    False(Vec<u8>),
    Number(Vec<u8>),
    String(Vec<u8>),
    Comma,
    None,
}
