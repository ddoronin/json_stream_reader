use crate::constants::{DIGIT_CHAR_SET, EMPTY_CHAR_SET};
use crate::error::{Error, ErrorCode};
use crate::token::*;

pub(crate) fn handle_arr<F1, F2, F3>(
    buf: &[u8],
    i: usize,
    tokens: &mut Vec<Token>,
    mut on_obj_beg: F1,
    mut on_arr_beg: F2,
    mut on_arr_end: F3,
) -> Option<Error>
where
    F1: FnMut(),
    F2: FnMut(),
    F3: FnMut(),
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
        b']' => {
            handle_end_arr(tokens);
            on_arr_end();
        }
        ch if EMPTY_CHAR_SET.contains(&ch) => {}
        ch if DIGIT_CHAR_SET.contains(&ch) => tokens.push(Token::Number(vec![ch])),
        _ => {
            error = Some(Error {
                column: i,
                code: ErrorCode::InvalidArrFormat,
            })
        }
    }
    error
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
        let error = handle_arr(buf, i, &mut tokens, || {}, || {}, || {});

        assert_eq!(tokens.pop(), Some(Token::Null(vec![b'n'])));
        assert_eq!(error, None);
    }

    #[test]
    fn should_expect_true() {
        let mut tokens = vec![Token::Arr];

        let buf: &[u8] = "[true]".as_bytes();
        let i = 1;
        let error = handle_arr(buf, i, &mut tokens, || {}, || {}, || {});

        assert_eq!(tokens.pop(), Some(Token::True(vec![b't'])));
        assert_eq!(error, None);
    }

    #[test]
    fn should_expect_false() {
        let mut tokens = vec![Token::Arr];

        let buf: &[u8] = "[false]".as_bytes();
        let i = 1;
        let error = handle_arr(buf, i, &mut tokens, || {}, || {}, || {});

        assert_eq!(tokens.pop(), Some(Token::False(vec![b'f'])));
        assert_eq!(error, None);
    }

    #[test]
    fn should_expect_string() {
        let mut tokens = vec![Token::Arr];

        let buf: &[u8] = "[\"false\"]".as_bytes();
        let i = 1;
        let error = handle_arr(buf, i, &mut tokens, || {}, || {}, || {});

        assert_eq!(tokens.pop(), Some(Token::String(vec![])));
        assert_eq!(error, None);
    }

    #[test]
    fn should_expect_num() {
        let mut tokens = vec![Token::Arr];

        let buf: &[u8] = "[42]".as_bytes();
        let i = 1;
        let error = handle_arr(buf, i, &mut tokens, || {}, || {}, || {});

        assert_eq!(tokens.pop(), Some(Token::Number(vec![b'4'])));
        assert_eq!(error, None);
    }

    #[test]
    fn should_expect_neg_num() {
        let mut tokens = vec![Token::Arr];

        let buf: &[u8] = "[-42]".as_bytes();
        let i = 1;
        let error = handle_arr(buf, i, &mut tokens, || {}, || {}, || {});

        assert_eq!(tokens.pop(), Some(Token::Number(vec![b'-'])));
        assert_eq!(error, None);
    }

    #[test]
    fn should_expect_arr() {
        let mut tokens = vec![Token::Arr];

        let buf: &[u8] = "[[42]]".as_bytes();
        let i = 1;
        let mut arr = 0;
        let mut obj = 0;
        let error = handle_arr(
            buf,
            i,
            &mut tokens,
            || {
                obj += 1;
            },
            || {
                arr += 1;
            },
            || {},
        );

        assert_eq!(tokens.pop(), Some(Token::Arr));
        assert_eq!(arr, 1);
        assert_eq!(obj, 0);
        assert_eq!(error, None);
    }

    #[test]
    fn should_expect_obj() {
        let mut tokens = vec![Token::Arr];
        let buf: &[u8] = "[{}]".as_bytes();
        let i = 1;
        let mut arr = 0;
        let mut obj = 0;
        let error = handle_arr(
            buf,
            i,
            &mut tokens,
            || {
                obj += 1;
            },
            || {
                arr += 1;
            },
            || {},
        );

        assert_eq!(tokens.pop(), Some(Token::Obj));
        assert_eq!(arr, 0);
        assert_eq!(obj, 1);
        assert_eq!(error, None);
    }

    #[test]
    fn should_return_err() {
        let mut tokens = vec![Token::Arr];
        let buf: &[u8] = "[}".as_bytes();
        let i = 1;
        let error = handle_arr(buf, i, &mut tokens, || {}, || {}, || {});

        assert_eq!(
            error,
            Some(Error {
                column: 1,
                code: ErrorCode::InvalidArrFormat
            })
        );
    }
}
