//! When json parsing goes wrong.
use core::result;

// A list specifying categories of JSON parser errors.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum ErrorCode {
    /// Expected this character to be a `':'`.
    ExpectedColon,

    /// Expected a key "foo".
    ExpectedKey,

    /// Expected a number.
    ExpectedNumber,

    /// Expected this character to be either a `'{'` or a `'['`.
    ExpectedObjectOrArray,

    /// Expected this character to be either a `','`, or a `'}'` or a `']'`.
    ExpectedCommaOrObjectEndOrArrayEnd,

    /// Expected any term such as number, boolean, string, `'{'` or `'['`.
    ExpectedAnyTerm,

    /// Expected `'true'`.
    ExpectedTrue,

    /// Expected `'false'`.
    ExpectedFalse,

    /// Expected `'null'`.
    ExpectedNull,

    /// The number is invalid, it contains non-digit characters.
    InvalidNumber,

    /// Expected a string.
    ExpectedString,

    /// Expected this character to be either a `','` or a `']'`.
    ExpectedListCommaOrEnd,

    /// Expected this character to be either a `','` or a `'}'`.
    ExpectedObjectCommaOrEnd,

    /// Stop signal was sent via a callback function.
    StopSignal(String),

    /// The length of an object key is too long.
    TooLongKey,

    /// Happens when the JSON Tokenizer returns wrong tokens.
    InvalidFormat,

    /// The level of objects/arrays nesting is too high.
    TooManyTokens,

    /// The array is invalid. It started with [ followed by an invalid character.
    InvalidArrFormat,
}

#[derive(Debug, PartialEq)]
pub struct Error {
    pub code: ErrorCode,
    pub column: usize,
}

// Alias for a `Result` w/ the error type `Error`.
pub type Result<T> = result::Result<T, Error>;
