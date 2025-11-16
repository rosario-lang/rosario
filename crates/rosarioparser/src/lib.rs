pub mod ast;
pub mod lexer;
pub mod parser;

pub fn parse_core() -> parser::Parser {
    let mut lexer = lexer::Lexer::from_file("../../core/core.ros");
    lexer.start();

    dbg!(&lexer.contents);

    let mut parser = parser::Parser::new(lexer);
    parser.start(None);

    dbg!(&parser.result);

    parser
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, parser::Parser};

    #[test]
    fn it_works() {
        let mut lexer = Lexer::from_file("tests/match_vs_if.ros");
        lexer.start();

        dbg!(&lexer.contents);

        let mut parser = Parser::new(lexer);
        parser.start(None);

        dbg!(&parser.result);
    }

    #[test]
    fn core_parsing() {
        let mut lexer = Lexer::from_file("../../core/core.ros");
        lexer.start();

        dbg!(&lexer.contents);

        let mut parser = Parser::new(lexer);
        parser.start(None);

        dbg!(&parser.result);
    }
}
