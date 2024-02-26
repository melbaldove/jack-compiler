use std::{
    fmt,
    fs::File,
    io::{Error, Write},
    path::PathBuf,
};

pub struct VmWriter {
    file: File,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Segment {
    CONSTANT,
    ARGUMENT,
    LOCAL,
    STATIC,
    THIS,
    THAT,
    POINTER,
    TEMP,
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant_str = match self {
            Segment::CONSTANT => "constant",
            Segment::ARGUMENT => "argument",
            Segment::LOCAL => "local",
            Segment::STATIC => "static",
            Segment::THIS => "this",
            Segment::THAT => "that",
            Segment::POINTER => "pointer",
            Segment::TEMP => "temp",
        };
        write!(f, "{}", variant_str)
    }
}

pub enum Arithmetic {
    ADD,
    SUB,
    NEG,
    EQ,
    GT,
    LT,
    AND,
    OR,
    NOT,
}

impl fmt::Display for Arithmetic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant_str = match self {
            Arithmetic::ADD => "add",
            Arithmetic::SUB => "sub",
            Arithmetic::NEG => "neg",
            Arithmetic::EQ => "eq",
            Arithmetic::GT => "gt",
            Arithmetic::LT => "lt",
            Arithmetic::AND => "and",
            Arithmetic::OR => "or",
            Arithmetic::NOT => "not",
        };
        write!(f, "{}", variant_str)
    }
}

impl VmWriter {
    pub fn build(path: PathBuf) -> Result<VmWriter, Error> {
        let file = File::create(&path)?;

        Ok(VmWriter { file })
    }

    pub fn write_push(&mut self, segment: Segment, index: usize) {
        self.writeln(&format!("push {segment} {index}"))
    }

    pub fn write_pop(&mut self, segment: Segment, index: usize) {
        self.writeln(&format!("pop {segment} {index}"))
    }

    pub fn write_arithmetic(&mut self, command: Arithmetic) {
        self.writeln(&format!("{command}"))
    }

    pub fn write_label(&mut self, label: &str) {
        self.writeln(&format!("label {label}"))
    }

    pub fn write_goto(&mut self, label: &str) {
        self.writeln(&format!("goto {label}"))
    }

    pub fn write_if(&mut self, label: &str) {
        self.writeln(&format!("if-goto {label}"))
    }

    pub fn write_call(&mut self, name: &str, n_args: usize) {
        self.writeln(&format!("call {name} {n_args}"))
    }

    pub fn write_function(&mut self, name: &str, n_vars: usize) {
        self.writeln(&format!("function {name} {n_vars}"))
    }

    pub fn write_return(&mut self) {
        self.writeln("return");
    }

    fn writeln(&mut self, str: &str) {
        let _ = self.file.write_all(format!("{}\n", str).as_bytes());
    }
}
