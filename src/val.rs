use std::str;

use crate::constants::{DIGIT_CHAR_SET, EMPTY_CHAR_SET};
use crate::error::{Error, ErrorCode};
use crate::json_value::JsonValue;
use crate::token::*;
use crate::utils::squash;

pub(crate) fn handle_string<F>(
    buf: &[u8],
    i: usize,
    tokens: &mut Vec<Token>,
    mut on_val: F,
) -> Option<Error>
where
    F: FnMut(JsonValue),
{
    let mut error: Option<Error> = None;
    match tokens.last_mut() {
        Some(Token::String(ref mut data)) => match buf[i] {
            // escaped chars
            b'\\' if data.last() == Some(&b'\\') => data.push(0),
            b'"' if data.last() == Some(&b'\\') => data.push(b'"'),
            b'"' => {
                let v = data
                    .clone()
                    .into_iter()
                    .filter(|&ch| ch != 0)
                    .collect::<Vec<u8>>();
                let val = str::from_utf8(&v).unwrap();
                on_val(JsonValue::String(val.to_string()));
                squash(tokens);
                tokens.push(Token::None);
            }
            ch => data.push(ch),
        },
        _ => {
            error = Some(Error {
                column: i,
                code: ErrorCode::InvalidFormat,
            })
        }
    }
    error
}

pub(crate) fn handle_number<V, A, O>(
    buf: &[u8],
    i: usize,
    tokens: &mut Vec<Token>,
    mut on_val: V,
    mut on_arr_end: A,
    mut on_obj_end: O,
) -> Option<Error>
where
    V: FnMut(JsonValue),
    A: FnMut(),
    O: FnMut(),
{
    let mut error: Option<Error> = None;
    if let Some(Token::Number(ref mut data)) = tokens.last_mut() {
        match buf[i] {
            b'.' if !data.contains(&b'.') => data.push(b'.'),
            b',' => {
                on_val(JsonValue::Number(
                    str::from_utf8(&data).unwrap().to_string(),
                ));
                squash(tokens);
                tokens.push(Token::Comma);
            }
            b']' => {
                on_val(JsonValue::Number(
                    str::from_utf8(&data).unwrap().to_string(),
                ));
                squash(tokens);
                handle_end_arr(tokens);
                on_arr_end();
            }
            b'}' => {
                on_val(JsonValue::Number(
                    str::from_utf8(&data).unwrap().to_string(),
                ));
                squash(tokens);
                handle_end_obj(tokens);
                on_obj_end();
            }
            ch if DIGIT_CHAR_SET.contains(&ch) => data.push(ch),
            ch if EMPTY_CHAR_SET.contains(&ch) => {
                on_val(JsonValue::Number(
                    str::from_utf8(&data).unwrap().to_string(),
                ));
                squash(tokens);
                tokens.push(Token::None);
            }
            _ => {
                error = Some(Error {
                    column: i,
                    code: ErrorCode::InvalidNumber,
                })
            }
        }
    }
    error
}

pub(crate) fn handle_null<V>(
    buf: &[u8],
    i: usize,
    tokens: &mut Vec<Token>,
    mut on_val: V,
) -> Option<Error>
where
    V: FnMut(JsonValue),
{
    let mut error: Option<Error> = None;
    if let Some(Token::Null(ref mut data)) = tokens.last_mut() {
        match buf[i] {
            b'u' if *data == [b'n'] => data.push(b'u'),
            b'l' if *data == [b'n', b'u'] => data.push(b'l'),
            b'l' if *data == [b'n', b'u', b'l'] => {
                on_val(JsonValue::Null);
                squash(tokens);
                tokens.push(Token::None);
            }
            _ => {
                error = Some(Error {
                    column: i,
                    code: ErrorCode::ExpectedNull,
                })
            }
        }
    }
    error
}

pub(crate) fn handle_true<V>(
    buf: &[u8],
    i: usize,
    tokens: &mut Vec<Token>,
    mut on_val: V,
) -> Option<Error>
where
    V: FnMut(JsonValue),
{
    let mut error: Option<Error> = None;
    if let Some(Token::True(ref mut data)) = tokens.last_mut() {
        match buf[i] {
            b'r' if *data == [b't'] => data.push(b'r'),
            b'u' if *data == [b't', b'r'] => data.push(b'u'),
            b'e' if *data == [b't', b'r', b'u'] => {
                data.push(b'e');
                on_val(JsonValue::Bool(true));
                squash(tokens);
                tokens.push(Token::None);
            }
            _ => {
                error = Some(Error {
                    column: i,
                    code: ErrorCode::ExpectedTrue,
                })
            }
        }
    }
    error
}

pub(crate) fn handle_false<V>(
    buf: &[u8],
    i: usize,
    tokens: &mut Vec<Token>,
    mut on_val: V,
) -> Option<Error>
where
    V: FnMut(JsonValue),
{
    let mut error: Option<Error> = None;
    if let Some(Token::False(ref mut data)) = tokens.last_mut() {
        match buf[i] {
            b'a' if *data == [b'f'] => data.push(b'a'),
            b'l' if *data == [b'f', b'a'] => data.push(b'l'),
            b's' if *data == [b'f', b'a', b'l'] => data.push(b's'),
            b'e' if *data == [b'f', b'a', b'l', b's'] => {
                data.push(b'e');
                on_val(JsonValue::Bool(false));
                squash(tokens);
                tokens.push(Token::None);
            }
            _ => {
                error = Some(Error {
                    column: i,
                    code: ErrorCode::ExpectedFalse,
                })
            }
        }
    }
    error
}

fn handle_end_obj(tokens: &mut Vec<Token>) {
    while let Some(token) = tokens.pop() {
        if let Token::Obj = token {
            break;
        }
    }
    tokens.push(Token::None);
}

fn handle_end_arr(tokens: &mut Vec<Token>) {
    while let Some(token) = tokens.pop() {
        if let Token::Arr = token {
            break;
        }
    }
    tokens.push(Token::None);
}

#[cfg(test)]
mod handle_string_tests {
    use super::*;

    #[test]
    fn should_parse_string_and_squash() {
        let buf = "\"foo\"".as_bytes();
        let mut tokens = vec![Token::String(vec![])];
        let mut i = 1;
        let mut res = String::new();
        while i < buf.len() {
            handle_string(buf, i, &mut tokens, |val: JsonValue| {
                if let JsonValue::String(str) = val {
                    res = str;
                }
            });
            i += 1;
        }
        assert_eq!(res, "foo");
        assert_eq!(tokens.last(), Some(&Token::None));
    }

    #[test]
    fn should_handle_escaped_double_quote() {
        let buf = r#""foo\"bar""#.as_bytes();
        let mut tokens = vec![Token::String(vec![])];
        let mut i = 1;
        let mut res = String::new();
        while i < buf.len() {
            handle_string(buf, i, &mut tokens, |val: JsonValue| {
                if let JsonValue::String(str) = val {
                    res = str;
                }
            });
            i += 1;
        }
        assert_eq!(res, r#"foo\"bar"#);
        assert_eq!(tokens.last(), Some(&Token::None));
    }
}

#[cfg(test)]
mod handle_number_tests {
    use super::*;

    #[test]
    fn should_parse_integer() {
        let buf = r#"42,"#.as_bytes();
        let mut tokens = vec![Token::Number(vec![])];
        let mut i = 0;
        let mut num = String::new();
        while i < buf.len() {
            handle_number(
                buf,
                i,
                &mut tokens,
                |val: JsonValue| {
                    if let JsonValue::Number(str) = val {
                        num = str;
                    }
                },
                || {},
                || {},
            );
            i += 1;
        }
        assert_eq!(num, r#"42"#);
        assert_eq!(tokens.last(), Some(&Token::Comma));
    }

    #[test]
    fn should_parse_float() {
        let buf = r#"42.123,"#.as_bytes();
        let mut tokens = vec![Token::Number(vec![])];
        let mut i = 0;
        let mut num = String::new();
        while i < buf.len() {
            handle_number(
                buf,
                i,
                &mut tokens,
                |val: JsonValue| {
                    if let JsonValue::Number(str) = val {
                        num = str;
                    }
                },
                || {},
                || {},
            );
            i += 1;
        }
        assert_eq!(num, r#"42.123"#);
        assert_eq!(tokens.last(), Some(&Token::Comma));
    }

    #[test]
    fn should_parse_float_e() {
        let buf = r#"42e-123,"#.as_bytes();
        let mut tokens = vec![Token::Number(vec![])];
        let mut i = 0;
        let mut num = String::new();
        while i < buf.len() {
            handle_number(
                buf,
                i,
                &mut tokens,
                |val: JsonValue| {
                    if let JsonValue::Number(str) = val {
                        num = str;
                    }
                },
                || {},
                || {},
            );
            i += 1;
        }
        assert_eq!(num, r#"42e-123"#);
        assert_eq!(tokens.last(), Some(&Token::Comma));
    }

    #[test]
    fn should_handle_obj_end() {
        let buf = r#"42}"#.as_bytes();
        let mut tokens = vec![Token::Number(vec![])];
        let mut i = 0;
        let mut num = String::new();
        while i < buf.len() {
            handle_number(
                buf,
                i,
                &mut tokens,
                |val: JsonValue| {
                    if let JsonValue::Number(str) = val {
                        num = str;
                    }
                },
                || {},
                || {},
            );
            i += 1;
        }
        assert_eq!(num, r#"42"#);
        assert_eq!(tokens.last(), Some(&Token::None));
    }

    #[test]
    fn should_handle_arr_end() {
        let buf = r#"42]"#.as_bytes();
        let mut tokens = vec![Token::Number(vec![])];
        let mut i = 0;
        let mut num = String::new();
        while i < buf.len() {
            handle_number(
                buf,
                i,
                &mut tokens,
                |val: JsonValue| {
                    if let JsonValue::Number(str) = val {
                        num = str;
                    }
                },
                || {},
                || {},
            );
            i += 1;
        }
        assert_eq!(num, r#"42"#);
        assert_eq!(tokens.last(), Some(&Token::None));
    }

    #[test]
    fn should_return_error_if_not_digit() {
        let buf = r#"42b"#.as_bytes();
        let mut tokens = vec![Token::Number(vec![])];
        let mut i = 0;
        let mut num = String::new();
        let mut error = None;
        while i < buf.len() && error.is_none() {
            error = handle_number(
                buf,
                i,
                &mut tokens,
                |val: JsonValue| {
                    if let JsonValue::Number(str) = val {
                        num = str;
                    }
                },
                || {},
                || {},
            );
            i += 1;
        }
        assert_eq!(
            error,
            Some(Error {
                column: 2,
                code: ErrorCode::InvalidNumber
            })
        );
    }
}

#[cfg(test)]
mod handle_null {
    use super::*;

    #[test]
    fn should_parse_null() {
        let buf = r#"null,"#.as_bytes();
        let mut tokens = vec![Token::Null(vec![b'n'])];
        let mut i = 1;
        let mut is_null = false;
        let mut error = None;
        while i < buf.len() && error.is_none() {
            error = handle_null(buf, i, &mut tokens, |val: JsonValue| {
                if let JsonValue::Null = val {
                    is_null = true;
                }
            });
            i += 1;
        }
        assert!(is_null);
        assert_eq!(tokens.last(), Some(&Token::None));
        assert_eq!(error, None);
    }

    #[test]
    fn should_return_error() {
        let buf = r#"nil,"#.as_bytes();
        let mut tokens = vec![Token::Null(vec![b'n'])];
        let mut i = 1;
        let mut is_null = false;
        let mut error = None;
        while i < buf.len() && error.is_none() {
            error = handle_null(buf, i, &mut tokens, |val: JsonValue| {
                if let JsonValue::Null = val {
                    is_null = true;
                }
            });
            i += 1;
        }
        assert!(!is_null);
        assert_eq!(
            error,
            Some(Error {
                column: 1,
                code: ErrorCode::ExpectedNull
            })
        );
    }
}

#[cfg(test)]
mod handle_true {
    use super::*;

    #[test]
    fn should_parse_true() {
        let buf = r#"true,"#.as_bytes();
        let mut tokens = vec![Token::True(vec![b't'])];
        let mut i = 1;
        let mut is_true = false;
        let mut error = None;
        while i < buf.len() && error.is_none() {
            error = handle_true(buf, i, &mut tokens, |val: JsonValue| {
                if let JsonValue::Bool(b) = val {
                    is_true = b;
                }
            });
            i += 1;
        }
        assert!(is_true);
        assert_eq!(tokens.last(), Some(&Token::None));
        assert_eq!(error, None);
    }

    #[test]
    fn should_return_error() {
        let buf = r#"truE,"#.as_bytes();
        let mut tokens = vec![Token::True(vec![b't'])];
        let mut i = 1;
        let mut is_true = false;
        let mut error = None;
        while i < buf.len() && error.is_none() {
            error = handle_true(buf, i, &mut tokens, |val: JsonValue| {
                if let JsonValue::Bool(b) = val {
                    is_true = b;
                }
            });
            i += 1;
        }
        assert!(!is_true);
        assert_eq!(
            error,
            Some(Error {
                column: 3,
                code: ErrorCode::ExpectedTrue
            })
        );
    }
}

#[cfg(test)]
mod handle_false {
    use super::*;

    #[test]
    fn should_parse_false() {
        let buf = r#"false,"#.as_bytes();
        let mut tokens = vec![Token::False(vec![b'f'])];
        let mut i = 1;
        let mut is_false = false;
        let mut error = None;
        while i < buf.len() && error.is_none() {
            error = handle_false(buf, i, &mut tokens, |val: JsonValue| {
                if let JsonValue::Bool(false) = val {
                    is_false = true;
                }
            });
            i += 1;
        }
        assert!(is_false);
        assert_eq!(tokens.last(), Some(&Token::None));
        assert_eq!(error, None);
    }

    #[test]
    fn should_return_error() {
        let buf = r#"falz,"#.as_bytes();
        let mut tokens = vec![Token::False(vec![b'f'])];
        let mut i = 1;
        let mut is_false = false;
        let mut error = None;
        while i < buf.len() && error.is_none() {
            error = handle_false(buf, i, &mut tokens, |val: JsonValue| {
                if let JsonValue::Bool(false) = val {
                    is_false = true;
                }
            });
            i += 1;
        }
        assert!(!is_false);
        assert_eq!(
            error,
            Some(Error {
                column: 3,
                code: ErrorCode::ExpectedFalse
            })
        );
    }
}
