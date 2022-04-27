use crate::constants::{DIGIT_CHAR_SET, EMPTY_CHAR_SET};
use crate::error::{Error, ErrorCode};
use crate::token::*;

pub(crate) fn handle_none<O, A>(
    buf: &[u8],
    i: usize,
    tokens: &mut Vec<Token>,
    mut on_obj_beg: O,
    mut on_arr_beg: A,
) -> Option<Error>
where
    O: FnMut(),
    A: FnMut(),
{
    let mut error: Option<Error> = None;
    match buf[i] {
        b'{' => {
            on_obj_beg();
            tokens.push(Token::Obj);
        }
        b'[' => {
            on_arr_beg();
            tokens.push(Token::Arr);
        }
        _ => {
            error = Some(Error {
                column: i,
                code: ErrorCode::ExpectedObjectOrArray,
            })
        }
    }
    error
}

pub(crate) fn handle_colon<F1, F2>(
    buf: &[u8],
    i: usize,
    tokens: &mut Vec<Token>,
    mut on_obj_beg: F1,
    mut on_arr_beg: F2,
) -> Option<Error>
where
    F1: FnMut(),
    F2: FnMut(),
{
    let mut error: Option<Error> = None;
    match buf[i] {
        b'n' => tokens.push(Token::Null(vec![b'n'])),
        b't' => tokens.push(Token::True(vec![b't'])),
        b'f' => tokens.push(Token::False(vec![b'f'])),
        b'"' => tokens.push(Token::String(vec![])),
        b'{' => {
            tokens.push(Token::Obj);
            on_obj_beg();
        }
        b'[' => {
            tokens.push(Token::Arr);
            on_arr_beg();
        }
        ch if EMPTY_CHAR_SET.contains(&ch) => {}
        ch if DIGIT_CHAR_SET.contains(&ch) => tokens.push(Token::Number(vec![ch])),
        _ => {
            error = Some(Error {
                column: i,
                code: ErrorCode::ExpectedColon,
            })
        }
    }
    error
}

pub(crate) fn handle_nil_token<O, A>(
    buf: &[u8],
    i: usize,
    tokens: &mut Vec<Token>,
    mut on_obj_end: O,
    mut on_arr_end: A,
) -> Option<Error>
where
    O: FnMut(),
    A: FnMut(),
{
    let mut error: Option<Error> = None;
    match buf[i] {
        ch if EMPTY_CHAR_SET.contains(&ch) => {}
        b',' => tokens.push(Token::Comma),
        b']' => {
            handle_end_arr(tokens);
            on_arr_end();
        }
        b'}' => {
            handle_end_obj(tokens);
            on_obj_end();
        }
        _ => {
            error = Some(Error {
                column: i,
                code: ErrorCode::ExpectedCommaOrObjectEndOrArrayEnd,
            })
        }
    }
    error
}

pub(crate) fn handle_comma<O, A>(
    buf: &[u8],
    i: usize,
    tokens: &mut Vec<Token>,
    mut on_obj_beg: O,
    mut on_arr_beg: A,
) -> Option<Error>
where
    O: FnMut(),
    A: FnMut(),
{
    let mut error: Option<Error> = None;
    match buf[i] {
        ch if EMPTY_CHAR_SET.contains(&ch) => {}
        ch if DIGIT_CHAR_SET.contains(&ch) => tokens.push(Token::Number(vec![ch])),
        b'n' => tokens.push(Token::Null(vec![b'n'])),
        b't' => tokens.push(Token::True(vec![b't'])),
        b'f' => tokens.push(Token::False(vec![b'f'])),
        b'"' => {
            let mut cursor = tokens.len();
            while cursor > 0 {
                match tokens[cursor - 1] {
                    Token::Arr => {
                        tokens.push(Token::String(vec![]));
                        break;
                    }
                    Token::Obj => {
                        tokens.push(Token::Key(vec![]));
                        break;
                    }
                    _ => cursor -= 1,
                }
            }
        }
        b'{' => {
            tokens.push(Token::Obj);
            on_obj_beg();
        }
        b'[' => {
            tokens.push(Token::Arr);
            on_arr_beg();
        }
        _ => {
            error = Some(Error {
                column: i,
                code: ErrorCode::ExpectedAnyTerm,
            })
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
mod colon {
    use super::*;

    #[test]
    fn should_expect_null() {
        let buf = "{\"foo\": null}".as_bytes();
        //                        ^______ after colon
        //               01_2345_67890
        let mut tokens = vec![
            Token::Obj,
            Token::Key("foo".as_bytes().to_vec()),
            Token::AfterKey,
            Token::Colon,
        ];
        let mut i = 7;
        while i < buf.len() && tokens.last() == Some(&Token::Colon) {
            handle_colon(buf, i, &mut tokens, || {}, || {});
            i += 1;
        }
        assert_eq!(tokens.pop(), Some(Token::Null("n".as_bytes().to_vec())));
    }

    #[test]
    fn should_expect_true() {
        let buf = "{\"foo\": true}".as_bytes();
        //                        ^______ after colon
        //               01_2345_67890
        let mut tokens = vec![
            Token::Obj,
            Token::Key("foo".as_bytes().to_vec()),
            Token::AfterKey,
            Token::Colon,
        ];
        let mut i = 7;
        while i < buf.len() && tokens.last() == Some(&Token::Colon) {
            handle_colon(buf, i, &mut tokens, || {}, || {});
            i += 1;
        }
        assert_eq!(tokens.pop(), Some(Token::True("t".as_bytes().to_vec())));
    }

    #[test]
    fn should_expect_false() {
        let buf = "{\"foo\": false}".as_bytes();
        //                        ^______ after colon
        //               01_2345_67890
        let mut tokens = vec![
            Token::Obj,
            Token::Key("foo".as_bytes().to_vec()),
            Token::AfterKey,
            Token::Colon,
        ];
        let mut i = 7;
        while i < buf.len() && tokens.last() == Some(&Token::Colon) {
            handle_colon(buf, i, &mut tokens, || {}, || {});
            i += 1;
        }
        assert_eq!(tokens.pop(), Some(Token::False("f".as_bytes().to_vec())));
    }

    #[test]
    fn should_expect_string() {
        let buf = "{\"foo\": \"bar\"}".as_bytes();
        //                        ^______ after colon
        //               01_2345_67890
        let mut tokens = vec![
            Token::Obj,
            Token::Key("foo".as_bytes().to_vec()),
            Token::AfterKey,
            Token::Colon,
        ];
        let mut i = 7;
        while i < buf.len() && tokens.last() == Some(&Token::Colon) {
            handle_colon(buf, i, &mut tokens, || {}, || {});
            i += 1;
        }
        assert_eq!(tokens.pop(), Some(Token::String(vec![])));
    }

    #[test]
    fn should_expect_number() {
        let buf = "{\"foo\": 42}".as_bytes();
        //                        ^______ after colon
        //               01_2345_67890
        let mut tokens = vec![
            Token::Obj,
            Token::Key("foo".as_bytes().to_vec()),
            Token::AfterKey,
            Token::Colon,
        ];
        let mut i = 7;
        while i < buf.len() && tokens.last() == Some(&Token::Colon) {
            handle_colon(buf, i, &mut tokens, || {}, || {});
            i += 1;
        }
        assert_eq!(tokens.pop(), Some(Token::Number("4".as_bytes().to_vec())));
    }

    #[test]
    fn should_expect_obj() {
        let buf = "{\"foo\": {\"bar\": \"zar\"}}".as_bytes();
        //                        ^______ after colon
        //               01_2345_67890
        let mut tokens = vec![
            Token::Obj,
            Token::Key("foo".as_bytes().to_vec()),
            Token::AfterKey,
            Token::Colon,
        ];
        let mut i = 7;
        let mut is_obj = false;
        let mut is_arr = false;
        while i < buf.len() && tokens.last() == Some(&Token::Colon) {
            handle_colon(buf, i, &mut tokens, || is_obj = true, || is_arr = true);
            i += 1;
        }
        assert_eq!(tokens.pop(), Some(Token::Obj));
        assert_eq!(is_obj, true);
        assert_eq!(is_arr, false);
    }

    #[test]
    fn should_expect_arr() {
        let buf = "{\"foo\": [\"bar\", \"zar\"]}".as_bytes();
        //                        ^______ after colon
        //               01_2345_67890
        let mut tokens = vec![
            Token::Obj,
            Token::Key("foo".as_bytes().to_vec()),
            Token::AfterKey,
            Token::Colon,
        ];
        let mut i = 7;
        let mut is_obj = false;
        let mut is_arr = false;
        while i < buf.len() && tokens.last() == Some(&Token::Colon) {
            handle_colon(buf, i, &mut tokens, || is_obj = true, || is_arr = true);
            i += 1;
        }
        assert_eq!(tokens.pop(), Some(Token::Arr));
        assert_eq!(is_obj, false);
        assert_eq!(is_arr, true);
    }
}
