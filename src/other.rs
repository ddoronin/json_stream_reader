use crate::constants::{DIGIT_CHAR_SET, EMPTY_CHAR_SET};
use crate::error::{Error, ErrorCode};
use crate::json_token::JsonToken;
use crate::token::*;

type Res = Result<Option<JsonToken>, Error>;

pub(crate) fn handle_none(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Res {
    match buf[i] {
        b'{' => {
            tokens.push(Token::Obj);
            Ok(Some(JsonToken::ObjBeg))
        }
        b'[' => {
            tokens.push(Token::Arr);
            Ok(Some(JsonToken::ArrBeg))
        }
        _ => Err(Error {
            column: i,
            code: ErrorCode::ExpectedObjectOrArray,
        }),
    }
}

pub(crate) fn handle_colon(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Res {
    match buf[i] {
        b'n' => {
            tokens.push(Token::Null(vec![b'n']));
            Ok(None)
        }
        b't' => {
            tokens.push(Token::True(vec![b't']));
            Ok(None)
        }
        b'f' => {
            tokens.push(Token::False(vec![b'f']));
            Ok(None)
        }
        b'"' => {
            tokens.push(Token::String(vec![]));
            Ok(None)
        }
        b'{' => {
            tokens.push(Token::Obj);
            Ok(Some(JsonToken::ObjBeg))
        }
        b'[' => {
            tokens.push(Token::Arr);
            Ok(Some(JsonToken::ArrBeg))
        }
        ch if EMPTY_CHAR_SET.contains(&ch) => Ok(None),
        ch if DIGIT_CHAR_SET.contains(&ch) => {
            tokens.push(Token::Number(vec![ch]));
            Ok(None)
        }
        _ => Err(Error {
            column: i,
            code: ErrorCode::ExpectedColon,
        }),
    }
}

pub(crate) fn handle_nil_token(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Res {
    match buf[i] {
        ch if EMPTY_CHAR_SET.contains(&ch) => Ok(None),
        b',' => {
            tokens.push(Token::Comma);
            Ok(None)
        }
        b']' => {
            handle_end_arr(tokens);
            Ok(Some(JsonToken::ArrEnd))
        }
        b'}' => {
            handle_end_obj(tokens);
            Ok(Some(JsonToken::ObjEnd))
        }
        _ => Err(Error {
            column: i,
            code: ErrorCode::ExpectedCommaOrObjectEndOrArrayEnd,
        }),
    }
}

pub(crate) fn handle_comma(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Res {
    match buf[i] {
        ch if EMPTY_CHAR_SET.contains(&ch) => Ok(None),
        ch if DIGIT_CHAR_SET.contains(&ch) => {
            tokens.push(Token::Number(vec![ch]));
            Ok(None)
        }
        b'n' => {
            tokens.push(Token::Null(vec![b'n']));
            Ok(None)
        }
        b't' => {
            tokens.push(Token::True(vec![b't']));
            Ok(None)
        }
        b'f' => {
            tokens.push(Token::False(vec![b'f']));
            Ok(None)
        }
        b'"' => {
            let mut cursor = tokens.len();
            while cursor > 0 {
                match tokens[cursor - 1] {
                    Token::Arr => {
                        // ["foo"]
                        tokens.push(Token::String(vec![]));
                        break;
                    }
                    Token::Obj => {
                        // {"foo": "bar"}
                        tokens.push(Token::Key(vec![]));
                        break;
                    }
                    _ => cursor -= 1,
                }
            }
            Ok(None)
        }
        b'{' => {
            tokens.push(Token::Obj);
            Ok(Some(JsonToken::ObjBeg))
        }
        b'[' => {
            tokens.push(Token::Arr);
            Ok(Some(JsonToken::ArrBeg))
        }
        _ => Err(Error {
            column: i,
            code: ErrorCode::ExpectedAnyTerm,
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
        //         ^______ after colon
        //         01_2345_67890
        let mut tokens = vec![
            Token::Obj,
            Token::Key("foo".as_bytes().to_vec()),
            Token::AfterKey,
            Token::Colon,
        ];
        let mut i = 7;
        while i < buf.len() && tokens.last() == Some(&Token::Colon) {
            handle_colon(buf, i, &mut tokens);
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
            handle_colon(buf, i, &mut tokens);
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
            handle_colon(buf, i, &mut tokens);
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
            handle_colon(buf, i, &mut tokens);
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
            handle_colon(buf, i, &mut tokens);
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
        let mut res = Ok(None);
        while i < buf.len() && tokens.last() == Some(&Token::Colon) {
            res = handle_colon(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(tokens.pop(), Some(Token::Obj));
        assert_eq!(res, Ok(Some(JsonToken::ObjBeg)));
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
        let mut res = Ok(None);
        while i < buf.len() && tokens.last() == Some(&Token::Colon) {
            res = handle_colon(buf, i, &mut tokens);
            i += 1;
        }
        assert_eq!(tokens.pop(), Some(Token::Arr));
        assert_eq!(res, Ok(Some(JsonToken::ArrBeg)));
    }
}
