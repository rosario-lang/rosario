use rosarioc::CCompiler;
use rosarioparser::{lexer::Lexer, parser::Parser};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut lexer = Lexer::from_file(&args[1]);
    lexer.start();

    dbg!(&lexer.contents);

    let mut parser = Parser::new(lexer);
    parser.start(Some(rosarioparser::parse_core().result.packages));

    dbg!(&parser.result);

    let mut compiler = CCompiler::new(parser);

    std::fs::write(args[2].clone() + ".c", compiler.start()).unwrap();
}
