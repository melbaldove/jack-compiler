use std::{fs, path::PathBuf};

use jack_compiler::{
    compilation_engine::CompilationEngine,
    tokenizer::{Token, Tokenizer},
};

fn main() {
    let mut args = std::env::args();
    args.next();
    let path = args
        .next()
        .map_or_else(|| PathBuf::from("."), |arg| PathBuf::from(arg));

    if path.is_dir() {
        let iter = path
            .read_dir()
            .expect("Expected to read_dir() successfully")
            .filter_map(|x| x.ok())
            .filter(|x| {
                x.path().is_file() && x.path().extension().and_then(|x| x.to_str()) == Some("jack")
            })
            .map(|x| x.path());

        for file_name in iter {
            compile(file_name);
        }
    } else {
        compile(path);
    }
}

fn compile(file_name: PathBuf) {
    let file_path = file_name.to_str().expect("Expected to_str() successfully");
    let file_name = file_name
        .file_name()
        .and_then(|x| x.to_str())
        .expect("Expected file_name() successfully")
        .strip_suffix(".jack")
        .expect("Expected strip successfully");

    let file = fs::read_to_string(&file_path).unwrap_or_else(|err| {
        eprintln!("ERROR: {}: {}", file_path, err);
        std::process::exit(2);
    });

    let tokenizer = Tokenizer::build(file.as_str()).unwrap().filter(|x| {
        !matches!(
            x,
            Token::Whitespace(_) | Token::SingleLineComment(_) | Token::BlockComment(_)
        )
    });

    let mut compilation_engine =
        CompilationEngine::build(tokenizer, PathBuf::from(format!("{file_name}.xml")))
            .unwrap_or_else(|err| {
                eprintln!("ERROR: {}: {}", file_path, err);
                std::process::exit(2);
            });

    compilation_engine.compile_class();
}
