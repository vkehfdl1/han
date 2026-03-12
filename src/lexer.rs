#![allow(dead_code, unused)]

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // 키워드 (Keywords)
    함수,
    반환,
    변수,
    상수,
    만약,
    아니면,
    반복,
    동안,
    멈춰,
    계속,
    참,
    거짓,
    없음,
    출력,
    입력,
    // 타입 키워드
    정수타입,
    실수타입,
    문자열타입,
    불타입,
    없음타입,
    // 리터럴
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    // 식별자
    Identifier(String),
    // 산술 연산자
    Plus,    // +
    Minus,   // -
    Star,    // *
    Slash,   // /
    Percent, // %
    // 비교 연산자
    EqEq,   // ==
    BangEq, // !=
    Lt,     // <
    Gt,     // >
    LtEq,   // <=
    GtEq,   // >=
    // 논리 연산자
    AmpAmp,   // &&
    PipePipe, // ||
    Bang,     // !
    // 할당 연산자
    Eq,      // =
    PlusEq,  // +=
    MinusEq, // -=
    StarEq,  // *=
    SlashEq, // /=
    // 화살표
    Arrow, // ->
    // 구분자
    Colon,     // :
    Comma,     // ,
    Semicolon, // ;
    // 괄호
    LBrace, // {
    RBrace, // }
    LParen, // (
    RParen, // )
    // 특수
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct TokenWithPos {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}

impl TokenWithPos {
    pub fn new(token: Token, line: usize, col: usize) -> Self {
        Self { token, line, col }
    }
}

pub fn get_keyword_map() -> HashMap<String, Token> {
    let mut map = HashMap::new();
    // 키워드
    map.insert("함수".to_string(), Token::함수);
    map.insert("반환".to_string(), Token::반환);
    map.insert("변수".to_string(), Token::변수);
    map.insert("상수".to_string(), Token::상수);
    map.insert("만약".to_string(), Token::만약);
    map.insert("아니면".to_string(), Token::아니면);
    map.insert("반복".to_string(), Token::반복);
    map.insert("동안".to_string(), Token::동안);
    map.insert("멈춰".to_string(), Token::멈춰);
    map.insert("계속".to_string(), Token::계속);
    map.insert("참".to_string(), Token::참);
    map.insert("거짓".to_string(), Token::거짓);
    map.insert("없음".to_string(), Token::없음);
    map.insert("출력".to_string(), Token::출력);
    map.insert("입력".to_string(), Token::입력);
    // 타입 키워드
    map.insert("정수".to_string(), Token::정수타입);
    map.insert("실수".to_string(), Token::실수타입);
    map.insert("문자열".to_string(), Token::문자열타입);
    map.insert("불".to_string(), Token::불타입);
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_map() {
        let map = get_keyword_map();
        assert_eq!(map.get("함수"), Some(&Token::함수));
        assert_eq!(map.get("반환"), Some(&Token::반환));
        assert_eq!(map.get("만약"), Some(&Token::만약));
        assert_eq!(map.get("정수"), Some(&Token::정수타입));
    }

    #[test]
    fn test_token_with_pos() {
        let t = TokenWithPos::new(Token::함수, 1, 0);
        assert_eq!(t.line, 1);
        assert_eq!(t.col, 0);
        assert!(matches!(t.token, Token::함수));
    }
}
