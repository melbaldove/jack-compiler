use std::{
    iter::{Enumerate, Peekable},
    str::Bytes,
};

#[derive(Clone)]
pub struct Tokenizer<'a> {
    file_contents: &'a str,
    iterator: Peekable<Enumerate<Bytes<'a>>>,
    cur_token_start: usize,
    state: State,
}

impl<'a> Tokenizer<'a> {
    pub fn build(file_contents: &str) -> Result<Tokenizer, &'static str> {
        let iterator = file_contents.bytes().enumerate().peekable();
        let tokenizer = Tokenizer {
            file_contents,
            iterator,
            cur_token_start: 0,
            state: State::Code,
        };
        Ok(tokenizer)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Token<'a> {
    Keyword(&'a str),
    Symbol(char),
    Identifier(&'a str),
    IntConstant(usize),
    StringConst(&'a str),
    Whitespace(char),
    SingleLineComment(&'a str),
    BlockComment(&'a str),
    Invalid(&'a str),
}

#[derive(Clone, Debug)]
enum State {
    Code,
    StringLiteral,
    SingleLineComment,
    BlockComment,
    BlockCommentEndStar,
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (i, b) = match self.iterator.next() {
                Some(i) => i,
                None => break None,
            };
            let (next_i, next_b) = match self.iterator.peek() {
                Some(i) => i,
                None => break None,
            };
            match self.state {
                State::Code => match b {
                    b if b.is_ascii_whitespace() => {
                        self.cur_token_start = *next_i;
                        break Some(Token::Whitespace(b as char));
                    }
                    b'"' => {
                        self.cur_token_start = *next_i;
                        self.state = State::StringLiteral;
                        continue;
                    }
                    b'/' if *next_b == b'/' => {
                        let (i, _) = self.iterator.next().expect("Expect next value as token will have been returned immediately otherwise"); // skip the next *
                        self.cur_token_start = i + 1;
                        self.state = State::SingleLineComment;
                        continue;
                    }
                    b'/' if *next_b == b'*' => {
                        let (i, _) = self.iterator.next().expect("Expect next value as token will have been returned immediately otherwise"); // skip the next *
                        self.cur_token_start = i + 1;
                        self.state = State::BlockComment;
                        continue;
                    }
                    b => {
                        if is_symbol(b) {
                            self.cur_token_start = *next_i;
                            break Some(Token::Symbol(b as char));
                        }
                        if next_b.is_ascii_whitespace() || is_symbol(*next_b) {
                            let token = &self.file_contents[self.cur_token_start..*next_i];
                            let token = match token {
                                token if is_keyword(token) => Token::Keyword(token),
                                token if is_numeric(token) => {
                                    Token::IntConstant(token.parse().unwrap())
                                }
                                token if is_valid_identifier(token) => Token::Identifier(token),
                                token => Token::Invalid(token),
                            };
                            self.cur_token_start = *next_i;
                            break Some(token);
                        }
                        continue;
                    }
                },
                State::StringLiteral => match b {
                    b'"' => {
                        let token = &self.file_contents[self.cur_token_start..i];
                        self.cur_token_start = *next_i;
                        self.state = State::Code;
                        break Some(Token::StringConst(token));
                    }
                    _ => continue,
                },
                State::SingleLineComment => match b {
                    b'\n' => {
                        let comment = &self.file_contents[self.cur_token_start..*next_i].trim();
                        self.cur_token_start = *next_i;
                        self.state = State::Code;
                        break Some(Token::SingleLineComment(comment));
                    }
                    _ => continue,
                },
                State::BlockComment => match b {
                    b'*' => {
                        self.state = State::BlockCommentEndStar;
                        continue;
                    }
                    _ => continue,
                },
                State::BlockCommentEndStar => match b {
                    b'/' => {
                        let comment = &self.file_contents[self.cur_token_start..*next_i - 2].trim();
                        self.cur_token_start = *next_i;
                        self.state = State::Code;
                        break Some(Token::BlockComment(comment));
                    }
                    _ => {
                        self.state = State::BlockComment;
                        continue;
                    }
                },
            }
        }
    }
}

fn is_valid_identifier(token: &str) -> bool {
    let first_char = token.chars().next().unwrap_or('0');
    token.chars().all(|c| c.is_alphanumeric() || c == '_') && !first_char.is_numeric()
}

fn is_numeric(token: &str) -> bool {
    token.chars().all(char::is_numeric)
}

fn is_keyword(token: &str) -> bool {
    matches!(
        token,
        "class"
            | "constructor"
            | "function"
            | "method"
            | "field"
            | "static"
            | "var"
            | "int"
            | "char"
            | "boolean"
            | "void"
            | "true"
            | "false"
            | "null"
            | "this"
            | "let"
            | "do"
            | "if"
            | "else"
            | "while"
            | "return"
    )
}

fn is_symbol(c: u8) -> bool {
    matches!(
        c,
        b'{' | b'}'
            | b'('
            | b')'
            | b'['
            | b']'
            | b'.'
            | b','
            | b';'
            | b'+'
            | b'-'
            | b'*'
            | b'/'
            | b'&'
            | b'|'
            | b'<'
            | b'>'
            | b'='
            | b'~'
    )
}
