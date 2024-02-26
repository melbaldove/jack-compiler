use std::{
    fs::File,
    io::{Error, Write},
    iter::Peekable,
    path::PathBuf,
};

use crate::{
    symbol_table::{SegmentKind, SymbolTable},
    tokenizer::Token,
};

pub struct CompilationEngine<'a, I>
where
    I: Iterator<Item = Token<'a>>,
{
    file: File,
    tokenizer: Peekable<I>,
    class_symbol_table: SymbolTable,
    subroutine_symbol_table: SymbolTable,
    class_name: String,
}

impl<'a, I> CompilationEngine<'a, I>
where
    I: Iterator<Item = Token<'a>>,
{
    pub fn build(tokenizer: I, path: PathBuf) -> Result<CompilationEngine<'a, I>, Error> {
        let file = File::create(&path)?;

        Ok(CompilationEngine {
            file,
            tokenizer: tokenizer.peekable(),
            class_symbol_table: SymbolTable::new(),
            subroutine_symbol_table: SymbolTable::new(),
            class_name: String::new(),
        })
    }

    fn process(&mut self, token: Token) {
        match self.tokenizer.next() {
            Some(current_token) if current_token == token => {
                self.writeln(&format!("{current_token}"))
            }
            Some(_) => self.writeln("error: syntax error"),
            None => return,
        }
    }

    pub fn compile_class(&mut self) {
        self.writeln("<class>");
        // parse class
        self.process(Token::Keyword("class"));

        // parse className
        match self.tokenizer.next() {
            Some(Token::Identifier(class_name)) => {
                self.writeln("<identifier>");
                self.writeln(&format!("<name>{class_name}</name>"));
                self.writeln("<category>class</category>");
                self.writeln("<usage>declared</usage>");
                self.writeln("</identifier>");
                self.class_name.push_str(class_name)
            }
            _ => self.writeln("error: syntax error"),
        }

        // parse body
        self.process(Token::Symbol('{'));
        while let Some(token) = self.tokenizer.peek() {
            match *token {
                Token::Keyword("static" | "field") => self.compile_class_var_dec(),
                _ => break,
            }
        }
        while let Some(token) = self.tokenizer.peek() {
            match *token {
                Token::Keyword("constructor" | "function" | "method") => self.compile_subroutine(),
                _ => break,
            }
        }
        self.process(Token::Symbol('}'));
        self.writeln("</class>");
    }

    pub fn compile_class_var_dec(&mut self) {
        self.writeln("<classVarDec>");
        // parse static | field
        let category = match self.tokenizer.next() {
            Some(t @ Token::Keyword(k @ ("static" | "field"))) => {
                self.writeln(&format!("{t}"));
                Some(k)
            }
            _ => {
                self.writeln("error: syntax error");
                None
            }
        };

        // parse type
        let _type = match self.tokenizer.next() {
            Some(t @ (Token::Keyword(k @ ("int" | "char" | "boolean")) | Token::Identifier(k))) => {
                self.writeln(&format!("{t}"));
                Some(k)
            }
            _ => {
                self.writeln("error: syntax error");
                None
            }
        };

        // parse varName
        match self.tokenizer.next() {
            Some(Token::Identifier(var_name)) => {
                let category = category.unwrap(); // ignore error handling for now
                let _type = _type.unwrap();
                let kind = get_segment_kind(category).unwrap();
                self.class_symbol_table.define(var_name, _type, kind);
                let index = self.class_symbol_table.index_of(var_name).unwrap();
                self.writeln("<identifier>");
                self.writeln(&format!("<name>{var_name}</name>"));
                self.writeln(&format!("<category>{category}</category>"));
                self.writeln(&format!("<index>{index}</index>"));
                self.writeln("<usage>declared</usage>");
                self.writeln("</identifier>");
            }
            _ => self.writeln("error: syntax error"),
        }

        // parse delimiter whether , or ;
        while let Some(token) = self.tokenizer.next() {
            match token {
                Token::Symbol(',') => {
                    self.writeln(&format!("{}", token));
                    if let Some(Token::Identifier(var_name)) = self.tokenizer.next() {
                        let category = category.unwrap(); // ignore error handling for now
                        let _type = _type.unwrap();
                        let kind = get_segment_kind(category).unwrap();
                        self.class_symbol_table.define(var_name, _type, kind);
                        let index = self.class_symbol_table.index_of(var_name).unwrap();
                        self.writeln("<identifier>");
                        self.writeln(&format!("<name>{var_name}</name>"));
                        self.writeln(&format!("<category>{category}</category>"));
                        self.writeln(&format!("<index>{index}</index>"));
                        self.writeln("<usage>declared</usage>");
                        self.writeln("</identifier>");
                    } else {
                        self.writeln("error: syntax error");
                    }
                }
                Token::Symbol(';') => {
                    self.writeln(&format!("{token}"));
                    break;
                }

                _ => {
                    self.writeln("error: syntax error");
                    break;
                }
            }
        }

        self.writeln("</classVarDec>");
    }

    pub fn compile_subroutine(&mut self) {
        self.writeln("<subroutineDec>");
        self.subroutine_symbol_table.reset();
        self.subroutine_symbol_table
            .define("this", self.class_name.as_str(), SegmentKind::Arg);
        // parse constructor | function | method
        match self.tokenizer.next() {
            Some(k @ Token::Keyword("constructor" | "function" | "method")) => {
                self.writeln(&format!("{k}"))
            }
            _ => self.writeln("error: syntax error"),
        }

        // parse type
        match self.tokenizer.next() {
            Some(
                t @ (Token::Keyword("void" | "int" | "char" | "boolean") | Token::Identifier(_)),
            ) => self.writeln(&format!("{t}")),
            _ => self.writeln("error: syntax error"),
        }

        // parse subroutineName
        match self.tokenizer.next() {
            Some(Token::Identifier(i)) => {
                self.writeln("<identifier>");
                self.writeln(&format!("<name>{i}</name>"));
                self.writeln("<category>subroutine</category>");
                self.writeln("</identifier>");
            }
            _ => self.writeln("error: syntax error"),
        }

        self.process(Token::Symbol('('));
        self.compile_parameter_list();
        self.process(Token::Symbol(')'));
        self.compile_subroutine_body();
        self.writeln("</subroutineDec>");
    }

    pub fn compile_parameter_list(&mut self) {
        // peek if its )
        // if its not, process the parameter list
        self.writeln("<parameterList>");
        while let Some(token) = self.tokenizer.peek() {
            match *token {
                Token::Symbol(')') => break,
                Token::Keyword(t @ ("int" | "char" | "boolean")) | Token::Identifier(t) => {
                    // parse type and advance the iterator
                    let token = self.tokenizer.next().unwrap();
                    self.writeln(&format!("{token}"));

                    // parse varName
                    match self.tokenizer.next() {
                        Some(Token::Identifier(arg_name)) => {
                            self.subroutine_symbol_table
                                .define(arg_name, t, SegmentKind::Arg);
                            let index = self.subroutine_symbol_table.index_of(arg_name).unwrap();
                            self.writeln("<identifier>");
                            self.writeln(&format!("<name>{arg_name}</name>"));
                            self.writeln("<category>arg</category>");
                            self.writeln(&format!("<index>{index}</index>"));
                            self.writeln("<usage>declared</usage>");
                            self.writeln("</identifier>");
                        }
                        _ => self.writeln("error: syntax error"),
                    }
                }
                Token::Symbol(',') => {
                    let token = self.tokenizer.next().unwrap();
                    self.writeln(&format!("{token}"));
                }
                _ => self.writeln("error: syntax error"),
            }
        }
        self.writeln("</parameterList>");
    }

    pub fn compile_subroutine_body(&mut self) {
        self.writeln("<subroutineBody>");
        self.process(Token::Symbol('{'));
        // loop
        while let Some(token) = self.tokenizer.peek() {
            match *token {
                Token::Keyword("var") => self.compile_var_dec(),
                _ => break,
            }
        }
        self.compile_statements();
        self.process(Token::Symbol('}'));
        self.writeln("</subroutineBody>");
    }

    pub fn compile_var_dec(&mut self) {
        self.writeln("<varDec>");
        self.process(Token::Keyword("var"));
        // parse type
        let _type = match self.tokenizer.next() {
            Some(k @ (Token::Keyword(t @ ("int" | "char" | "boolean")) | Token::Identifier(t))) => {
                self.writeln(&format!("{k}"));
                Some(t)
            }
            _ => {
                self.writeln("error: syntax error");
                None
            }
        };

        // parse varName
        match self.tokenizer.next() {
            Some(Token::Identifier(var_name)) => {
                let _type = _type.unwrap();
                self.subroutine_symbol_table
                    .define(var_name, _type, SegmentKind::Var);
                let index = self.subroutine_symbol_table.index_of(var_name).unwrap();
                self.writeln("<identifier>");
                self.writeln(&format!("<name>{var_name}</name>"));
                self.writeln("<category>var</category>");
                self.writeln(&format!("<index>{index}</index>"));
                self.writeln("<usage>declared</usage>");
                self.writeln("</identifier>");
            }
            _ => self.writeln("error: syntax error"),
        }

        // parse delimiter whether , or ;
        while let Some(token) = self.tokenizer.next() {
            match token {
                Token::Symbol(',') => {
                    self.writeln(&format!("{}", token));
                    if let Some(Token::Identifier(var_name)) = self.tokenizer.next() {
                        let _type = _type.unwrap();
                        self.subroutine_symbol_table
                            .define(var_name, _type, SegmentKind::Var);
                        let index = self.subroutine_symbol_table.index_of(var_name).unwrap();
                        self.writeln("<identifier>");
                        self.writeln(&format!("<name>{var_name}</name>"));
                        self.writeln("<category>var</category>");
                        self.writeln(&format!("<index>{index}</index>"));
                        self.writeln("<usage>declared</usage>");
                        self.writeln("</identifier>");
                    } else {
                        self.writeln("error: syntax error");
                    }
                }
                Token::Symbol(';') => {
                    self.writeln(&format!("{token}"));
                    break;
                }

                _ => {
                    self.writeln("error: syntax error");
                    break;
                }
            }
        }
        self.writeln("</varDec>");
    }

    pub fn compile_statements(&mut self) {
        self.writeln("<statements>");
        while let Some(token) = self.tokenizer.peek() {
            match *token {
                Token::Symbol('}') => break,
                Token::Keyword("let") => self.compile_let(),
                Token::Keyword("if") => self.compile_if(),
                Token::Keyword("while") => self.compile_while(),
                Token::Keyword("do") => self.compile_do(),
                Token::Keyword("return") => self.compile_return(),
                _ => {
                    self.writeln("error: syntax error");
                    break;
                }
            }
        }
        self.writeln("</statements>");
    }

    pub fn compile_let(&mut self) {
        self.writeln("<letStatement>");
        // parse let
        self.process(Token::Keyword("let"));

        // parse varName
        match self.tokenizer.next() {
            Some(Token::Identifier(ident)) => {
                self.writeln("<identifier>");
                let category = self.kind_of(ident);
                let _type = self.type_of(ident);
                let index = self.index_of(ident);
                self.writeln(&format!("<name>{ident}</name>"));
                if let Some(category) = category {
                    self.writeln(&format!("<category>{category}</category>"));
                } else {
                    self.writeln("<category>error</category>");
                }
                if let Some(_type) = _type {
                    self.writeln(&format!("<type>{_type}</type>"));
                } else {
                    self.writeln(&format!("<type>error</type>"));
                }
                if let Some(index) = index {
                    self.writeln(&format!("<index>{index}</index>"));
                } else {
                    self.writeln(&format!("<index>error</index>"));
                }
                self.writeln("<usage>used</usage>");
                self.writeln("</identifier>");
            }
            _ => self.writeln("error: syntax error"),
        }

        if let Some(Token::Symbol('[')) = self.tokenizer.peek() {
            self.process(Token::Symbol('['));
            self.compile_expression();
            self.process(Token::Symbol(']'));
        }

        self.process(Token::Symbol('='));

        self.compile_expression();
        self.process(Token::Symbol(';'));

        self.writeln("</letStatement>");
    }

    pub fn compile_if(&mut self) {
        self.writeln("<ifStatement>");
        // parse if
        self.process(Token::Keyword("if"));

        self.process(Token::Symbol('('));
        self.compile_expression();
        self.process(Token::Symbol(')'));
        self.process(Token::Symbol('{'));
        self.compile_statements();
        self.process(Token::Symbol('}'));

        if let Some(Token::Keyword("else")) = self.tokenizer.peek() {
            self.process(Token::Keyword("else"));
            self.process(Token::Symbol('{'));
            self.compile_statements();
            self.process(Token::Symbol('}'));
        }
        self.writeln("</ifStatement>");
    }

    pub fn compile_while(&mut self) {
        self.writeln("<whileStatement>");
        // parse while
        self.process(Token::Keyword("while"));
        self.process(Token::Symbol('('));
        self.compile_expression();
        self.process(Token::Symbol(')'));

        self.process(Token::Symbol('{'));
        self.compile_statements();
        self.process(Token::Symbol('}'));
        self.writeln("</whileStatement>");
    }

    pub fn compile_do(&mut self) {
        self.writeln("<doStatement>");

        // parse do
        self.process(Token::Keyword("do"));
        self.compile_term();
        self.process(Token::Symbol(';'));

        self.writeln("</doStatement>");
    }

    pub fn compile_return(&mut self) {
        self.writeln("<returnStatement>");

        // parse return
        self.process(Token::Keyword("return"));

        if let Some(Token::Symbol(';')) = self.tokenizer.peek() {
            self.process(Token::Symbol(';'));
        } else {
            self.compile_expression();
            self.process(Token::Symbol(';'));
        }
        self.writeln("</returnStatement>");
    }

    pub fn compile_expression(&mut self) {
        self.writeln("<expression>");
        self.compile_term();
        while let Some(token) = self.tokenizer.peek() {
            match *token {
                Token::Symbol('+' | '-' | '*' | '/' | '&' | '|' | '<' | '>' | '=') => {
                    let token = self.tokenizer.next().unwrap();
                    self.writeln(&format!("{}", token));
                    self.compile_term();
                }
                _ => break,
            }
        }
        self.writeln("</expression>");
    }

    pub fn compile_term(&mut self) {
        self.writeln("<term>");
        match self.tokenizer.peek() {
            Some(
                Token::IntConstant(_)
                | Token::StringConst(_)
                | Token::Keyword("true" | "false" | "null" | "this"),
            ) => {
                let c = self.tokenizer.next().unwrap();
                self.writeln(&format!("{c}"));
            }
            Some(Token::Symbol('(')) => {
                self.process(Token::Symbol('('));
                self.compile_expression();
                self.process(Token::Symbol(')'));
            }
            Some(Token::Identifier(_)) => {
                let ident = if let Token::Identifier(ident) = self.tokenizer.next().unwrap() {
                    ident
                } else {
                    panic!()
                };

                let index = self.index_of(ident);
                let _type = self.type_of(ident);
                let segment = self.kind_of(ident);
                self.writeln("<identifier>");
                self.writeln(&format!("<name>{ident}</name>"));
                self.writeln("<usage>used</usage>");
                if let Some(index) = index {
                    self.writeln(&format!("<index>{index}</index>"));
                }
                if let Some(_type) = _type {
                    self.writeln(&format!("<type>{_type}</type>"));
                }

                match self.tokenizer.peek() {
                    Some(Token::Symbol('[')) => {
                        if let Some(segment) = segment {
                            self.writeln(&format!("<category>{segment}</category>"));
                        } else {
                            self.writeln("<category>error</category>");
                        }
                        self.writeln(&format!("</identifier>"));
                        self.process(Token::Symbol('['));
                        self.compile_expression();
                        self.process(Token::Symbol(']'));
                    }
                    Some(Token::Symbol('(' | '.')) => {
                        if let Some(Token::Symbol('.')) = self.tokenizer.peek() {
                            if let Some(segment) = segment {
                                self.writeln(&format!("<category>{segment}</category>"));
                            } else {
                                self.writeln(&format!("<category>class</category>"));
                            }
                            self.writeln(&format!("</identifier>"));
                            self.process(Token::Symbol('.'));

                            // parse subroutineName
                            match self.tokenizer.next() {
                                Some(Token::Identifier(ident)) => {
                                    self.writeln("<identifier>");
                                    self.writeln(&format!("<name>{ident}</name>"));
                                    self.writeln("<category>subroutine</category>");
                                    self.writeln("<usage>used</usage>");
                                    self.writeln("</identifier>");
                                }
                                _ => self.writeln("error: syntax error"),
                            }
                        } else {
                            self.writeln("<category>subroutine</category>");
                            self.writeln(&format!("</identifier>"));
                        }

                        self.process(Token::Symbol('('));
                        self.compile_expression_list();
                        self.process(Token::Symbol(')'));
                    }
                    _ => {
                        if let Some(segment) = segment {
                            self.writeln(&format!("<category>{segment}</category>"));
                        } else {
                            self.writeln("<category>error</category>");
                        }
                        self.writeln(&format!("</identifier>"));
                    }
                }
            }
            Some(Token::Symbol('-' | '~')) => {
                let s = self.tokenizer.next().unwrap();
                self.writeln(&format!("{s}"));
                self.compile_term();
            }
            _ => self.writeln("error: syntax error"),
        }
        self.writeln("</term>");
    }

    pub fn compile_expression_list(&mut self) -> usize {
        self.writeln("<expressionList>");
        let mut count = 0;
        while let Some(token) = self.tokenizer.peek() {
            match *token {
                Token::StringConst(_)
                | Token::IntConstant(_)
                | Token::Identifier(_)
                | Token::Symbol('-' | '~' | '(')
                | Token::Keyword("true" | "false" | "null" | "this") => {
                    count += 1;
                    self.compile_expression();
                }
                Token::Symbol(',') => {
                    self.process(Token::Symbol(','));
                }
                _ => break,
            }
        }
        self.writeln("</expressionList>");
        return count;
    }

    fn writeln(&mut self, str: &str) {
        let _ = self.file.write_all(format!("{}\n", str).as_bytes());
    }

    fn kind_of(&self, name: &str) -> Option<SegmentKind> {
        self.subroutine_symbol_table
            .kind_of(name)
            .or_else(|| self.class_symbol_table.kind_of(name))
    }

    fn type_of(&self, name: &str) -> Option<String> {
        self.subroutine_symbol_table
            .type_of(name)
            .or_else(|| self.class_symbol_table.type_of(name))
    }

    fn index_of(&self, name: &str) -> Option<usize> {
        self.subroutine_symbol_table
            .index_of(name)
            .or_else(|| self.class_symbol_table.index_of(name))
    }
}
fn get_segment_kind(str: &str) -> Option<SegmentKind> {
    match str {
        "field" => Some(SegmentKind::Field),
        "static" => Some(SegmentKind::Static),
        "var" => Some(SegmentKind::Var),
        "arg" => Some(SegmentKind::Arg),
        _ => None,
    }
}
