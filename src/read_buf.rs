// //! Helper functions that read buffer and create tokens.
// #[allow(unused_doc_comments)]
// use std::collections::HashSet;
// use std::str;
//
// use crate::constants::{DIGIT_CHAR_SET, EMPTY_CHAR_SET};
// use crate::error::{Error, ErrorCode};
// use crate::token::*;
// use crate::utils::squash;
//
// /// Reads a buffer of a given size starting from an index `'i'`.
// /// This function receives a mutable stack of tokens that is used to store the internal state
// /// between consecutive buffer reads.
// ///
// /// Returns an error if the json format is invalid.
// pub(crate) fn read_buf(
//     buf: &[u8],
//     size: usize,
//     mut i: usize,
//     tokens: &mut Vec<Token>,
//     on_obj_beg: fn() -> (),
//     on_obj_end: fn() -> (),
//     on_arr_beg: fn() -> (),
//     on_arr_end: fn() -> (),
//     on_key: fn(key: &str) -> (),
//     on_val: fn(val: &str) -> (),
// ) -> Option<Error> {
//     let mut error: Option<Error> = None;
//     while i < size && error.is_none() {
//         let token = tokens.last();
//         error = match token {
//             None => handle_none(&buf, i, tokens, on_obj_beg, on_arr_beg),
//             Some(Token::Obj) => handle_obj(buf, i, tokens),
//             Some(Token::Key(_)) => handle_key(&buf, i, tokens, on_key),
//             Some(Token::AfterKey) => handle_after_key(&buf, i, tokens),
//             Some(Token::Colon) => handle_colon(&buf, size, i, tokens, on_obj_beg, on_arr_beg),
//             Some(Token::String(_)) => handle_string(&buf, i, tokens, on_val),
//             Some(Token::Number(_)) => handle_number(&buf, i, tokens, on_val, on_arr_end, on_obj_end),
//             Some(Token::Null(_)) => handle_null(&buf, i, tokens, on_val),
//             Some(Token::True(_)) => handle_true(&buf, i, tokens, on_val),
//             Some(Token::False(_)) => handle_false(&buf, i, tokens, on_val),
//             Some(Token::Arr) => handle_array(&buf, i, tokens, on_obj_beg, on_arr_beg),
//             Some(Token::Comma) => handle_comma(&buf, i, tokens, on_obj_beg, on_arr_beg),
//             Some(Token::None) => handle_nil_token(&buf, i, tokens, on_obj_end, on_arr_end),
//         };
//         i += 1;
//     }
//     error
// }
//
// fn handle_end_obj(tokens: &mut Vec<Token>) {
//     while let Some(token) = tokens.pop() {
//         if let Token::Obj = token {
//             break;
//         }
//     }
//     tokens.push(Token::None);
// }
//
// fn handle_end_arr(tokens: &mut Vec<Token>) {
//     while let Some(token) = tokens.pop() {
//         if let Token::Arr = token {
//             break;
//         }
//     }
//     tokens.push(Token::None);
// }
//
// fn handle_obj(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Option<Error> {
//     let mut error: Option<Error> = None;
//     match buf[i] {
//         ch if EMPTY_CHAR_SET.contains(&ch) => {}
//         b'"' => tokens.push(Token::Key(vec![])),
//         _ => {
//             error = Some(Error {
//                 column: i,
//                 code: ErrorCode::ExpectedKey,
//             })
//         }
//     }
//     error
// }
//
// fn handle_key(
//     buf: &[u8],
//     i: usize,
//     tokens: &mut Vec<Token>,
//     on_key: fn(&str) -> (),
// ) -> Option<Error> {
//     let mut error: Option<Error> = None;
//     if let Some(Token::Key(ref mut data)) = tokens.last_mut() {
//         match buf[i] {
//             // escaped character \"
//             b'"' if data.last() == Some(&b'\\') => data.push(b'"'),
//             b'"' => {
//                 let key = str::from_utf8(&data).unwrap();
//                 on_key(key);
//                 tokens.push(Token::AfterKey);
//             }
//             ch => data.push(ch),
//         }
//     }
//     // if key is too long return an error.
//     error
// }
//
// fn handle_after_key(buf: &[u8], i: usize, tokens: &mut Vec<Token>) -> Option<Error> {
//     let mut error: Option<Error> = None;
//     if let Some(Token::AfterKey) = tokens.last_mut() {
//         match buf[i] {
//             b':' => {
//                 tokens.pop();
//                 tokens.push(Token::Colon);
//             }
//             ch if EMPTY_CHAR_SET.contains(&ch) => {}
//             _ => {
//                 error = Some(Error {
//                     column: i,
//                     code: ErrorCode::ExpectedColon,
//                 })
//             }
//         }
//     }
//     error
// }
//
// fn handle_none(
//     buf: &[u8],
//     i: usize,
//     tokens: &mut Vec<Token>,
//     on_obj_beg: fn() -> (),
//     on_arr_beg: fn() -> (),
// ) -> Option<Error> {
//     let mut error: Option<Error> = None;
//     match buf[i] {
//         b'{' => {
//             on_obj_beg();
//             tokens.push(Token::Obj);
//         }
//         b'[' => {
//             on_arr_beg();
//             tokens.push(Token::Arr);
//         }
//         _ => {
//             error = Some(Error {
//                 column: i,
//                 code: ErrorCode::ExpectedObjectOrArray,
//             })
//         }
//     }
//     error
// }
//
// fn handle_colon(
//     buf: &[u8],
//     i: usize,
//     tokens: &mut Vec<Token>,
//     on_obj_beg: fn() -> (),
//     on_arr_beg: fn() -> (),
// ) -> Option<Error> {
//     let mut error: Option<Error> = None;
//     match buf[i] {
//         ch if EMPTY_CHAR_SET.contains(&ch) => {}
//         ch if DIGIT_CHAR_SET.contains(&ch) => tokens.push(Token::Number(vec![ch])),
//         b'n' => tokens.push(Token::Null(vec![b'n'])),
//         b't' => tokens.push(Token::True(vec![b't'])),
//         b'f' => tokens.push(Token::False(vec![b'f'])),
//         b'"' => tokens.push(Token::String(vec![])),
//         b'{' => {
//             tokens.push(Token::Obj);
//             on_obj_beg();
//         }
//         b'[' => {
//             tokens.push(Token::Arr);
//             on_arr_beg();
//         }
//         _ => {
//             error = Some(Error {
//                 column: i,
//                 code: ErrorCode::ExpectedColon,
//             })
//         }
//     }
//     error
// }
//
// fn handle_string(
//     buf: &[u8],
//     i: usize,
//     tokens: &mut Vec<Token>,
//     on_val: fn(val: &str) -> (),
// ) -> Option<Error> {
//     let mut error: Option<Error> = None;
//     if let Some(Token::String(ref mut data)) = tokens.last_mut() {
//         match buf[i] {
//             b'"' if data.last() == Some(&b'\\') => data.push(b'"'),
//             b'"' => {
//                 let val = str::from_utf8(&data).unwrap();
//                 on_val(val);
//                 squash(tokens);
//             }
//             s => data.push(s),
//             _ => {
//                 error = Some(Error {
//                     column: i,
//                     code: ErrorCode::ExpectedString,
//                 })
//             }
//         }
//     }
//     error
// }
//
// fn handle_number(
//     buf: &[u8],
//     i: usize,
//     tokens: &mut Vec<Token>,
//     on_val: fn(val: &str) -> (),
//     on_arr_end: fn() -> (),
//     on_obj_end: fn() -> (),
// ) -> Option<Error> {
//     let mut error: Option<Error> = None;
//     if let Some(Token::Number(ref mut data)) = tokens.last_mut() {
//         match buf[i] {
//             ch if DIGIT_CHAR_SET.contains(&ch) => data.push(ch),
//             b'.' if !data.contains(&b'.') => data.push(b'.'),
//             b',' => {
//                 let val = str::from_utf8(&data).unwrap();
//                 on_val(val);
//                 squash(tokens);
//                 tokens.push(Token::Comma);
//             }
//             b']' => {
//                 let val = str::from_utf8(&data).unwrap();
//                 on_val(val);
//                 squash(tokens);
//                 handle_end_arr(tokens);
//                 on_arr_end();
//             }
//             b'}' => {
//                 let val = str::from_utf8(&data).unwrap();
//                 on_val(val);
//                 squash(tokens);
//                 handle_end_obj(tokens);
//                 on_obj_end();
//             }
//             ch if EMPTY_CHAR_SET.contains(&ch) => {
//                 let val = str::from_utf8(&data).unwrap();
//                 on_val(val);
//                 squash(tokens);
//             }
//             _ => {
//                 error = Some(Error {
//                     column: i,
//                     code: ErrorCode::InvalidNumber,
//                 })
//             }
//         }
//     }
//     error
// }
//
// fn handle_null(
//     buf: &[u8],
//     i: usize,
//     tokens: &mut Vec<Token>,
//     on_val: fn(val: &str) -> (),
// ) -> Option<Error> {
//     let mut error: Option<Error> = None;
//     if let Some(Token::Null(ref mut data)) = tokens.last_mut() {
//         match buf[i] {
//             b'u' if *data == [b'n'] => data.push(b'u'),
//             b'l' if *data == [b'n', b'u'] => data.push(b'l'),
//             b'l' if *data == [b'n', b'u', b'l'] => {
//                 data.push(b'l');
//                 let val = str::from_utf8(&data).unwrap();
//                 on_val(val);
//                 squash(tokens);
//             }
//             _ => {
//                 error = Some(Error {
//                     column: i,
//                     code: ErrorCode::ExpectedNull,
//                 })
//             }
//         }
//     }
//     error
// }
//
// fn handle_true(
//     buf: &[u8],
//     i: usize,
//     tokens: &mut Vec<Token>,
//     on_val: fn(val: &str) -> (),
// ) -> Option<Error> {
//     let mut error: Option<Error> = None;
//     if let Some(Token::True(ref mut data)) = tokens.last_mut() {
//         match buf[i] {
//             b'r' if *data == [b't'] => data.push(b'r'),
//             b'u' if *data == [b't', b'r'] => data.push(b'u'),
//             b'e' if *data == [b't', b'r', b'u'] => {
//                 data.push(b'e');
//                 let val = str::from_utf8(&data).unwrap();
//                 on_val(val);
//                 squash(tokens);
//             }
//             _ => {
//                 error = Some(Error {
//                     column: i,
//                     code: ErrorCode::ExpectedFalse,
//                 })
//             }
//         }
//     }
//     error
// }
//
// fn handle_false(
//     buf: &[u8],
//     i: usize,
//     tokens: &mut Vec<Token>,
//     on_val: fn(val: &str) -> (),
// ) -> Option<Error> {
//     let mut error: Option<Error> = None;
//     if let Some(Token::False(ref mut data)) = tokens.last_mut() {
//         match buf[i] {
//             b'a' if *data == [b'f'] => data.push(b'a'),
//             b'l' if *data == [b'f', b'a'] => data.push(b'l'),
//             b's' if *data == [b'f', b'a', b'l'] => data.push(b's'),
//             b'e' if *data == [b'f', b'a', b'l', b's'] => {
//                 data.push(b'e');
//                 let val = str::from_utf8(&data).unwrap();
//                 on_val(val);
//                 squash(tokens);
//             }
//             _ => {
//                 error = Some(Error {
//                     column: i,
//                     code: ErrorCode::ExpectedFalse,
//                 })
//             }
//         }
//     }
//     error
// }
//
// fn handle_nil_token(
//     buf: &[u8],
//     i: usize,
//     tokens: &mut Vec<Token>,
//     on_obj_end: fn() -> (),
//     on_arr_end: fn() -> (),
// ) -> Option<Error> {
//     let mut error: Option<Error> = None;
//     match buf[i] {
//         ch if EMPTY_CHAR_SET.contains(&ch) => {}
//         b',' => tokens.push(Token::Comma),
//         b']' => {
//             handle_end_arr(tokens);
//             on_arr_end();
//         }
//         b'}' => {
//             handle_end_obj(tokens);
//             on_obj_end();
//         }
//         _ => {
//             error = Some(Error {
//                 column: i,
//                 code: ErrorCode::ExpectedCommaOrObjectEndOrArrayEnd,
//             })
//         }
//     }
//     error
// }
//
// fn handle_comma(
//     buf: &[u8],
//     i: usize,
//     tokens: &mut Vec<Token>,
//     on_obj_beg: fn() -> (),
//     on_arr_beg: fn() -> (),
// ) -> Option<Error> {
//     let mut error: Option<Error> = None;
//     match buf[i] {
//         ch if EMPTY_CHAR_SET.contains(&ch) => {}
//         ch if DIGIT_CHAR_SET.contains(&ch) => tokens.push(Token::Number(vec![ch])),
//         b'n' => tokens.push(Token::Null(vec![b'n'])),
//         b't' => tokens.push(Token::True(vec![b't'])),
//         b'f' => tokens.push(Token::False(vec![b'f'])),
//         b'"' => {
//             let mut cursor = tokens.len();
//             while cursor > 0 {
//                 match tokens[cursor - 1] {
//                     Token::Arr => {
//                         tokens.push(Token::String(vec![]));
//                         break;
//                     }
//                     Token::Obj => {
//                         tokens.push(Token::Key(vec![]));
//                         break;
//                     }
//                     _ => cursor -= 1,
//                 }
//             }
//         }
//         b'{' => {
//             tokens.push(Token::Obj);
//             on_obj_beg();
//         }
//         b'[' => {
//             tokens.push(Token::Arr);
//             on_arr_beg();
//         }
//         _ => {
//             error = Some(Error {
//                 column: i,
//                 code: ErrorCode::ExpectedAnyTerm,
//             })
//         }
//     }
//     error
// }
//
// fn handle_array(
//     buf: &[u8],
//     i: usize,
//     tokens: &mut Vec<Token>,
//     on_obj_beg: fn() -> (),
//     on_arr_beg: fn() -> (),
// ) -> Option<Error> {
//     let mut error: Option<Error> = None;
//     match buf[i] {
//         ch if EMPTY_CHAR_SET.contains(&ch) => {}
//         ch if DIGIT_CHAR_SET.contains(&ch) => tokens.push(Token::Number(vec![ch])),
//         b'n' => tokens.push(Token::Null(vec![b'n'])),
//         b't' => tokens.push(Token::True(vec![b't'])),
//         b'f' => tokens.push(Token::False(vec![b'f'])),
//         b'"' => tokens.push(Token::String(vec![])),
//         b'{' => {
//             tokens.push(Token::Obj);
//             on_obj_beg();
//         }
//         b'[' => {
//             tokens.push(Token::Arr);
//             on_arr_beg();
//         }
//         _ => {
//             error = Some(Error {
//                 column: i,
//                 code: ErrorCode::ExpectedAnyTerm,
//             })
//         }
//     }
//     error
// }
//
// // #[cfg(test)]
// // mod test {
// //     use super::*;
// //
// //     #[test]
// //     fn test_handle_first_char() {
// //
// //     }
// // }
