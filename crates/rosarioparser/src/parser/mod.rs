use crate::{
    lexer::{Lexer, Token, TokenType},
    parser::ast::{
        Ast, BinOpType, DefinitionOwner, DefinitionSignature, ExpressionId, Generic, GenericEnd,
        Range, RosarioType, TypeBody, TypeSignature, Variable, VariableType,
    },
};

pub mod ast;

#[derive(Debug, Default, Clone)]
pub struct Parser {
    pub lexer: Lexer,
    current_item: usize,

    pub ast: Ast,
    pub current_def_sig: Option<DefinitionSignature>,
}

impl Parser {
    pub fn start(&mut self, lex: Lexer) {
        self.lexer = lex;

        self.lexer.start();
        self.advance();

        while self.current_token().ty != TokenType::EndOfFile {
            match self.current_token().ty {
                TokenType::Procedure => self.parse_procedure(),
                TokenType::Type => self.parse_type(),
                _ => todo!("{:?}", self.current_token().ty),
            }

            self.advance();
        }
    }

    pub fn parse_type(&mut self) {
        self.advance();

        let name = self.get_identifier().clone();

        self.advance();

        if self.current_token().ty != TokenType::Is {
            Self::expected_token_found_error("is", &format!("{:?}", self.current_token().ty));
        }

        self.advance();

        let ty = match self.current_token().ty {
            TokenType::Range => RosarioType {
                generics: vec![],
                ty: TypeBody::Range(self.parse_range()),
            },
            _ => todo!("{:?}", self.current_token().ty),
        };

        self.advance();

        if self.current_token().ty != TokenType::Semicolon {
            Self::expected_token_found_error(";", &format!("{:?}", self.current_token().ty));
        }

        self.ast.new_type(name, self.file_path_to_rosario(), ty);
    }

    pub fn parse_range(&mut self) -> Range {
        self.advance();

        let min = self.parse_expression(None, None);

        self.advance();

        if self.current_token().ty != TokenType::RangeDots {
            Self::expected_token_found_error("..", &format!("{:?}", self.current_token().ty));
        }

        self.advance();

        let max = self.parse_expression(None, None);

        Range { min, max }
    }

    pub fn parse_definition_signature(&mut self) -> DefinitionSignature {
        let name = self.get_identifier().clone();

        self.advance();

        if self.current_token().ty != TokenType::Character('(') {
            todo!("Expected '(' Error.");
        }

        self.advance();

        while self.current_token().ty != TokenType::Character(')') {
            // TODO: Arguments.
            self.advance();
        }

        self.advance();

        let return_type = if self.current_token().ty == TokenType::Arrow {
            self.advance();

            Some(self.parse_type_signature())
        } else {
            self.go_back();

            None
        };

        DefinitionSignature {
            owner: DefinitionOwner::Path(self.file_path_to_rosario()),
            name,
            args: vec![],
            return_type,
        }
    }

    pub fn file_path_to_rosario(&self) -> String {
        format!("{}::{}", self.lexer.main_rosario_path, self.lexer.file_name)
    }

    pub fn parse_type_signature(&mut self) -> TypeSignature {
        let name = self.get_identifier().clone();

        TypeSignature {
            owner: self.ast.find_type_signature_owner(name.clone()),
            name,
        }
    }

    pub fn expected_token_found_error(expected: &str, found: &str) -> ! {
        panic!(
            "Expected Token `{}`. Found `{}` instead...",
            expected, found
        )
    }

    pub fn parse_procedure(&mut self) {
        self.advance();

        let signature = self.parse_definition_signature();

        self.current_def_sig = Some(signature.clone());

        self.advance();

        if self.current_token().ty != TokenType::Is {
            Self::expected_token_found_error("is", &format!("{:?}", self.current_token().ty));
        }

        self.advance();

        let body = self.parse_expression(Some(signature.name.clone()), None);

        self.advance();

        if self.current_token().ty != TokenType::Semicolon {
            Self::expected_token_found_error(";", &format!("{:?}", self.current_token().ty));
        }

        self.current_def_sig = None;

        self.ast.new_definition(signature, body);
    }

    pub fn parse_let(&mut self) -> ExpressionId {
        self.advance();

        let name = self.get_identifier().clone();

        self.advance();

        if self.current_token().ty != TokenType::Colon {
            Self::expected_token_found_error(":", &format!("{:?}", self.current_token().ty));
        }

        self.advance();

        let ty = self.parse_type_signature();

        self.advance();

        let generics = self.parse_generics();

        self.advance();

        let initializer = if self.current_token().ty == TokenType::Equals {
            self.advance();
            Some(self.parse_expression(Some(name.clone()), None))
        } else {
            None
        };

        self.ast.new_variable(
            self.current_def_sig.clone().unwrap(),
            Variable {
                name,
                ty,
                variable_type: VariableType::Value,
                generics,
                initializer,
            },
        )
    }

    pub fn parse_generics(&mut self) -> Vec<Generic> {
        if self.current_token().ty != TokenType::MoreThan {
            self.go_back();
            return vec![];
        }

        self.advance();

        let mut result = vec![];

        while self.current_token().ty != TokenType::MoreThan {
            let generic_name = self.get_identifier().clone();

            self.advance();

            let generic_end = match self.current_token().ty {
                TokenType::Character(',') => GenericEnd::Comma,
                TokenType::Of => GenericEnd::Of,
                TokenType::MoreThan => GenericEnd::Nothing,
                _ => Self::expected_token_found_error(
                    ",` or `of",
                    &format!("{:?}", self.current_token().ty),
                ),
            };

            result.push(Generic {
                name: generic_name,
                end: generic_end,
            });

            if self.current_token().ty != TokenType::MoreThan {
                self.advance();
            }
        }

        result
    }

    pub fn bin_op_token_to_type(&self) -> BinOpType {
        match self.current_token().ty {
            TokenType::Add => BinOpType::Add,
            TokenType::Sub => BinOpType::Sub,
            TokenType::Multiply => BinOpType::Mul,
            TokenType::Divide => BinOpType::Div,
            TokenType::ToThePowerOf => BinOpType::ToThePowerOf,
            _ => todo!("{:?}", self.current_token().ty),
        }
    }

    pub fn parse_binary_operator(&mut self, left: ExpressionId) -> ExpressionId {
        let current_bin_op = self.bin_op_token_to_type();

        self.advance();

        let right = self.parse_expression(None, None);

        let right_bin_op = match self.ast.find_expression(right) {
            Some(r) => r.get_binary_operator(),
            None => None,
        };

        let right_op_left_value = match right_bin_op {
            Some(bin_op) => {
                if bin_op.op > current_bin_op {
                    ExpressionId(0)
                } else {
                    bin_op.left
                }
            }
            None => ExpressionId(0),
        };

        let new_right_op_left_value = if right_op_left_value != ExpressionId(0) {
            self.ast
                .new_binary_operator(current_bin_op.clone(), left, right_op_left_value)
        } else {
            ExpressionId(0)
        };

        let mut_right_bin_op = match self.ast.find_mut_expression(right) {
            Some(r) => r.get_mut_binary_operator(),
            None => None,
        };

        match mut_right_bin_op {
            Some(bin_op) => {
                if bin_op.op > current_bin_op {
                    return self.ast.new_binary_operator(current_bin_op, left, right);
                } else {
                    bin_op.left = new_right_op_left_value;
                    return right;
                }
            }
            None => {
                return self.ast.new_binary_operator(current_bin_op, left, right);
            }
        }
    }

    pub fn parse_expression(
        &mut self,
        end_name: Option<String>,
        left: Option<ExpressionId>,
    ) -> ExpressionId {
        let expr = match &self.current_token().ty {
            TokenType::Begin => self.parse_body(end_name.clone()),
            TokenType::Let => self.parse_let(),
            TokenType::Number(natural, decimal) => self.ast.new_number(
                natural.parse::<u128>().unwrap(),
                match decimal {
                    Some(d) => Some(d.parse::<u128>().unwrap()),
                    None => None,
                },
            ),
            TokenType::Add
            | TokenType::Sub
            | TokenType::Multiply
            | TokenType::Divide
            | TokenType::ToThePowerOf => self.parse_binary_operator(left.unwrap()),
            _ => todo!("{:?}", self.current_token().ty),
        };

        self.advance();

        if self.is_token_delimiter() {
            self.go_back();
            return expr;
        }

        self.parse_expression(end_name, Some(expr))
    }

    pub fn is_token_delimiter(&self) -> bool {
        match self.current_token().ty {
            TokenType::Semicolon | TokenType::RangeDots => true,
            _ => false,
        }
    }

    pub fn parse_body(&mut self, end_name: Option<String>) -> ExpressionId {
        let mut contents = vec![];

        self.advance();

        while self.current_token().ty != TokenType::End
            && self.current_token().ty != TokenType::EndOfFile
        {
            contents.push(self.parse_expression(None, None));

            self.advance();

            if !matches!(self.current_token().ty, TokenType::Semicolon) {
                todo!("Semicolon Error.");
            }

            self.advance();
        }

        self.parse_ending(end_name);

        self.ast.new_body(contents)
    }

    pub fn parse_ending(&mut self, end_name: Option<String>) {
        if !matches!(self.current_token().ty, TokenType::End) {
            todo!("'end' Token Error.");
        }

        match end_name {
            Some(name) => {
                self.advance();

                if *self.get_identifier() != name {
                    todo!("End Name Error.")
                }
            }
            None => {}
        }
    }

    pub fn get_identifier(&self) -> &String {
        match &self.current_token().ty {
            TokenType::Identifier(id) => &id,
            _ => todo!("Identifier Error."),
        }
    }

    pub fn advance(&mut self) -> &Token {
        self.current_item += 1;
        self.current_token()
    }

    pub fn go_back(&mut self) -> &Token {
        self.current_item -= 1;
        self.current_token()
    }

    pub fn current_token(&self) -> &Token {
        if self.current_item == 0 {
            panic!("Parser hasn't started yet.");
        }

        &self.lexer.contents[self.current_item - 1]
    }
}
