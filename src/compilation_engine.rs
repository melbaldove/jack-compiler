use std::{io::Error, iter::Peekable, path::PathBuf};

use crate::{
    symbol_table::{SegmentKind, SymbolTable},
    tokenizer::Token,
    vm_writer::{Arithmetic, Segment, VmWriter},
};

pub struct CompilationEngine<'a, I>
where
    I: Iterator<Item = Token<'a>>,
{
    vm_writer: VmWriter,
    tokenizer: Peekable<I>,
    class_symbol_table: SymbolTable,
    subroutine_symbol_table: SymbolTable,
    class_name: String,
    subroutine_name: String,
    subroutine_type: String,
    subroutine_category: String,
    control_counter: usize,
}

impl<'a, I> CompilationEngine<'a, I>
where
    I: Iterator<Item = Token<'a>>,
{
    pub fn build(tokenizer: I, path: PathBuf) -> Result<CompilationEngine<'a, I>, Error> {
        let vm_writer = VmWriter::build(path)?;

        Ok(CompilationEngine {
            vm_writer,
            tokenizer: tokenizer.peekable(),
            class_symbol_table: SymbolTable::new(),
            subroutine_symbol_table: SymbolTable::new(),
            class_name: String::new(),
            subroutine_name: String::new(),
            subroutine_type: String::new(),
            subroutine_category: String::new(),
            control_counter: 0,
        })
    }

    fn process(&mut self, token: Token) {
        match self.tokenizer.next() {
            Some(current_token) if current_token == token => {
                println!("{current_token}");
            }
            Some(_) => println!("error: syntax error"),
            None => return,
        }
    }

    pub fn compile_class(&mut self) {
        println!("<class>");
        // parse class
        self.process(Token::Keyword("class"));

        // parse className
        match self.tokenizer.next() {
            Some(Token::Identifier(class_name)) => {
                println!("<identifier>");
                println!("<name>{class_name}</name>");
                println!("<category>class</category>");
                println!("<usage>declared</usage>");
                println!("</identifier>");
                self.class_name.push_str(class_name)
            }
            _ => println!("error: syntax error"),
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
        println!("</class>");
    }

    pub fn compile_class_var_dec(&mut self) {
        println!("<classVarDec>");
        // parse static | field
        let category = match self.tokenizer.next() {
            Some(t @ Token::Keyword(k @ ("static" | "field"))) => {
                println!("{t}");
                Some(k)
            }
            _ => {
                println!("error: syntax error");
                None
            }
        };

        // parse type
        let _type = match self.tokenizer.next() {
            Some(t @ (Token::Keyword(k @ ("int" | "char" | "boolean")) | Token::Identifier(k))) => {
                println!("{t}");
                Some(k)
            }
            _ => {
                println!("error: syntax error");
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
                println!("<identifier>");
                println!("<name>{var_name}</name>");
                println!("<category>{category}</category>");
                println!("<index>{index}</index>");
                println!("<usage>declared</usage>");
                println!("</identifier>");
            }
            _ => println!("error: syntax error"),
        }

        // parse delimiter whether , or ;
        while let Some(token) = self.tokenizer.next() {
            match token {
                Token::Symbol(',') => {
                    println!("{}", token);
                    if let Some(Token::Identifier(var_name)) = self.tokenizer.next() {
                        let category = category.unwrap(); // ignore error handling for now
                        let _type = _type.unwrap();
                        let kind = get_segment_kind(category).unwrap();
                        self.class_symbol_table.define(var_name, _type, kind);
                        let index = self.class_symbol_table.index_of(var_name).unwrap();
                        println!("<identifier>");
                        println!("<name>{var_name}</name>");
                        println!("<category>{category}</category>");
                        println!("<index>{index}</index>");
                        println!("<usage>declared</usage>");
                        println!("</identifier>");
                    } else {
                        println!("error: syntax error");
                    }
                }
                Token::Symbol(';') => {
                    println!("{token}");
                    break;
                }

                _ => {
                    println!("error: syntax error");
                    break;
                }
            }
        }

        println!("</classVarDec>");
    }

    pub fn compile_subroutine(&mut self) {
        println!("<subroutineDec>");
        self.subroutine_symbol_table.reset();
        self.subroutine_name.clear();
        self.subroutine_category.clear();
        self.subroutine_type.clear();
        // parse constructor | function | method
        match self.tokenizer.next() {
            Some(k @ Token::Keyword(c @ ("constructor" | "function" | "method"))) => {
                println!("{k}");
                self.subroutine_category.push_str(c);
                Some(c)
            }
            _ => {
                println!("error: syntax error");
                None
            }
        };
        if self.subroutine_category == "method" {
            self.subroutine_symbol_table
                .define("this", self.class_name.as_str(), SegmentKind::Arg);
        }

        // parse type
        match self.tokenizer.next() {
            Some(
                t @ (Token::Keyword(_type @ ("void" | "int" | "char" | "boolean"))
                | Token::Identifier(_type)),
            ) => {
                println!("{t}");
                self.subroutine_type.push_str(_type);
            }
            _ => println!("error: syntax error"),
        }

        // parse subroutineName
        match self.tokenizer.next() {
            Some(Token::Identifier(i)) => {
                println!("<identifier>");
                println!("<name>{i}</name>");
                println!("<category>subroutine</category>");
                println!("</identifier>");
                self.subroutine_name.push_str(i);
            }
            _ => {
                println!("error: syntax error");
            }
        };

        self.process(Token::Symbol('('));
        self.compile_parameter_list();
        self.process(Token::Symbol(')'));
        self.compile_subroutine_body();

        println!("</subroutineDec>");
    }

    pub fn compile_parameter_list(&mut self) {
        // peek if its )
        // if its not, process the parameter list
        println!("<parameterList>");
        while let Some(token) = self.tokenizer.peek() {
            match *token {
                Token::Symbol(')') => break,
                Token::Keyword(t @ ("int" | "char" | "boolean")) | Token::Identifier(t) => {
                    // parse type and advance the iterator
                    let token = self.tokenizer.next().unwrap();
                    println!("{token}");

                    // parse varName
                    match self.tokenizer.next() {
                        Some(Token::Identifier(arg_name)) => {
                            self.subroutine_symbol_table
                                .define(arg_name, t, SegmentKind::Arg);
                            let index = self.subroutine_symbol_table.index_of(arg_name).unwrap();
                            println!("<identifier>");
                            println!("<name>{arg_name}</name>");
                            println!("<category>arg</category>");
                            println!("<index>{index}</index>");
                            println!("<usage>declared</usage>");
                            println!("</identifier>");
                        }
                        _ => println!("error: syntax error"),
                    }
                }
                Token::Symbol(',') => {
                    let token = self.tokenizer.next().unwrap();
                    println!("{token}");
                }
                _ => println!("error: syntax error"),
            }
        }
        println!("</parameterList>");
    }

    pub fn compile_subroutine_body(&mut self) {
        println!("<subroutineBody>");
        self.process(Token::Symbol('{'));
        // loop
        while let Some(token) = self.tokenizer.peek() {
            match *token {
                Token::Keyword("var") => self.compile_var_dec(),
                _ => break,
            }
        }
        let n_vars = self.subroutine_symbol_table.var_count(SegmentKind::Var);
        self.vm_writer.write_function(
            &format!("{}.{}", self.class_name, self.subroutine_name),
            n_vars,
        );

        if self.subroutine_category == "constructor" {
            self.vm_writer.write_push(
                Segment::CONSTANT,
                self.class_symbol_table.var_count(SegmentKind::Field),
            );
            self.vm_writer.write_call("Memory.alloc", 1);
            self.vm_writer.write_pop(Segment::POINTER, 0);
        }

        if self.subroutine_category == "method" {
            self.vm_writer.write_push(Segment::ARGUMENT, 0);
            self.vm_writer.write_pop(Segment::POINTER, 0);
        }

        self.compile_statements();
        self.process(Token::Symbol('}'));
        println!("</subroutineBody>");
    }

    pub fn compile_var_dec(&mut self) {
        println!("<varDec>");
        self.process(Token::Keyword("var"));
        // parse type
        let _type = match self.tokenizer.next() {
            Some(k @ (Token::Keyword(t @ ("int" | "char" | "boolean")) | Token::Identifier(t))) => {
                println!("{k}");
                Some(t)
            }
            _ => {
                println!("error: syntax error");
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
                println!("<identifier>");
                println!("<name>{var_name}</name>");
                println!("<category>var</category>");
                println!("<index>{index}</index>");
                println!("<usage>declared</usage>");
                println!("</identifier>");
            }
            _ => println!("error: syntax error"),
        }

        // parse delimiter whether , or ;
        while let Some(token) = self.tokenizer.next() {
            match token {
                Token::Symbol(',') => {
                    println!("{}", token);
                    if let Some(Token::Identifier(var_name)) = self.tokenizer.next() {
                        let _type = _type.unwrap();
                        self.subroutine_symbol_table
                            .define(var_name, _type, SegmentKind::Var);
                        let index = self.subroutine_symbol_table.index_of(var_name).unwrap();
                        println!("<identifier>");
                        println!("<name>{var_name}</name>");
                        println!("<category>var</category>");
                        println!("<index>{index}</index>");
                        println!("<usage>declared</usage>");
                        println!("</identifier>");
                    } else {
                        println!("error: syntax error");
                    }
                }
                Token::Symbol(';') => {
                    println!("{token}");
                    break;
                }

                _ => {
                    println!("error: syntax error");
                    break;
                }
            }
        }
        println!("</varDec>");
    }

    pub fn compile_statements(&mut self) {
        println!("<statements>");
        while let Some(token) = self.tokenizer.peek() {
            match *token {
                Token::Symbol('}') => break,
                Token::Keyword("let") => self.compile_let(),
                Token::Keyword("if") => self.compile_if(),
                Token::Keyword("while") => self.compile_while(),
                Token::Keyword("do") => self.compile_do(),
                Token::Keyword("return") => self.compile_return(),
                _ => {
                    println!("error: syntax error");
                    break;
                }
            }
        }
        println!("</statements>");
    }

    pub fn compile_let(&mut self) {
        println!("<letStatement>");
        // parse let
        self.process(Token::Keyword("let"));

        // parse varName
        match self.tokenizer.next() {
            Some(Token::Identifier(var_name)) => {
                println!("<identifier>");
                let category = self.kind_of(var_name);
                let _type = self.type_of(var_name);
                let index = self.index_of(var_name);
                println!("<name>{var_name}</name>");
                if let Some(category) = category {
                    println!("<category>{category}</category>");
                } else {
                    println!("<category>error</category>");
                }
                if let Some(_type) = _type {
                    println!("<type>{_type}</type>");
                } else {
                    println!("<type>error</type>");
                }
                if let Some(index) = index {
                    println!("<index>{index}</index>");
                } else {
                    println!("<index>error</index>");
                }
                println!("<usage>used</usage>");
                println!("</identifier>");
                let segment = category.map(|c| match c {
                    SegmentKind::Static => Segment::STATIC,
                    SegmentKind::Field => Segment::THIS,
                    SegmentKind::Arg => Segment::ARGUMENT,
                    SegmentKind::Var => Segment::LOCAL,
                });
                if let Some(Token::Symbol('[')) = self.tokenizer.peek() {
                    if let (Some(segment), Some(index)) = (segment, index) {
                        self.vm_writer.write_push(segment, index);
                    }
                    self.process(Token::Symbol('['));
                    self.compile_expression();
                    self.vm_writer.write_arithmetic(Arithmetic::ADD);
                    self.process(Token::Symbol(']'));

                    self.process(Token::Symbol('='));

                    self.compile_expression();
                    self.vm_writer.write_pop(Segment::TEMP, 0);
                    self.vm_writer.write_pop(Segment::POINTER, 1);
                    self.vm_writer.write_push(Segment::TEMP, 0);
                    self.vm_writer.write_pop(Segment::THAT, 0);
                } else {
                    self.process(Token::Symbol('='));

                    self.compile_expression();

                    if let (Some(segment), Some(index)) = (segment, index) {
                        self.vm_writer.write_pop(segment, index);
                    }
                }

                self.process(Token::Symbol(';'));
            }
            _ => {
                println!("error: syntax error");
            }
        };

        println!("</letStatement>");
    }
    pub fn compile_if(&mut self) {
        let else_label = &format!("{}L{}", self.class_name, self.control_counter);
        self.control_counter += 1;
        let exit_label = &format!("{}L{}", self.class_name, self.control_counter);
        self.control_counter += 1;
        println!("<ifStatement>");
        // parse if
        self.process(Token::Keyword("if"));

        self.process(Token::Symbol('('));
        self.compile_expression();
        self.vm_writer.write_arithmetic(Arithmetic::NOT);
        self.vm_writer.write_if(else_label);
        self.process(Token::Symbol(')'));
        self.process(Token::Symbol('{'));
        self.compile_statements();
        self.vm_writer.write_goto(exit_label);
        self.process(Token::Symbol('}'));

        if let Some(Token::Keyword("else")) = self.tokenizer.peek() {
            self.process(Token::Keyword("else"));
            self.process(Token::Symbol('{'));
            self.vm_writer.write_label(else_label);
            self.compile_statements();
            self.process(Token::Symbol('}'));
        } else {
            self.vm_writer.write_label(else_label);
        }
        self.vm_writer.write_label(exit_label);
        println!("</ifStatement>");
    }

    pub fn compile_while(&mut self) {
        let loop_label = &format!("{}L{}", self.class_name, self.control_counter);
        self.control_counter += 1;
        let exit_label = &format!("{}L{}", self.class_name, self.control_counter);
        self.control_counter += 1;
        println!("<whileStatement>");
        // parse while
        self.process(Token::Keyword("while"));
        self.process(Token::Symbol('('));
        self.vm_writer.write_label(loop_label);
        self.compile_expression();
        self.vm_writer.write_arithmetic(Arithmetic::NOT);
        self.vm_writer.write_if(exit_label);
        self.process(Token::Symbol(')'));

        self.process(Token::Symbol('{'));
        self.compile_statements();
        self.vm_writer.write_goto(loop_label);
        self.vm_writer.write_label(exit_label);
        self.process(Token::Symbol('}'));
        println!("</whileStatement>");
    }

    pub fn compile_do(&mut self) {
        println!("<doStatement>");

        // parse do
        self.process(Token::Keyword("do"));
        self.compile_term();
        self.vm_writer.write_pop(Segment::TEMP, 0);
        self.process(Token::Symbol(';'));

        println!("</doStatement>");
    }

    pub fn compile_return(&mut self) {
        println!("<returnStatement>");

        // parse return
        self.process(Token::Keyword("return"));

        if let Some(Token::Symbol(';')) = self.tokenizer.peek() {
            self.process(Token::Symbol(';'));
        } else {
            self.compile_expression();
            self.process(Token::Symbol(';'));
        }

        if self.subroutine_type == "void" {
            self.vm_writer.write_push(Segment::CONSTANT, 0);
        }
        self.vm_writer.write_return();
        println!("</returnStatement>");
    }

    pub fn compile_expression(&mut self) {
        println!("<expression>");
        self.compile_term();
        while let Some(token) = self.tokenizer.peek() {
            match *token {
                Token::Symbol('+' | '-' | '*' | '/' | '&' | '|' | '<' | '>' | '=') => {
                    if let Some(token @ Token::Symbol(op)) = self.tokenizer.next() {
                        println!("{}", token);
                        self.compile_term();
                        match op {
                            '+' => self.vm_writer.write_arithmetic(Arithmetic::ADD),
                            '-' => self.vm_writer.write_arithmetic(Arithmetic::SUB),
                            '*' => self.vm_writer.write_call("Math.multiply", 2),
                            '/' => self.vm_writer.write_call("Math.divide", 2),
                            '&' => self.vm_writer.write_arithmetic(Arithmetic::AND),
                            '|' => self.vm_writer.write_arithmetic(Arithmetic::OR),
                            '<' => self.vm_writer.write_arithmetic(Arithmetic::LT),
                            '>' => self.vm_writer.write_arithmetic(Arithmetic::GT),
                            '=' => self.vm_writer.write_arithmetic(Arithmetic::EQ),
                            _ => {}
                        };
                    }
                }
                _ => break,
            }
        }
        println!("</expression>");
    }

    pub fn compile_term(&mut self) {
        println!("<term>");
        match self.tokenizer.peek() {
            Some(Token::IntConstant(_)) => {
                if let Some(t @ (Token::IntConstant(c))) = self.tokenizer.next() {
                    self.vm_writer.write_push(Segment::CONSTANT, c);
                    println!("{t}");
                }
            }
            Some(Token::StringConst(_)) => {
                if let Some(t @ (Token::StringConst(c))) = self.tokenizer.next() {
                    self.vm_writer.write_push(Segment::CONSTANT, c.len());
                    self.vm_writer.write_call("String.new", 1);
                    for char in c.bytes() {
                        self.vm_writer.write_push(Segment::CONSTANT, char.into());
                        self.vm_writer.write_call("String.appendChar", 2);
                    }
                    println!("{t}");
                }
            }

            Some(Token::Keyword("true" | "false" | "null" | "this")) => {
                if let Some(Token::Keyword(c)) = self.tokenizer.next() {
                    match c {
                        "null" | "false" => self.vm_writer.write_push(Segment::CONSTANT, 0),
                        "true" => {
                            self.vm_writer.write_push(Segment::CONSTANT, 0);
                            self.vm_writer.write_arithmetic(Arithmetic::NOT)
                        }
                        "this" => self.vm_writer.write_push(Segment::POINTER, 0),
                        _ => {}
                    }
                    println!("{c}");
                }
            }
            Some(Token::Symbol('(')) => {
                self.process(Token::Symbol('('));
                self.compile_expression();
                self.process(Token::Symbol(')'));
            }
            Some(Token::Identifier(_)) => {
                let mut subroutine_name = String::new();
                let ident = if let Token::Identifier(ident) = self.tokenizer.next().unwrap() {
                    ident
                } else {
                    panic!()
                };

                let index = self.index_of(ident);
                let _type = self.type_of(ident);
                let segment = self.kind_of(ident);
                println!("<identifier>");
                println!("<name>{ident}</name>");
                println!("<usage>used</usage>");
                if let Some(index) = index {
                    println!("<index>{index}</index>");
                }
                if let Some(_type) = _type {
                    subroutine_name.push_str(_type.as_str());
                    println!("<type>{_type}</type>");
                } else {
                    subroutine_name.push_str(ident);
                }

                let segment = segment.map(|c| match c {
                    SegmentKind::Static => Segment::STATIC,
                    SegmentKind::Field => Segment::THIS,
                    SegmentKind::Arg => Segment::ARGUMENT,
                    SegmentKind::Var => Segment::LOCAL,
                });
                match self.tokenizer.peek() {
                    Some(Token::Symbol('[')) => {
                        if let Some(segment) = segment {
                            println!("<category>{segment}</category>");
                        } else {
                            println!("<category>error</category>");
                        }
                        if let (Some(segment), Some(index)) = (segment, index) {
                            self.vm_writer.write_push(segment, index);
                        }
                        println!("</identifier>");
                        self.process(Token::Symbol('['));
                        self.compile_expression();
                        self.vm_writer.write_arithmetic(Arithmetic::ADD);
                        self.vm_writer.write_pop(Segment::POINTER, 1);
                        self.vm_writer.write_push(Segment::THAT, 0);
                        self.process(Token::Symbol(']'));
                    }
                    Some(Token::Symbol('(' | '.')) => {
                        let mut n_args = 0;
                        if let Some(Token::Symbol('.')) = self.tokenizer.peek() {
                            if let Some(segment) = segment {
                                println!("<category>{segment}</category>");
                            } else {
                                println!("<category>class</category>");
                            }
                            println!("</identifier>");
                            subroutine_name.push('.');
                            self.process(Token::Symbol('.'));

                            // parse subroutineName
                            match self.tokenizer.next() {
                                Some(Token::Identifier(ident)) => {
                                    subroutine_name.push_str(ident);
                                    println!("<identifier>");
                                    println!("<name>{ident}</name>");
                                    println!("<category>subroutine</category>");
                                    println!("<usage>used</usage>");
                                    println!("</identifier>");
                                }
                                _ => println!("error: syntax error"),
                            }
                        } else {
                            // subroutine() which is implicitly this.subroutine()
                            subroutine_name = format!("{}.{}", self.class_name, ident);
                            self.vm_writer.write_push(Segment::POINTER, 0);
                            n_args += 1;
                            println!("<category>subroutine</category>");
                            println!("</identifier>");
                        }

                        self.process(Token::Symbol('('));
                        if let (Some(segment), Some(index)) = (segment, index) {
                            self.vm_writer.write_push(segment, index);
                            n_args += 1;
                        }
                        n_args += self.compile_expression_list();
                        self.vm_writer.write_call(subroutine_name.as_str(), n_args);
                        self.process(Token::Symbol(')'));
                    }
                    _ => {
                        if let (Some(segment), Some(index)) = (segment, index) {
                            self.vm_writer.write_push(segment, index);
                            println!("<category>{segment}</category>");
                        } else {
                            println!("<category>error</category>");
                        }
                        println!("</identifier>");
                    }
                }
            }
            Some(s @ Token::Symbol('-' | '~')) => {
                println!("{s}");
                if let Some(Token::Symbol(op)) = self.tokenizer.next() {
                    self.compile_term();
                    match op {
                        '-' => self.vm_writer.write_arithmetic(Arithmetic::NEG),
                        '~' => self.vm_writer.write_arithmetic(Arithmetic::NOT),
                        _ => {}
                    }
                }
            }
            _ => println!("error: syntax error"),
        }
        println!("</term>");
    }

    pub fn compile_expression_list(&mut self) -> usize {
        println!("<expressionList>");
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
        println!("</expressionList>");
        return count;
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
