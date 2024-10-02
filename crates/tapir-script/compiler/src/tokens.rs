use std::num::ParseIntError;

use logos::Logos;
use serde::Serialize;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize)]
pub struct FileId(usize);

impl FileId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Span {
    file_id: FileId,
    start: usize,
    end: usize,
}

impl Span {
    pub fn new(file_id: FileId, start: usize, end: usize) -> Self {
        assert!(start < end, "{} was not less than {}", start, end);
        Self {
            file_id,
            start,
            end,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LexicalError {
    pub kind: LexicalErrorKind,
    pub span: Span,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize)]
pub enum LexicalErrorKind {
    InvalidNumber(#[serde(skip)] ParseIntError),
    #[default]
    InvalidToken,
}

impl From<ParseIntError> for LexicalErrorKind {
    fn from(value: ParseIntError) -> Self {
        Self::InvalidNumber(value)
    }
}

impl LexicalErrorKind {
    pub fn with_span(self, file_id: FileId, start: usize, end: usize) -> LexicalError {
        LexicalError {
            kind: self,
            span: Span::new(file_id, start, end),
        }
    }
}

#[derive(Logos, Clone, Debug, PartialEq, Serialize)]
#[logos(skip r"[ \t\n\f\r]+", skip r"#.*\n?", error = LexicalErrorKind)]
pub enum Token<'input> {
    #[token("wait")]
    KeywordWait,
    #[token("var")]
    KeywordVar,

    #[regex("[_a-zA-Z][_0-9a-zA-Z]*", |lex| lex.slice())]
    Identifier(&'input str),
    #[regex("-?[0-9]+", |lex| lex.slice())]
    Integer(&'input str),
    #[regex("-?[0-9]+\\.[0-9]*", |lex| lex.slice())]
    Fix(&'input str),

    #[token("true")]
    True,
    #[token("false")]
    False,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("=")]
    Assign,
    #[token(";")]
    Semicolon,

    #[token("+")]
    OperatorAdd,
    #[token("-")]
    OperatorSub,
    #[token("*")]
    OperatorMul,
    #[token("/")]
    OperatorDiv,
    #[token("%")]
    OperatorMod,
    #[token("//")]
    OperatorRealDiv,
    #[token("%%")]
    OperatorRealMod,
}
