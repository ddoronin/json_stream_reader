#[allow(unused_doc_comments)]
use std::collections::HashSet;
use std::str;

use crate::token::*;
use crate::constants::{DIGIT_CHAR_SET, EMPTY_CHAR_SET};
use crate::error::{Error, ErrorCode, Result};
use crate::token::*;
use crate::utils::squash;
use crate::obj::*;
use crate::arr::*;
use crate::val::*;
use crate::other::*;

#[derive(Debug)]
pub struct JsonStreamReader {
    // internal state needed for buffering
    state: Vec<Token>,
}

impl JsonStreamReader {
    pub fn new() -> Self {
        JsonStreamReader { state: vec![] }
    }

    // Clears the internal state of the reader.
    pub fn clear(&mut self) -> &Self {
        self.state.clear();
        self
    }

    /// Reads buffer from a given start index to the end.
    /// When a json element is parsed a function callback will be called.
    ///
    /// # Arguments
    ///
    /// * `buf`:
    /// * `i`: start index
    /// * `on_obj_beg`:
    /// * `on_obj_end`:
    /// * `on_arr_beg`:
    /// * `on_arr_end`:
    /// * `on_key`:
    /// * `on_val`:
    ///
    /// returns: Result<&JsonReader, String>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    pub fn read<A, B, C, D, E, F>(
        &mut self,
        buf: &[u8],
        mut i: usize,
        mut on_obj_beg: A,
        mut on_obj_end: B,
        mut on_arr_beg: C,
        mut on_arr_end: D,
        mut on_key: E,
        mut on_val: F,
    ) -> Result<&Self> where A: FnMut(), B: FnMut(), C: FnMut(), D: FnMut(), E: FnMut(&str), F: FnMut(&str){
        let mut tokens = &mut self.state;
        let mut error: Option<Error> = None;
        let size = buf.len();
        while i < size && error.is_none() {
            let token = tokens.last();
            error = match token {
                None => handle_none(&buf, i, tokens, &mut on_obj_beg, &mut on_arr_beg),
                Some(Token::Obj) => handle_obj(buf, i, tokens),
                Some(Token::Key(_)) => handle_key(&buf, i, tokens, &mut on_key),
                Some(Token::AfterKey) => handle_after_key(&buf, i, tokens),
                Some(Token::Colon) => handle_colon(&buf, i, tokens, &mut on_obj_beg, &mut on_arr_beg),
                Some(Token::String(_)) => handle_string(&buf, i, tokens, &mut on_val),
                Some(Token::Number(_)) => handle_number(&buf, i, tokens, &mut on_val, &mut on_arr_end, &mut on_obj_end),
                Some(Token::Null(_)) => handle_null(&buf, i, tokens, &mut on_val),
                Some(Token::True(_)) => handle_true(&buf, i, tokens, &mut on_val),
                Some(Token::False(_)) => handle_false(&buf, i, tokens, &mut on_val),
                Some(Token::Arr) => handle_arr(&buf, i, tokens, &mut on_obj_beg, &mut on_arr_beg),
                Some(Token::Comma) => handle_comma(&buf, i, tokens, &mut on_obj_beg, &mut on_arr_beg),
                Some(Token::None) => handle_nil_token(&buf, i, tokens, &mut on_obj_end, &mut on_arr_end),
            };
            i += 1;
        }
        match error {
            Some(error) => Err(error),
            None => Ok(self),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn test_read() {
        let buf = "{\"foo1\": \"bar1\", \"foo2\": \"bar2\", \"foo3\": { \"foo4\": \"bar4\" }, \"foo5\": [ \"bar5\", \"bar6\" ] }".as_bytes();

        let mut obj = vec![];
        let r = RefCell::new(obj);
        let mut reader = JsonStreamReader::new();
        let result = reader.read(
            buf,
            0,
            || {
                r.borrow_mut().push("{".to_string());
            },
            || {
                r.borrow_mut().push("}".to_string());
            },
            || {
                r.borrow_mut().push("[".to_string());
            },
            || {
                r.borrow_mut().push("]".to_string());
            },
            |obj_key: &str| {
                r.borrow_mut().push(format!("key: {:}", obj_key.to_string()));
            },
            |obj_val: &str| {
                r.borrow_mut().push(format!("val: {:}", obj_val.to_string()));
            },
        );

        assert_eq!(result.is_ok(), true);
        assert_eq!(*r.borrow(), vec![
            "{",
            "key: foo1",
            "val: bar1",

            "key: foo2",
            "val: bar2",

            "key: foo3",
            "{",

            "key: foo4",
            "val: bar4",

            "}",

            "key: foo5",

            "[",
            "val: bar5",
            "val: bar6",
            "]",

            "}"
        ]);
    }
}
