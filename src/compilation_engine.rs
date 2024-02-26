use std::{io::Error, iter::Peekable, path::PathBuf};

use crate::{
    symbol_table::{Category, SymbolTable},
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
            Some(current_token) if current_token == token => return,
            None => return,
            Some(_) => println!("error: syntax error"),
        }
    }

    pub fn compile_class(&mut self) {
        // parse class
        self.process(Token::Keyword("class"));

        // parse className
        match self.tokenizer.next() {
            Some(Token::Identifier(class_name)) => self.class_name.push_str(class_name),
            _ => println!("error: syntax error"),
        }

        // parse body
        self.process(Token::Symbol('{'));
        while let Some(Token::Keyword("static" | "field")) = self.tokenizer.peek() {
            self.compile_class_var_dec();
        }
        while let Some(Token::Keyword("constructor" | "function" | "method")) =
            self.tokenizer.peek()
        {
            self.compile_subroutine();
        }
        self.process(Token::Symbol('}'));
    }

    pub fn compile_class_var_dec(&mut self) {
        // parse static | field
        let category = match self.tokenizer.next() {
            Some(Token::Keyword(k @ ("static" | "field"))) => Some(k),
            _ => {
                println!("error: syntax error");
                None
            }
        };

        // parse type
        let _type = match self.tokenizer.next() {
            Some(Token::Keyword(k @ ("int" | "char" | "boolean")) | Token::Identifier(k)) => {
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
                let kind = category.and_then(|category| get_category(category));
                let _type = _type;
                if let (Some(kind), Some(_type)) = (kind, _type) {
                    self.class_symbol_table.define(var_name, _type, kind);
                }
            }
            _ => println!("error: syntax error"),
        }

        // parse delimiter whether , or ;
        while let Some(token) = self.tokenizer.next() {
            match token {
                Token::Symbol(',') => {
                    if let Some(Token::Identifier(var_name)) = self.tokenizer.next() {
                        let kind = category.and_then(|category| get_category(category));
                        let _type = _type;
                        if let (Some(kind), Some(_type)) = (kind, _type) {
                            self.class_symbol_table.define(var_name, _type, kind);
                        }
                    } else {
                        println!("error: syntax error");
                    }
                }
                Token::Symbol(';') => {
                    break;
                }
                _ => {
                    println!("error: syntax error");
                    break;
                }
            }
        }
    }

    pub fn compile_subroutine(&mut self) {
        self.subroutine_symbol_table.reset();
        self.subroutine_name.clear();
        self.subroutine_category.clear();
        self.subroutine_type.clear();
        // parse constructor | function | method
        match self.tokenizer.next() {
            Some(Token::Keyword(c @ ("constructor" | "function" | "method"))) => {
                self.subroutine_category.push_str(c)
            }
            _ => {
                println!("error: syntax error");
            }
        };
        if self.subroutine_category == "method" {
            self.subroutine_symbol_table
                .define("this", self.class_name.as_str(), Category::Arg);
        }

        // parse type
        match self.tokenizer.next() {
            Some(
                Token::Keyword(_type @ ("void" | "int" | "char" | "boolean"))
                | Token::Identifier(_type),
            ) => self.subroutine_type.push_str(_type),
            _ => println!("error: syntax error"),
        }

        // parse subroutineName
        match self.tokenizer.next() {
            Some(Token::Identifier(i)) => self.subroutine_name.push_str(i),
            _ => println!("error: syntax error"),
        };

        self.process(Token::Symbol('('));
        self.compile_parameter_list();
        self.process(Token::Symbol(')'));
        self.compile_subroutine_body();
    }

    pub fn compile_parameter_list(&mut self) {
        // peek if its )
        // if its not, process the parameter list
        while let Some(token) = self.tokenizer.peek() {
            match *token {
                Token::Symbol(')') => break,
                Token::Keyword(t @ ("int" | "char" | "boolean")) | Token::Identifier(t) => {
                    // parse type and advance the iterator
                    self.tokenizer.next();

                    // parse varName
                    match self.tokenizer.next() {
                        Some(Token::Identifier(arg_name)) => {
                            self.subroutine_symbol_table
                                .define(arg_name, t, Category::Arg)
                        }
                        _ => println!("error: syntax error"),
                    }
                }
                Token::Symbol(',') => {
                    self.tokenizer.next();
                }
                _ => println!("error: syntax error"),
            }
        }
    }

    pub fn compile_subroutine_body(&mut self) {
        self.process(Token::Symbol('{'));
        // loop
        while let Some(Token::Keyword("var")) = self.tokenizer.peek() {
            self.compile_var_dec();
        }
        let n_vars = self.subroutine_symbol_table.var_count(Category::Var);
        self.vm_writer.write_function(
            &format!("{}.{}", self.class_name, self.subroutine_name),
            n_vars,
        );

        if self.subroutine_category == "constructor" {
            self.vm_writer.write_push(
                Segment::CONSTANT,
                self.class_symbol_table.var_count(Category::Field),
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
    }

    pub fn compile_var_dec(&mut self) {
        self.process(Token::Keyword("var"));
        // parse type
        if let Some(
            Token::Keyword(_type @ ("int" | "char" | "boolean")) | Token::Identifier(_type),
        ) = self.tokenizer.next()
        {
            // parse varName
            match self.tokenizer.next() {
                Some(Token::Identifier(var_name)) => {
                    self.subroutine_symbol_table
                        .define(var_name, _type, Category::Var)
                }
                _ => println!("error: syntax error"),
            }

            // parse delimiter whether , or ;
            while let Some(Token::Symbol(',')) = self.tokenizer.next() {
                if let Some(Token::Identifier(var_name)) = self.tokenizer.next() {
                    self.subroutine_symbol_table
                        .define(var_name, _type, Category::Var);
                }
            }
        }
    }

    pub fn compile_statements(&mut self) {
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
    }

    pub fn compile_let(&mut self) {
        // parse let
        self.process(Token::Keyword("let"));

        // parse varName
        match self.tokenizer.next() {
            Some(Token::Identifier(var_name)) => {
                let segment = self.kind_of(var_name).map(category_to_segment);
                let _type = self.type_of(var_name);
                let index = self.index_of(var_name);
                if let (Some(segment), Some(index)) = (segment, index) {
                    if let Some(Token::Symbol('[')) = self.tokenizer.peek() {
                        self.vm_writer.write_push(segment, index);
                        self.process(Token::Symbol('['));
                        self.compile_expression();
                        self.vm_writer.write_arithmetic(Arithmetic::ADD);
                        self.process(Token::Symbol(']'));

                        self.process(Token::Symbol('='));

                        self.compile_expression();
                        self.vm_writer.write_pop(Segment::TEMP, 0);
                        // pops to arr + i
                        self.vm_writer.write_pop(Segment::POINTER, 1);
                        self.vm_writer.write_push(Segment::TEMP, 0);
                        self.vm_writer.write_pop(Segment::THAT, 0);
                    } else {
                        self.process(Token::Symbol('='));
                        self.compile_expression();
                        self.vm_writer.write_pop(segment, index);
                    }
                }
                self.process(Token::Symbol(';'));
            }
            _ => println!("error: syntax error"),
        };
    }

    pub fn compile_if(&mut self) {
        let else_label = &format!("{}L{}", self.class_name, self.control_counter);
        self.control_counter += 1;
        let exit_label = &format!("{}L{}", self.class_name, self.control_counter);
        self.control_counter += 1;
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
    }

    fn generate_control_label(&mut self) -> String {
        let label = format!("{}L{}", self.class_name, self.control_counter);
        self.control_counter += 1;
        label
    }

    pub fn compile_while(&mut self) {
        let loop_label = &self.generate_control_label();
        let exit_label = &self.generate_control_label();
        self.control_counter += 1;
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
    }

    pub fn compile_do(&mut self) {
        // parse do
        self.process(Token::Keyword("do"));
        self.compile_term();
        self.vm_writer.write_pop(Segment::TEMP, 0);
        self.process(Token::Symbol(';'));
    }

    pub fn compile_return(&mut self) {
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
    }

    pub fn compile_expression(&mut self) {
        self.compile_term();
        while let Some(token) = self.tokenizer.peek() {
            match *token {
                Token::Symbol('+' | '-' | '*' | '/' | '&' | '|' | '<' | '>' | '=') => {
                    if let Some(Token::Symbol(op)) = self.tokenizer.next() {
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
    }

    pub fn compile_term(&mut self) {
        match self.tokenizer.next() {
            Some(Token::IntConstant(c)) => {
                self.vm_writer.write_push(Segment::CONSTANT, c);
            }
            Some(Token::StringConst(c)) => {
                self.vm_writer.write_push(Segment::CONSTANT, c.len());
                self.vm_writer.write_call("String.new", 1);
                for char in c.bytes() {
                    self.vm_writer.write_push(Segment::CONSTANT, char.into());
                    self.vm_writer.write_call("String.appendChar", 2);
                }
            }

            Some(Token::Keyword(c @ ("true" | "false" | "null" | "this"))) => match c {
                "null" | "false" => self.vm_writer.write_push(Segment::CONSTANT, 0),
                "true" => {
                    self.vm_writer.write_push(Segment::CONSTANT, 0);
                    self.vm_writer.write_arithmetic(Arithmetic::NOT)
                }
                "this" => self.vm_writer.write_push(Segment::POINTER, 0),
                _ => {}
            },
            Some(Token::Symbol('(')) => {
                self.compile_expression();
                self.process(Token::Symbol(')'));
            }
            Some(Token::Identifier(ident)) => {
                let mut subroutine_name = String::new();
                let index = self.index_of(ident);
                let _type = self.type_of(ident);
                let segment = self.kind_of(ident).map(category_to_segment);
                if let Some(_type) = _type {
                    subroutine_name.push_str(_type.as_str());
                } else {
                    subroutine_name.push_str(ident);
                }

                match self.tokenizer.peek() {
                    Some(Token::Symbol('[')) => {
                        if let (Some(segment), Some(index)) = (segment, index) {
                            self.vm_writer.write_push(segment, index);
                        }
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
                            subroutine_name.push('.');
                            self.process(Token::Symbol('.'));

                            // parse subroutineName
                            match self.tokenizer.next() {
                                Some(Token::Identifier(ident)) => {
                                    subroutine_name.push_str(ident);
                                }
                                _ => println!("error: syntax error"),
                            }
                        } else {
                            // subroutine() which is implicitly this.subroutine()
                            subroutine_name = format!("{}.{}", self.class_name, ident);
                            // push `this` as receiver
                            self.vm_writer.write_push(Segment::POINTER, 0);
                            n_args += 1;
                        }

                        self.process(Token::Symbol('('));
                        // push the receiver
                        if let (Some(segment), Some(index)) = (segment, index) {
                            self.vm_writer.write_push(segment, index);
                            n_args += 1;
                        }
                        n_args += self.compile_expression_list();
                        self.vm_writer.write_call(subroutine_name.as_str(), n_args);
                        self.process(Token::Symbol(')'));
                    }
                    _ => {
                        // variable
                        if let (Some(segment), Some(index)) = (segment, index) {
                            self.vm_writer.write_push(segment, index);
                        }
                    }
                }
            }
            Some(Token::Symbol(op @ ('-' | '~'))) => {
                self.compile_term();
                match op {
                    '-' => self.vm_writer.write_arithmetic(Arithmetic::NEG),
                    '~' => self.vm_writer.write_arithmetic(Arithmetic::NOT),
                    _ => {}
                }
            }
            _ => println!("error: syntax error"),
        }
    }

    pub fn compile_expression_list(&mut self) -> usize {
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
                Token::Symbol(',') => self.process(Token::Symbol(',')),
                _ => break,
            }
        }
        return count;
    }

    fn kind_of(&self, name: &str) -> Option<Category> {
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
fn get_category(str: &str) -> Option<Category> {
    match str {
        "field" => Some(Category::Field),
        "static" => Some(Category::Static),
        "var" => Some(Category::Var),
        "arg" => Some(Category::Arg),
        _ => None,
    }
}

fn category_to_segment(category: Category) -> Segment {
    match category {
        Category::Static => Segment::STATIC,
        Category::Field => Segment::THIS,
        Category::Arg => Segment::ARGUMENT,
        Category::Var => Segment::LOCAL,
    }
}
