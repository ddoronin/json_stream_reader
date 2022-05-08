use crate::constants::{DIGIT_CHAR_SET, EMPTY_CHAR_SET};
use crate::error::{Error, ErrorCode};
use crate::json_token::JsonToken;
use crate::token::*;

type Res = Result<Option<JsonToken>, Error>;

pub(crate) fn handle_arr(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Res {
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
        b']' => {
            handle_end_arr(tokens);
            Ok(Some(JsonToken::ArrEnd))
        }
        ch if EMPTY_CHAR_SET.contains(&ch) => Ok(None),
        ch if DIGIT_CHAR_SET.contains(&ch) => {
            tokens.push(Token::Number(vec![ch]));
            Ok(None)
        }
        _ => Err(Error {
            column: i,
            code: ErrorCode::InvalidArrFormat,
        }),
    }
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
mod handle_array_tests {
    use super::*;

    #[test]
    fn should_expect_null() {
        let mut tokens = vec![Token::Arr];

        let buf: &[u8] = "[null]".as_bytes();
        let i = 1;
        let res = handle_arr(buf, i, &mut tokens);

        assert_eq!(tokens.pop(), Some(Token::Null(vec![b'n'])));
        assert_eq!(res, Ok(None));
    }

    #[test]
    fn should_expect_true() {
        let mut tokens = vec![Token::Arr];

        let buf: &[u8] = "[true]".as_bytes();
        let i = 1;
        let res = handle_arr(buf, i, &mut tokens);

        assert_eq!(tokens.pop(), Some(Token::True(vec![b't'])));
        assert_eq!(res, Ok(None));
    }

    #[test]
    fn should_expect_false() {
        let mut tokens = vec![Token::Arr];

        let buf: &[u8] = "[false]".as_bytes();
        let i = 1;
        let res = handle_arr(buf, i, &mut tokens);

        assert_eq!(tokens.pop(), Some(Token::False(vec![b'f'])));
        assert_eq!(res, Ok(None));
    }

    #[test]
    fn should_expect_string() {
        let mut tokens = vec![Token::Arr];

        let buf: &[u8] = "[\"false\"]".as_bytes();
        let i = 1;
        let res = handle_arr(buf, i, &mut tokens);

        assert_eq!(tokens.pop(), Some(Token::String(vec![])));
        assert_eq!(res, Ok(None));
    }

    #[test]
    fn should_expect_num() {
        let mut tokens = vec![Token::Arr];

        let buf: &[u8] = "[42]".as_bytes();
        let i = 1;
        let res = handle_arr(buf, i, &mut tokens);

        assert_eq!(tokens.pop(), Some(Token::Number(vec![b'4'])));
        assert_eq!(res, Ok(None));
    }

    #[test]
    fn should_expect_neg_num() {
        let mut tokens = vec![Token::Arr];

        let buf: &[u8] = "[-42]".as_bytes();
        let i = 1;
        let res = handle_arr(buf, i, &mut tokens);

        assert_eq!(tokens.pop(), Some(Token::Number(vec![b'-'])));
        assert_eq!(res, Ok(None));
    }

    #[test]
    fn should_expect_arr() {
        let mut tokens = vec![Token::Arr];

        let buf: &[u8] = "[[42]]".as_bytes();
        let i = 1;
        let res = handle_arr(buf, i, &mut tokens);
        assert_eq!(tokens.pop(), Some(Token::Arr));
        assert_eq!(res, Ok(Some(JsonToken::ArrBeg)));
    }

    #[test]
    fn should_expect_obj() {
        let mut tokens = vec![Token::Arr];
        let buf: &[u8] = "[{}]".as_bytes();
        let i = 1;
        let res = handle_arr(buf, i, &mut tokens);

        assert_eq!(tokens.pop(), Some(Token::Obj));
        assert_eq!(res.unwrap(), Some(JsonToken::ObjBeg));
    }

    #[test]
    fn should_return_err() {
        let mut tokens = vec![Token::Arr];
        let buf: &[u8] = "[}".as_bytes();
        let i = 1;
        let res = handle_arr(buf, i, &mut tokens);

        assert_eq!(
            res,
            Err(Error {
                column: 1,
                code: ErrorCode::InvalidArrFormat
            })
        );
    }
}
