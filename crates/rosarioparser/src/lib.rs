pub mod lexer;
pub mod parser;

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, parser::Parser};

    #[test]
    fn local_variables() {
        let mut parser = Parser::default();

        parser.start(Lexer::from_file("tests/local_variables.ros", None));

        dbg!(parser.ast);
    }
}
