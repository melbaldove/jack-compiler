use std::fs;

use jack_compiler::tokenizer::{Token, Tokenizer};

// fn main() {
//     let mut args = std::env::args();
//     args.next();
//
//     let path = args
//         .next()
//         .map_or_else(|| PathBuf::from("."), |arg| PathBuf::from(arg));
//     let file_stem = path.file_stem().and_then(|x| x.to_str()).unwrap();
// }

fn main() {
    let file_contents = fs::read_to_string("samples/ExpressionLessSquare/Main.jack").unwrap();

    let mut tokenizer = Tokenizer::build(file_contents.as_str()).unwrap();
    println!("Tokenizing...");
    for token in tokenizer {
        match token {
            Token::Whitespace(_) => {}
            token => {
                println!("{:?}", token);
            }
        }
    }
}
