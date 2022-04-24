use crate::token::Token;

pub(crate) fn squash(tokens: &mut Vec<Token>) {
    while let Some(token) = tokens.last() {
        match token {
            Token::Colon => break,
            Token::Arr => break,
            _ => {}
        }
        tokens.pop();
    }
    if let Some(token) = tokens.last() {
        if let Token::Colon = token {
            while let Some(token) = tokens.pop() {
                if let Token::Key(_) = token {
                    break;
                }
            }
        }
    }
    if tokens.len() > 0 {
        tokens.push(Token::None);
    }
}

#[cfg(test)]
mod squash_tests {
    use super::*;

    #[test]
    fn should_squash_simple_obj() {
        let mut tokens = vec![
            Token::Obj,
            Token::Key("foo".as_bytes().to_vec()),
            Token::Colon,
            Token::String("bar".as_bytes().to_vec()),
        ];

        squash(&mut tokens);
        assert_eq!(tokens, vec![Token::Obj, Token::None]);
    }

    #[test]
    fn should_squash_key_val() {
        let mut tokens = vec![
            Token::Key("foo".as_bytes().to_vec()),
            Token::Colon,
            Token::String("bar".as_bytes().to_vec()),
        ];

        squash(&mut tokens);
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn should_squash_key_val_list() {
        let mut tokens = vec![
            Token::Arr,
            Token::String("foo".as_bytes().to_vec()),
            Token::Comma,
            Token::String("bar".as_bytes().to_vec()),
            Token::Comma,
            Token::String("zar".as_bytes().to_vec()),
        ];

        squash(&mut tokens);
        assert_eq!(tokens, vec![Token::Arr, Token::None]);
    }

    #[test]
    fn should_ignore_empty_list() {
        let mut tokens = vec![];

        squash(&mut tokens);
        assert_eq!(tokens, vec![]);
    }
}
