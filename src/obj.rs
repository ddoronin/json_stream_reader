use std::str;

use crate::constants::EMPTY_CHAR_SET;
use crate::error::{Error, ErrorCode};
use crate::json_token::JsonToken;
use crate::token::*;

const KEY_MAX_LEN: usize = 100;

type Res = Result<Option<JsonToken>, Error>;

pub(crate) fn handle_key(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Res {
    match tokens.last_mut() {
        Some(Token::Key(ref mut data)) if data.len() > KEY_MAX_LEN => Err(Error {
            column: i,
            code: ErrorCode::TooLongKey,
        }),
        Some(Token::Key(ref mut data)) => match buf[i] {
            // escaped character \"
            b'"' if data.last() == Some(&b'\\') => {
                data.push(b'"');
                Ok(None)
            }
            b'"' => match str::from_utf8(&data) {
                Ok(key) => {
                    let val = JsonToken::Key(key.to_string());
                    tokens.push(Token::AfterKey);
                    Ok(Some(val))
                }
                Err(err) => Err(Error {
                    column: i,
                    code: ErrorCode::InvalidFormat,
                }),
            },
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

pub(crate) fn handle_obj(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Res {
    match buf[i] {
        b'"' => {
            tokens.push(Token::Key(vec![]));
            Ok(None)
        }
        ch if EMPTY_CHAR_SET.contains(&ch) => Ok(None),
        b'}' => {
            handle_end_obj(tokens);
            Ok(Some(JsonToken::ObjEnd))
        }
        _ => Err(Error {
            column: i,
            code: ErrorCode::ExpectedKey,
        }),
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

pub(crate) fn handle_after_key(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Res {
    match tokens.last_mut() {
        Some(Token::AfterKey) => match buf[i] {
            b':' => {
                tokens.pop();
                tokens.push(Token::Colon);
                Ok(None)
            }
            ch if EMPTY_CHAR_SET.contains(&ch) => Ok(None),
            _ => Err(Error {
                column: i,
                code: ErrorCode::ExpectedColon,
            }),
        },
        _ => Err(Error {
            column: i,
            code: ErrorCode::InvalidFormat,
        }),
    }
}

#[cfg(test)]
mod test_handle_key {
    use super::*;

    #[test]
    fn should_parse_simple_key() {
        let buf: &[u8] = "\"foo\"".as_bytes();
        let mut tokens = vec![Token::Key(vec![])];
        let mut key = String::new();
        let mut i = 1;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_key(buf, i, &mut tokens);
            i += 1;
        }

        assert_eq!(tokens.pop(), Some(Token::AfterKey));
        assert_eq!(tokens.pop(), Some(Token::Key("foo".as_bytes().to_vec())));
        assert_eq!(res.unwrap(), Some(JsonToken::Key("foo".to_string())));
    }

    #[test]
    fn should_parse_escaped_double_quote() {
        let buf: &[u8] = r#""foo\"bar""#.as_bytes();
        let mut tokens = vec![Token::Key(vec![])];
        let mut key = String::new();
        let mut i = 1;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_key(buf, i, &mut tokens);
            i += 1;
        }

        assert_eq!(tokens.pop(), Some(Token::AfterKey));
        assert_eq!(
            tokens.pop(),
            Some(Token::Key(r#"foo\"bar"#.as_bytes().to_vec()))
        );
        assert_eq!(
            res.unwrap(),
            Some(JsonToken::Key(r#"foo\"bar"#.to_string()))
        );
    }

    #[test]
    fn should_return_too_long_key_error() {
        // data = "fooooooooo...oooo" The key is of KEY_MAX_LEN + 1 size.
        let mut data = vec![0 as u8; KEY_MAX_LEN + 3];
        let len = data.len();
        data.fill(b'o');
        data[0] = b'\"';
        data[1] = b'f';
        data[len - (1 as usize)] = b'\"';

        let buf: &[u8] = &data;
        let mut tokens = vec![Token::Key(vec![])];
        let mut key = String::new();
        let mut i = 1;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_key(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(
            res,
            Err(Error {
                column: i - 1,
                code: ErrorCode::TooLongKey
            })
        )
    }

    #[test]
    fn should_return_invalid_format() {
        let buf: &[u8] = r#""foo""#.as_bytes();
        // Let's assign any token that is not Key.
        let mut tokens = vec![Token::None];
        let mut i = 1;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_key(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(
            res,
            Err(Error {
                column: 1,
                code: ErrorCode::InvalidFormat
            })
        )
    }
}

#[cfg(test)]
mod obj_tests {
    use super::*;

    #[test]
    fn should_add_key_token() {
        let buf = "{   \"foo\": \"bar\"  }".as_bytes();
        let mut tokens = vec![Token::Obj];
        let mut i = 1;
        while i < buf.len() && tokens.last() == Some(&Token::Obj) {
            handle_obj(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(tokens.pop(), Some(Token::Key(vec![])));
    }

    #[test]
    fn should_return_error() {
        let buf = "{    foo  \"foo\": \"bar\"  }".as_bytes();
        //                    | invalid char
        //               0123456789012345678901234567890

        let mut tokens = vec![Token::Obj];
        let mut i = 1;
        let mut res = Ok(None);
        while i < buf.len() && res.is_ok() {
            res = handle_obj(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(
            res,
            Err(Error {
                column: 5,
                code: ErrorCode::ExpectedKey
            })
        );
    }
}

#[cfg(test)]
mod obj_after_key {
    use super::*;

    #[test]
    fn should_expect_colon() {
        let buf = "{\"foo\" : \"bar\"  }".as_bytes();
        //                        ^______ colon
        //               01_2345_67890
        let mut tokens = vec![
            Token::Obj,
            Token::Key("foo".as_bytes().to_vec()),
            Token::AfterKey,
        ];
        let mut i = 6;
        while i < buf.len() && tokens.last() == Some(&Token::AfterKey) {
            handle_after_key(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(tokens.pop(), Some(Token::Colon));
    }

    #[test]
    fn should_return_exptected_colon_error() {
        let buf = "{\"foo\" ? \"bar\"  }".as_bytes();
        //                        ^______ colon
        //               01_2345_67890
        let mut tokens = vec![
            Token::Obj,
            Token::Key("foo".as_bytes().to_vec()),
            Token::AfterKey,
        ];
        let mut i = 6;
        let mut res = Ok(None);
        while i < buf.len() && tokens.last() == Some(&Token::AfterKey) && res.is_ok() {
            res = handle_after_key(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(
            res,
            Err(Error {
                column: 7,
                code: ErrorCode::ExpectedColon
            })
        );
    }
}
