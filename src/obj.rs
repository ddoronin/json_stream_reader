use std::collections::HashSet;
use std::str;

use crate::constants::{DIGIT_CHAR_SET, EMPTY_CHAR_SET};
use crate::error::{Error, ErrorCode};
use crate::token::*;
use crate::utils::squash;

const KEY_MAX_LEN: usize = 100;

pub(crate) fn handle_key<F>(
    buf: &[u8],
    i: usize,
    tokens: &mut Vec<Token>,
    mut on_key: F,
) -> Option<Error>
where
    F: FnMut(&str)
{
    let mut error: Option<Error> = None;
    match tokens.last_mut() {
        Some(Token::Key(ref mut data)) if data.len() > KEY_MAX_LEN =>
            error = Some(Error { column: i, code: ErrorCode::TooLongKey }),
        Some(Token::Key(ref mut data)) => match buf[i] {
            // escaped character \"
            b'"' if data.last() == Some(&b'\\') => data.push(b'"'),
            b'"' => {
                let key = str::from_utf8(&data).unwrap();
                on_key(key);
                tokens.push(Token::AfterKey);
            }
            ch => data.push(ch),
        },
        _ => error = Some(Error { column: i, code: ErrorCode::InvalidFormat })
    }
    error
}

pub(crate) fn handle_obj(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Option<Error> {
    let mut error: Option<Error> = None;
    match buf[i] {
        b'"' => tokens.push(Token::Key(vec![])),
        ch if EMPTY_CHAR_SET.contains(&ch) => {}
        _ => {
            error = Some(Error {
                column: i,
                code: ErrorCode::ExpectedKey,
            })
        }
    }
    error
}

pub(crate) fn handle_after_key(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Option<Error> {
    let mut error: Option<Error> = None;
    match tokens.last_mut() {
        Some(Token::AfterKey) =>match buf[i] {
            b':' => {
                tokens.pop();
                tokens.push(Token::Colon);
            }
            ch if EMPTY_CHAR_SET.contains(&ch) => {}
            _ => {
                error = Some(Error {
                    column: i,
                    code: ErrorCode::ExpectedColon,
                })
            }
        },
        _ => error = Some(Error { column: i, code: ErrorCode::InvalidFormat })
    }
    error
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
        while i < buf.len() {
            handle_key(buf, i, &mut tokens, |str: &str| {
                key = str.to_string();
            });
            i += 1;
        }

        assert_eq!(tokens.pop(), Some(Token::AfterKey));
        assert_eq!(tokens.pop(), Some(Token::Key("foo".as_bytes().to_vec())));
        assert_eq!(key, String::from("foo"));
    }

    #[test]
    fn should_parse_escaped_double_quote() {
        let buf: &[u8] = r#""foo\"bar""#.as_bytes();
        let mut tokens = vec![Token::Key(vec![])];
        let mut key = String::new();
        let mut i = 1;
        while i < buf.len() {
            handle_key(buf, i, &mut tokens, |str: &str| {
                key = str.to_string();
            });
            i += 1;
        }

        assert_eq!(tokens.pop(), Some(Token::AfterKey));
        assert_eq!(tokens.pop(), Some(Token::Key(r#"foo\"bar"#.as_bytes().to_vec())));
        assert_eq!(key, String::from(r#"foo\"bar"#));
    }

    #[test]
    fn should_return_too_long_key_error(){
        // data = "fooooooooo...oooo" The key is of KEY_MAX_LEN + 1 size.
        let mut data = vec![0 as u8; KEY_MAX_LEN + 3];
        let len = data.len();
        data.fill(b'o');
        data[0] = b'\"';
        data[1] = b'f';
        data[len - (1 as usize)] = b'\"';

        let mut buf: &[u8] = &data;
        let mut tokens = vec![Token::Key(vec![])];
        let mut key = String::new();
        let mut i = 1;
        let mut error = None;
        while i < buf.len() && error.is_none() {
            error = handle_key(buf, i, &mut tokens, |str: &str| {
                key = str.to_string();
            });
            i += 1;
        }
        assert_eq!(error, Some(Error { column: i - 1, code: ErrorCode::TooLongKey }))
    }

    #[test]
    fn should_return_invalid_format() {
        let mut buf: &[u8] = r#""foo""#.as_bytes();
        // Let's assign any token that is not Key.
        let mut tokens = vec![Token::None];
        let mut i = 1;
        let mut error = None;
        while i < buf.len() && error.is_none() {
            error = handle_key(buf, i, &mut tokens, |str: &str| {});
            i += 1;
        }
        assert_eq!(error, Some(Error { column: 1, code: ErrorCode::InvalidFormat }))
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
        let mut error = None;
        while i < buf.len() && error.is_none() {
            error = handle_obj(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(error, Some(Error{column: 5, code: ErrorCode::ExpectedKey}));
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
        let mut tokens = vec![Token::Obj, Token::Key("foo".as_bytes().to_vec()), Token::AfterKey];
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
        let mut tokens = vec![Token::Obj, Token::Key("foo".as_bytes().to_vec()), Token::AfterKey];
        let mut i = 6;
        let mut error = None;
        while i < buf.len() && tokens.last() == Some(&Token::AfterKey) && error.is_none() {
            error = handle_after_key(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(error, Some(Error{column: 7, code: ErrorCode::ExpectedColon}));
    }
}
