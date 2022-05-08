use std::str;

use crate::arr::*;
use crate::error::{Error, Result};
use crate::json_token::JsonToken;
use crate::json_value::JsonValue;
use crate::obj::*;
use crate::other::*;
use crate::token::*;
use crate::val::*;

#[derive(Debug)]
pub struct JsonStreamReader {
    // internal state needed for buffering
    state: Vec<Token>,
}

type Res = Result<Option<JsonToken>>;

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
    pub fn read(&mut self, buf: &[u8]) -> Result<Vec<JsonToken>> {
        let mut json_tokens = vec![];
        let tokens = &mut self.state;
        let size = buf.len();
        let mut i = 0;
        let mut res = Ok(None);
        let mut error = None;
        while i < size && error.is_none() {
            let token = tokens.last();
            res = match token {
                None => handle_none(&buf, i, tokens),
                Some(Token::Obj) => handle_obj(buf, i, tokens),
                Some(Token::Key(_)) => handle_key(&buf, i, tokens),
                Some(Token::AfterKey) => handle_after_key(&buf, i, tokens),
                Some(Token::Colon) => handle_colon(&buf, i, tokens),
                Some(Token::String(_)) => handle_string(&buf, i, tokens),
                Some(Token::Number(_)) => match handle_number(&buf, i, tokens) {
                    Ok(Some((number_token, None))) => Ok(Some(number_token)),
                    Ok(Some((number_token, Some(extra_token)))) => {
                        json_tokens.push(number_token);
                        Ok(Some(extra_token))
                    }
                    Ok(None) => Ok(None),
                    Err(err) => Err(err),
                },
                Some(Token::Null(_)) => handle_null(&buf, i, tokens),
                Some(Token::True(_)) => handle_true(&buf, i, tokens),
                Some(Token::False(_)) => handle_false(&buf, i, tokens),
                Some(Token::Arr) => handle_arr(&buf, i, tokens),
                Some(Token::Comma) => handle_comma(&buf, i, tokens),
                Some(Token::None) => handle_nil_token(&buf, i, tokens),
            };
            match res {
                Ok(Some(new_token)) => json_tokens.push(new_token),
                Ok(None) => {}
                Err(err) => {
                    error = Some(err);
                }
            }
            i += 1;
        }
        match error {
            None => Ok(json_tokens),
            Some(err) => Err(err),
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

        let mut reader = JsonStreamReader::new();
        let res = reader.read(&buf);
        assert_eq!(
            res.unwrap(),
            vec![
                JsonToken::ObjBeg,
                JsonToken::Key("foo1".to_string()),
                JsonToken::Val(JsonValue::String("bar1".to_string())),
                JsonToken::Key("foo2".to_string()),
                JsonToken::Val(JsonValue::String("bar2".to_string())),
                JsonToken::Key("foo3".to_string()),
                JsonToken::ObjBeg,
                JsonToken::Key("foo4".to_string()),
                JsonToken::Val(JsonValue::String("bar4".to_string())),
                JsonToken::ObjEnd,
                JsonToken::Key("foo5".to_string()),
                JsonToken::ArrBeg,
                JsonToken::Val(JsonValue::String("bar5".to_string())),
                JsonToken::Val(JsonValue::String("bar6".to_string())),
                JsonToken::ArrEnd,
                JsonToken::ObjEnd
            ]
        );
    }
}
