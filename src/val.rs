use std::str;

use crate::constants::{DIGIT_CHAR_SET, EMPTY_CHAR_SET};
use crate::error::{Error, ErrorCode};
use crate::json_token::JsonToken;
use crate::json_value::JsonValue;
use crate::token::*;
use crate::utils::squash;

type Res = Result<Option<JsonToken>, Error>;

pub(crate) fn handle_string(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Res {
    match tokens.last_mut() {
        Some(Token::String(ref mut data)) => match buf[i] {
            // escaped chars
            b'\\' if data.last() == Some(&b'\\') => {
                data.push(0);
                Ok(None)
            }
            b'"' if data.last() == Some(&b'\\') => {
                data.push(b'"');
                Ok(None)
            }
            b'"' => {
                let v = data
                    .clone()
                    .into_iter()
                    .filter(|&ch| ch != 0)
                    .collect::<Vec<u8>>();

                squash(tokens);
                tokens.push(Token::None);

                match str::from_utf8(&v) {
                    Ok(val) => Ok(Some(JsonToken::Val(JsonValue::String(val.to_string())))),
                    Err(_) => Err(Error {
                        column: i,
                        code: ErrorCode::InvalidFormat,
                    }),
                }
            }
            ch => {
                data.push(ch);
                Ok(None)
            }
        },
        _ => Err(Error {
            column: i,
            code: ErrorCode::InvalidFormat,
        }),
    }
}

pub(crate) fn handle_number(
    buf: &[u8],
    i: usize,
    tokens: &mut Vec<Token>,
) -> Result<Option<(JsonToken, Option<JsonToken>)>, Error> {
    if let Some(Token::Number(ref mut data)) = tokens.last_mut() {
        match buf[i] {
            b'.' if !data.contains(&b'.') => {
                data.push(b'.');
                Ok(None)
            }
            b',' => {
                let val = JsonToken::Val(JsonValue::Number(
                    str::from_utf8(&data).unwrap().to_string(),
                ));
                squash(tokens);
                tokens.push(Token::Comma);
                Ok(Some((val, None)))
            }
            b']' => {
                let val = JsonToken::Val(JsonValue::Number(
                    str::from_utf8(&data).unwrap().to_string(),
                ));
                squash(tokens);
                handle_end_arr(tokens);
                Ok(Some((val, Some(JsonToken::ArrEnd))))
            }
            b'}' => {
                let val = JsonToken::Val(JsonValue::Number(
                    str::from_utf8(&data).unwrap().to_string(),
                ));
                squash(tokens);
                handle_end_obj(tokens);
                Ok(Some((val, Some(JsonToken::ObjEnd))))
            }
            ch if DIGIT_CHAR_SET.contains(&ch) => {
                data.push(ch);
                Ok(None)
            }
            ch if EMPTY_CHAR_SET.contains(&ch) => {
                let val = JsonToken::Val(JsonValue::Number(
                    str::from_utf8(&data).unwrap().to_string(),
                ));
                squash(tokens);
                tokens.push(Token::None);
                Ok(Some((val, None)))
            }
            _ => Err(Error {
                column: i,
                code: ErrorCode::InvalidNumber,
            }),
        }
    } else {
        Ok(None)
    }
}

pub(crate) fn handle_null(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Res {
    if let Some(Token::Null(ref mut data)) = tokens.last_mut() {
        match buf[i] {
            b'u' if *data == [b'n'] => {
                data.push(b'u');
                Ok(None)
            }
            b'l' if *data == [b'n', b'u'] => {
                data.push(b'l');
                Ok(None)
            }
            b'l' if *data == [b'n', b'u', b'l'] => {
                squash(tokens);
                tokens.push(Token::None);
                Ok(Some(JsonToken::Val(JsonValue::Null)))
            }
            _ => Err(Error {
                column: i,
                code: ErrorCode::ExpectedNull,
            }),
        }
    } else {
        Ok(None)
    }
}

pub(crate) fn handle_true(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Res {
    if let Some(Token::True(ref mut data)) = tokens.last_mut() {
        match buf[i] {
            b'r' if *data == [b't'] => {
                data.push(b'r');
                Ok(None)
            }
            b'u' if *data == [b't', b'r'] => {
                data.push(b'u');
                Ok(None)
            }
            b'e' if *data == [b't', b'r', b'u'] => {
                data.push(b'e');
                squash(tokens);
                tokens.push(Token::None);
                Ok(Some(JsonToken::Val(JsonValue::Bool(true))))
            }
            _ => Err(Error {
                column: i,
                code: ErrorCode::ExpectedTrue,
            }),
        }
    } else {
        Ok(None)
    }
}

pub(crate) fn handle_false(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Res {
    if let Some(Token::False(ref mut data)) = tokens.last_mut() {
        match buf[i] {
            b'a' if *data == [b'f'] => {
                data.push(b'a');
                Ok(None)
            }
            b'l' if *data == [b'f', b'a'] => {
                data.push(b'l');
                Ok(None)
            }
            b's' if *data == [b'f', b'a', b'l'] => {
                data.push(b's');
                Ok(None)
            }
            b'e' if *data == [b'f', b'a', b'l', b's'] => {
                data.push(b'e');
                squash(tokens);
                tokens.push(Token::None);
                Ok(Some(JsonToken::Val(JsonValue::Bool(false))))
            }
            _ => Err(Error {
                column: i,
                code: ErrorCode::ExpectedFalse,
            }),
        }
    } else {
        Ok(None)
    }
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
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_string(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(
            res,
            Ok(Some(JsonToken::Val(JsonValue::String("foo".to_string()))))
        );
        assert_eq!(tokens.last(), Some(&Token::None));
    }

    #[test]
    fn should_handle_escaped_double_quote() {
        let buf = r#""foo\"bar""#.as_bytes();
        let mut tokens = vec![Token::String(vec![])];
        let mut i = 1;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_string(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(
            res,
            Ok(Some(JsonToken::Val(JsonValue::String(
                r#"foo\"bar"#.to_string()
            ))))
        );
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
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_number(buf, i, &mut tokens);
            i += 1;
        }
        let v = JsonToken::Val(JsonValue::Number(r#"42"#.to_string()));
        assert_eq!(res.unwrap(), Some((v, None)));
        assert_eq!(tokens.last(), Some(&Token::Comma));
    }

    #[test]
    fn should_parse_float() {
        let buf = r#"42.123,"#.as_bytes();
        let mut tokens = vec![Token::Number(vec![])];
        let mut i = 0;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_number(buf, i, &mut tokens);
            i += 1;
        }
        let v = JsonToken::Val(JsonValue::Number(r#"42.123"#.to_string()));
        assert_eq!(res.unwrap(), Some((v, None)));
        assert_eq!(tokens.last(), Some(&Token::Comma));
    }

    #[test]
    fn should_parse_float_e() {
        let buf = r#"42e-123,"#.as_bytes();
        let mut tokens = vec![Token::Number(vec![])];
        let mut i = 0;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_number(buf, i, &mut tokens);
            i += 1;
        }
        let v = JsonToken::Val(JsonValue::Number(r#"42e-123"#.to_string()));
        assert_eq!(res.unwrap(), Some((v, None)));
        assert_eq!(tokens.last(), Some(&Token::Comma));
    }

    #[test]
    fn should_handle_obj_end() {
        let buf = r#"42}"#.as_bytes();
        let mut tokens = vec![Token::Number(vec![])];
        let mut i = 0;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_number(buf, i, &mut tokens);
            i += 1;
        }
        let v = JsonToken::Val(JsonValue::Number(r#"42"#.to_string()));
        assert_eq!(res.unwrap(), Some((v, Some(JsonToken::ObjEnd))));
        assert_eq!(tokens.last(), Some(&Token::None));
    }

    #[test]
    fn should_handle_arr_end() {
        let buf = r#"42]"#.as_bytes();
        let mut tokens = vec![Token::Number(vec![])];
        let mut i = 0;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_number(buf, i, &mut tokens);
            i += 1;
        }
        let v = JsonToken::Val(JsonValue::Number(r#"42"#.to_string()));
        assert_eq!(res.unwrap(), Some((v, Some(JsonToken::ArrEnd))));
        assert_eq!(tokens.last(), Some(&Token::None));
    }

    #[test]
    fn should_return_error_if_not_digit() {
        let buf = r#"42b"#.as_bytes();
        let mut tokens = vec![Token::Number(vec![])];
        let mut i = 0;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_number(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(
            res,
            Err(Error {
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
        let buf = r#"null"#.as_bytes();
        let mut tokens = vec![Token::Null(vec![b'n'])];
        let mut i = 1;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_null(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(tokens.last(), Some(&Token::None));
        assert_eq!(res.unwrap(), Some(JsonToken::Val(JsonValue::Null)));
    }

    #[test]
    fn should_return_error() {
        let buf = r#"nil"#.as_bytes();
        let mut tokens = vec![Token::Null(vec![b'n'])];
        let mut i = 1;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_null(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(
            res,
            Err(Error {
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
        let buf = r#"true"#.as_bytes();
        let mut tokens = vec![Token::True(vec![b't'])];
        let mut i = 1;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_true(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(tokens.last(), Some(&Token::None));
        assert_eq!(res.unwrap(), Some(JsonToken::Val(JsonValue::Bool(true))));
    }

    #[test]
    fn should_return_error() {
        let buf = r#"truE"#.as_bytes();
        let mut tokens = vec![Token::True(vec![b't'])];
        let mut i = 1;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_true(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(
            res,
            Err(Error {
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
        let buf = r#"false"#.as_bytes();
        let mut tokens = vec![Token::False(vec![b'f'])];
        let mut i = 1;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_false(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(tokens.last(), Some(&Token::None));
        assert_eq!(res.unwrap(), Some(JsonToken::Val(JsonValue::Bool(false))));
    }

    #[test]
    fn should_return_error() {
        let buf = r#"falz,"#.as_bytes();
        let mut tokens = vec![Token::False(vec![b'f'])];
        let mut i = 1;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_false(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(
            res,
            Err(Error {
                column: 3,
                code: ErrorCode::ExpectedFalse
            })
        );
    }
}
