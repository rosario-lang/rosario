use std::{collections::HashMap, path::Path, sync::LazyLock};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    EndOfFile,
    Identifier(String),
    Character(char),
    Number(String, Option<String>),
    Procedure,
    Function,
    Is,
    Loop,
    For,
    Return,
    Semicolon,
    Begin,
    End,
    If,
    ElsIf,
    Else,
    Then,
    StaticString(String),
    Do,
    Mutable,
    Let,
    Add,
    Sub,
    Multiply,
    Divide,
    Equals,
    Not,
    LessThan,
    MoreThan,
    IsEquals,
    IsNotEquals,
    AddEquals,
    SubEquals,
    MultiplyEquals,
    DivideEquals,
    LessThanOrEquals,
    MoreThanOrEquals,
    Match,
    Of,
    Arrow,
    Type,
    Range,
    RangeDots,
    ToThePowerOf,
    Dot,
    Modulo,
    Public,
    Use,
    Package,
    Colon,
    DoubleColon,
    Enum,
    Comment,
    Implement,
    SelfVariable,
    Operator,
    Trait,
    Clang,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub begin_location: (usize, usize),
    pub location_size: usize,
    pub ty: TokenType,
}

pub static SYMBOL_LIST: LazyLock<HashMap<&str, TokenType>> = LazyLock::new(|| {
    HashMap::from([
        ("=", TokenType::Equals),
        ("+", TokenType::Add),
        ("-", TokenType::Sub),
        ("*", TokenType::Multiply),
        ("/", TokenType::Divide),
        ("<", TokenType::LessThan),
        (">", TokenType::MoreThan),
        (".", TokenType::Dot),
        (":", TokenType::Colon),
        ("!", TokenType::Not),
        ("+=", TokenType::AddEquals),
        ("-=", TokenType::SubEquals),
        ("*=", TokenType::MultiplyEquals),
        ("/=", TokenType::DivideEquals),
        ("<=", TokenType::LessThanOrEquals),
        (">=", TokenType::MoreThanOrEquals),
        ("==", TokenType::IsEquals),
        ("!=", TokenType::IsNotEquals),
        ("=>", TokenType::Arrow),
        ("..", TokenType::RangeDots),
        ("**", TokenType::ToThePowerOf),
        ("::", TokenType::DoubleColon),
        ("--", TokenType::Comment),
    ])
});

pub static IDENTIFIER_LIST: LazyLock<HashMap<&str, TokenType>> = LazyLock::new(|| {
    HashMap::from([
        ("procedure", TokenType::Procedure),
        ("function", TokenType::Function),
        ("is", TokenType::Is),
        ("loop", TokenType::Loop),
        ("for", TokenType::For),
        ("return", TokenType::Return),
        ("begin", TokenType::Begin),
        ("end", TokenType::End),
        ("if", TokenType::If),
        ("elsif", TokenType::ElsIf),
        ("else", TokenType::Else),
        ("then", TokenType::Then),
        ("do", TokenType::Do),
        ("mutable", TokenType::Mutable),
        ("let", TokenType::Let),
        ("match", TokenType::Match),
        ("of", TokenType::Of),
        ("type", TokenType::Type),
        ("range", TokenType::Range),
        ("mod", TokenType::Modulo),
        ("public", TokenType::Public),
        ("use", TokenType::Use),
        ("package", TokenType::Package),
        ("enum", TokenType::Enum),
        ("implement", TokenType::Implement),
        ("self", TokenType::SelfVariable),
        ("operator", TokenType::Operator),
        ("trait", TokenType::Trait),
        ("C_LANG", TokenType::Clang),
    ])
});

#[derive(Default, Debug, Clone)]
pub struct Lexer {
    input: Vec<u8>,
    pub contents: Vec<Token>,
    position: usize,
    last_line: usize,
    location: (usize, usize),
    pub file_path: String,
    pub folder_path: String,
    pub file_name: String,
}

impl Lexer {
    pub fn from(input: Vec<u8>, path: String) -> Self {
        let path_as_path = Path::new(&path);

        // Yeah... I know...
        let file_path = path_as_path
            .canonicalize()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();

        let folder_path = path_as_path
            .canonicalize()
            .unwrap()
            .ancestors()
            .nth(1)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let mut file_name = path_as_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            .replace(".ros", "");

        if file_name == "module" {
            file_name = Path::new(&folder_path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
        }

        Self {
            input,
            file_path,
            folder_path,
            file_name,
            ..Default::default()
        }
    }

    pub fn from_file(path: &str) -> Self {
        Lexer::from(std::fs::read(path).unwrap(), path.to_string())
    }

    pub fn advance(&mut self) -> u8 {
        self.position += 1;

        let result = self.current_char();
        self.location.1 += 1;

        if result == b'\n' {
            self.last_line = self.location.1;
            self.location = (self.location.0 + 1, 0);
        }

        result
    }

    pub fn go_back(&mut self) -> u8 {
        self.position -= 1;

        let result = self.current_char();
        let sub = self.location.1.checked_sub(1);

        match sub {
            Some(s) => self.location.1 = s,
            None => self.location = (self.location.0 - 1, self.last_line),
        };

        result
    }

    pub fn current_char(&self) -> u8 {
        if self.position - 1 >= self.input.len() {
            return b'\0';
        }

        self.input[self.position - 1]
    }

    pub fn get_token(&mut self) {
        let mut char = self.advance();

        loop {
            if char != b' ' && char != b'\n' && char != b'\t' {
                break;
            }

            char = self.advance();
        }

        if char.is_ascii_digit() {
            return self.get_number();
        }

        if char == b'\"' {
            return self.get_static_string();
        }

        if char.is_ascii_alphabetic() {
            return self.get_identifier();
        }

        if char == b';' {
            return self.contents.push(Token {
                begin_location: self.location,
                location_size: 1,
                ty: TokenType::Semicolon,
            });
        }

        if char == b'\0' {
            return self.contents.push(Token {
                begin_location: self.location,
                location_size: 0,
                ty: TokenType::EndOfFile,
            });
        }

        match SYMBOL_LIST.get(String::from_utf8(vec![char]).unwrap().as_str()) {
            Some(token) => {
                let second_char = self.advance();
                match SYMBOL_LIST.get(String::from_utf8(vec![char, second_char]).unwrap().as_str())
                {
                    Some(second_token) => {
                        if *second_token == TokenType::Comment {
                            while char != b'\n' && char != b'\0' {
                                char = self.advance();
                            }

                            return;
                        }

                        return self.contents.push(Token {
                            begin_location: self.location,
                            location_size: 2,
                            ty: second_token.clone(),
                        });
                    }
                    None => {
                        self.go_back();

                        return self.contents.push(Token {
                            begin_location: self.location,
                            location_size: 1,
                            ty: token.clone(),
                        });
                    }
                }
            }
            None => {}
        }

        return self.contents.push(Token {
            begin_location: self.location,
            location_size: 1,
            ty: TokenType::Character(char as char),
        });
    }

    pub fn get_static_string(&mut self) {
        let begin_location = self.location;
        let start_position = self.position;

        let mut char = self.advance();
        let mut result = String::new();
        while char != b'\"' {
            result.push(char as char);
            char = self.advance();
        }

        self.contents.push(Token {
            begin_location,
            location_size: self.position - start_position,
            ty: TokenType::StaticString(result),
        });
    }

    pub fn get_identifier(&mut self) {
        let mut char = self.current_char();
        while !char.is_ascii_alphanumeric() {
            char = self.advance();
        }

        let start_location = self.location;
        let start_position = self.position;

        let mut result_string = String::new();
        while char.is_ascii_alphanumeric() || char == b'_' {
            result_string.push(char as char);
            char = self.advance();
        }

        let token_ty = match IDENTIFIER_LIST.get(result_string.as_str()) {
            Some(string) => string.clone(),
            None => TokenType::Identifier(result_string),
        };

        self.contents.push(Token {
            begin_location: start_location,
            location_size: self.position - start_position,
            ty: token_ty,
        });

        self.position -= 1;
    }

    pub fn get_number(&mut self) {
        let mut char = self.current_char();
        let mut result = (vec![char], vec![]);

        let start_location = self.location;
        let start_position = self.position;

        char = self.advance();

        let mut decimal = false;
        while char.is_ascii_digit() || char == b'_' {
            if char == b'_' {
                char = self.advance();
                continue;
            }

            if char != b'.' {
                match decimal {
                    false => result.0.push(char),
                    true => result.1.push(char),
                };
            } else {
                decimal = true;
            }

            char = self.advance();
        }

        self.go_back();

        self.contents.push(Token {
            begin_location: start_location,
            location_size: self.position - start_position,
            ty: TokenType::Number(
                String::from_utf8(result.0).unwrap(),
                if result.1.len() != 0 {
                    Some(String::from_utf8(result.1).unwrap())
                } else {
                    None
                },
            ),
        });
    }

    pub fn start(&mut self) {
        while self.position < self.input.len() {
            self.get_token();
        }

        if self.contents.last().unwrap().ty != TokenType::EndOfFile {
            self.contents.push(Token {
                begin_location: self.location,
                location_size: 0,
                ty: TokenType::EndOfFile,
            })
        }
    }
}
