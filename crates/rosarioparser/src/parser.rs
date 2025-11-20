use std::{collections::BTreeMap, path::Path};

use crate::{
    ast::{
        Argument, ArgumentType, Body, Condition, ConditionType, Enum, EnumArgument, Expression,
        Function, FunctionCall, Generic, GenericEnd, If, IsMutable, Let, Match, MatchOption,
        NewEnum, Operator, Package, ParsedResult, Procedure, Range, ReturnType, RosarioType,
        RosarioTypeContent, RosarioTypeImplementation, RosarioTypeSignature, Signature,
    },
    lexer::{Lexer, Token, TokenType},
};

pub struct Parser {
    token_pos: usize,
    pub lexer: Lexer,
    pub generic_results: BTreeMap<String, Package>,
    pub result: ParsedResult,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer,
            token_pos: 0,
            generic_results: BTreeMap::new(),
            result: ParsedResult::default(),
        }
    }

    pub fn current_token(&self) -> &Token {
        &self.lexer.contents[self.token_pos - 1]
    }

    pub fn parse_enum(&mut self, name: Option<String>, generics: &Vec<Generic>) -> Enum {
        self.token_pos += 1;

        let mut contents: BTreeMap<usize, (String, Vec<EnumArgument>)> = BTreeMap::new();
        let mut count = 0;
        while self.current_token().ty != TokenType::End {
            let name = self.get_identifier();

            self.token_pos += 1;

            if self.current_token().ty == TokenType::Character(',') {
                self.token_pos += 1;
                contents.insert(count, (name, vec![]));
                count += 1;
                continue;
            }

            let mut enum_arguments = vec![];
            if self.current_token().ty == TokenType::Character('(') {
                while self.current_token().ty != TokenType::Character(')') {
                    self.token_pos += 1;

                    let ty = self.parse_type_signature();

                    self.token_pos += 1;

                    if self.current_token().ty != TokenType::Character(',')
                        && self.current_token().ty != TokenType::Character(')')
                    {
                        panic!(
                            "Expected `,` or `)`. Found {:?} at {:?}",
                            self.current_token().ty,
                            self.current_token().begin_location
                        );
                    }

                    let mut generic_found = false;
                    'inner: for i in generics {
                        if i.name == ty.name {
                            enum_arguments.push(EnumArgument::Generic(ty.name.clone()));
                            generic_found = true;
                            break 'inner;
                        }
                    }

                    if !generic_found {
                        enum_arguments.push(EnumArgument::Type(ty));
                    }
                }

                self.check_token(TokenType::Character(')'));

                self.token_pos += 1;
            }

            if self.current_token().ty == TokenType::Character(',') {
                self.token_pos += 1;
            }

            contents.insert(count, (name, enum_arguments));
            count += 1;
        }

        self.check_ending(name, None);

        Enum { contents }
    }

    pub fn parse_type_signature(&mut self) -> RosarioTypeSignature {
        let name = self.get_identifier();

        self.token_pos += 1;

        let mut generics = vec![];
        if self.current_token().ty == TokenType::LessThan {
            self.token_pos += 1;
            loop {
                let generic_name = self.get_identifier();

                self.token_pos += 1;

                let generic_end = match self.current_token().ty {
                    TokenType::MoreThan => GenericEnd::None,
                    TokenType::Character(',') => GenericEnd::Comma,
                    TokenType::Of => GenericEnd::Of,
                    _ => panic!(
                        "Expected `>`, `,` or `of` inside of generic. Found {:?} at {:?}",
                        self.current_token().ty,
                        self.current_token().begin_location
                    ),
                };

                generics.push(Generic {
                    name: generic_name,
                    ends_with: generic_end,
                });

                if self.current_token().ty == TokenType::MoreThan {
                    break;
                }

                self.token_pos += 1;
            }
        } else {
            self.token_pos -= 1;
        }

        RosarioTypeSignature { name, generics }
    }

    pub fn parse_type(
        &mut self,
        public_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
        local_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
    ) -> (RosarioTypeSignature, RosarioType) {
        self.token_pos += 1;

        let signature = self.parse_type_signature();

        self.token_pos += 1;

        self.check_token(TokenType::Is);

        self.token_pos += 1;

        let content = match &self.current_token().ty {
            TokenType::Range => {
                RosarioTypeContent::Range(self.parse_range(public_types, local_types))
            }
            TokenType::Modulo => RosarioTypeContent::Modulo({
                self.token_pos += 1;
                self.parse_expression(None, None, public_types, local_types)
            }),
            TokenType::Enum => RosarioTypeContent::Enum(
                self.parse_enum(Some(signature.name.clone()), &signature.generics),
            ),
            TokenType::Identifier(_) => RosarioTypeContent::TypeRef(self.parse_type_signature()),
            _ => todo!("{:?}", self.current_token().ty),
        };

        self.token_pos += 1;

        self.check_token(TokenType::Semicolon);

        (
            signature,
            RosarioType {
                traits: vec![],
                content,
            },
        )
    }

    pub fn is_expression_signed(expr: &Expression) -> bool {
        match expr {
            Expression::Negative(_) => true,
            _ => false,
        }
    }

    pub fn parse_range(
        &mut self,
        public_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
        local_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
    ) -> Range {
        self.token_pos += 1;

        let left = self.parse_expression(None, None, public_types, local_types);

        let is_signed = Self::is_expression_signed(&left);

        self.token_pos += 1;

        self.check_token(TokenType::RangeDots);

        self.token_pos += 1;

        let right = self.parse_expression(None, None, public_types, local_types);

        Range {
            is_signed,
            left,
            right,
        }
    }

    pub fn main_pass(&mut self) {
        self.token_pos += 1;

        let mut main = Package::default();
        let mut is_public = false;

        while self.current_token().ty != TokenType::EndOfFile {
            match self.current_token().ty {
                TokenType::Procedure => {
                    let procedure =
                        self.parse_procedure(&vec![], &main.file.public_types, &main.file.types);
                    match is_public {
                        false => main.file.procedures.push(procedure),
                        true => main.file.public_procedures.push(procedure),
                    };

                    is_public = false;
                }
                TokenType::Function => {
                    let function =
                        self.parse_function(&vec![], &main.file.public_types, &main.file.types);
                    match is_public {
                        false => main.file.functions.push(function),
                        true => main.file.public_functions.push(function),
                    };

                    is_public = false;
                }
                TokenType::EndOfFile => {
                    return;
                }
                TokenType::Type => {
                    let ty = self.parse_type(&main.file.public_types, &main.file.types);

                    match is_public {
                        false => main.file.types.insert(ty.0, ty.1),
                        true => main.file.public_types.insert(ty.0, ty.1),
                    };

                    is_public = false;
                }
                TokenType::Public => {
                    is_public = true;
                }
                TokenType::Package => {
                    self.token_pos += 1;

                    let name = self.get_identifier();

                    let path = Path::new(&self.lexer.folder_path);
                    let path = path.join(&(name.clone() + ".ros"));

                    let mut lexer = Lexer::from_file(path.to_str().unwrap());
                    lexer.start();

                    let mut parser = Parser::new(lexer);
                    parser.start(Some(self.result.packages.clone()));

                    'outer: for (key, value) in parser.result.packages {
                        for (key_two, _) in &self.result.packages {
                            if key == *key_two {
                                continue 'outer;
                            }
                        }

                        let final_key = if !key.starts_with(&self.lexer.file_name) {
                            println!(
                                "{} is not equals to {}",
                                &key,
                                format!("{}::{}", self.lexer.file_name, key)
                            );
                            format!("{}::{}", self.lexer.file_name, key)
                        } else {
                            key
                        };

                        self.result.packages.insert(final_key, value);
                    }

                    self.token_pos += 1;

                    self.check_token(TokenType::Semicolon);
                }
                TokenType::Use => {
                    self.token_pos += 1;

                    if self.current_token().ty == TokenType::Clang {
                        self.token_pos += 1;

                        self.check_token(TokenType::Character('('));

                        self.token_pos += 1;

                        let path = match &self.current_token().ty {
                            TokenType::StaticString(string) => string.clone(),
                            _ => panic!(
                                "Expected a static string. Found {:?} at {:?}",
                                self.current_token().ty,
                                self.current_token().begin_location
                            ),
                        };

                        self.token_pos += 1;

                        self.check_token(TokenType::Character(')'));

                        main.c_includes.push(path);

                        self.token_pos += 1;
                    } else {
                        let mut result = String::new();
                        let mut grab_all = false;
                        while self.current_token().ty != TokenType::Semicolon {
                            let mut name = String::new();

                            match &self.current_token().ty {
                                TokenType::Identifier(id) => name = id.clone(),
                                TokenType::Multiply => grab_all = true,
                                _ => todo!("{:?}", self.current_token().ty),
                            }

                            self.token_pos += 1;

                            if grab_all {
                                break;
                            }

                            self.check_token(TokenType::DoubleColon);

                            self.token_pos += 1;

                            if !name.is_empty() {
                                result += &(name + "::");
                            }
                        }

                        result.pop();
                        result.pop();

                        let mut found_external_package = false;
                        for (key, _) in &self.result.packages {
                            if *key == result {
                                found_external_package = true;
                                break;
                            }
                        }

                        if !found_external_package {
                            result = self.lexer.file_name.clone() + "::" + &result;
                        }

                        main.packages.push((result, grab_all));
                    }

                    self.check_token(TokenType::Semicolon);
                }
                TokenType::Implement => {
                    let implem =
                        self.parse_implementation(&main.file.public_types, &main.file.types);
                    main.file.implementations.insert(implem.0, implem.1);
                }
                TokenType::Trait => {
                    self.token_pos += 1;

                    let name = self.get_identifier();

                    self.token_pos += 1;

                    let mut signatures = vec![];
                    if self.current_token().ty != TokenType::Semicolon {
                        self.check_token(TokenType::Is);

                        self.token_pos += 1;

                        while self.current_token().ty != TokenType::End {
                            let is_procedure = self.current_token().ty == TokenType::Procedure;

                            if !is_procedure && self.current_token().ty != TokenType::Function {
                                panic!(
                                    "Expected `procedure` or `function`. Found {:?} at {:?}",
                                    self.current_token().ty,
                                    self.current_token().begin_location
                                );
                            }

                            self.token_pos += 1;

                            signatures.push(self.parse_signature(is_procedure, &vec![]));

                            self.token_pos += 1;

                            self.check_token(TokenType::Semicolon);

                            self.token_pos += 1;
                        }

                        self.check_ending(Some(name), None);

                        self.token_pos += 1;

                        self.check_token(TokenType::Semicolon);
                    }
                }
                _ => todo!(
                    "{:?} at {:?}",
                    self.current_token().ty,
                    self.current_token().begin_location
                ),
            }

            self.token_pos += 1;
        }

        self.result
            .packages
            .insert(self.lexer.file_name.clone(), main);
    }

    pub fn solve_expression_to_number(expr: &Expression) -> Option<i128> {
        match expr {
            Expression::Number(l, r) => match r {
                Some(_) => todo!("Decimal numbers."),
                None => return Some(*l as i128),
            },
            Expression::Add(l, r) => {
                let left = Self::solve_expression_to_number(l);
                let right = Self::solve_expression_to_number(r);

                if left.is_none() || right.is_none() {
                    return None;
                }

                Some(left.unwrap() + right.unwrap())
            }
            Expression::Sub(l, r) => {
                let left = Self::solve_expression_to_number(l);
                let right = Self::solve_expression_to_number(r);

                if left.is_none() || right.is_none() {
                    return None;
                }

                Some(left.unwrap() - right.unwrap())
            }
            Expression::ToThePowerOf(l, r) => {
                let left = Self::solve_expression_to_number(l);
                let right = Self::solve_expression_to_number(r);

                if left.is_none() || right.is_none() {
                    return None;
                }

                Some(left.unwrap().pow(right.unwrap() as u32))
            }
            Expression::Parenthesis(expr) | Expression::Positive(expr) => {
                Self::solve_expression_to_number(expr)
            }
            Expression::Negative(expr) => {
                let expr = Self::solve_expression_to_number(expr);

                if expr.is_none() {
                    return None;
                }

                Some(0 - expr.unwrap())
            }
            _ => None,
        }
    }

    pub fn static_number_solver_pass(&mut self) {
        for (_, package) in &mut self.result.packages {
            let types = &mut package.file.types;
            'types_loop: for (_, ty) in types {
                match &mut ty.content {
                    RosarioTypeContent::Range(range) => {
                        let left = Self::solve_expression_to_number(&range.left);
                        let right = Self::solve_expression_to_number(&range.right);

                        if left.is_none() || right.is_none() {
                            continue 'types_loop;
                        }

                        let left_u = left.unwrap();
                        let right_u = right.unwrap();

                        range.left = Expression::Number(left_u, None);
                        range.right = Expression::Number(right_u, None);
                    }
                    _ => {}
                }
            }

            let public_types = &mut package.file.public_types;
            'types_loop: for (_, ty) in public_types {
                match &mut ty.content {
                    RosarioTypeContent::Range(range) => {
                        let left = Self::solve_expression_to_number(&range.left);
                        let right = Self::solve_expression_to_number(&range.right);

                        if left.is_none() || right.is_none() {
                            continue 'types_loop;
                        }

                        let left_u = left.unwrap();
                        let right_u = right.unwrap();

                        range.left = Expression::Number(left_u, None);
                        range.right = Expression::Number(right_u, None);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn get_solved_generic_type_signature(ty: &RosarioTypeSignature) -> RosarioTypeSignature {
        RosarioTypeSignature {
            name: format!("{}_{}", ty.name, {
                let mut result = String::new();

                for i in &ty.generics {
                    result += &(i.name.clone() + "_");
                }

                result.pop();
                result
            }),
            generics: vec![],
        }
    }

    pub fn get_solved_generics(expr: &mut Expression) -> Vec<RosarioTypeSignature> {
        let mut result: Vec<RosarioTypeSignature> = vec![];
        match expr {
            Expression::Body(body) => {
                for i in &mut body.content {
                    result.extend(Self::get_solved_generics(i));
                }
            }
            Expression::Let(let_var) => {
                if !let_var.ty.generics.is_empty() {
                    result.push(let_var.ty.clone());
                    let_var.ty = Self::get_solved_generic_type_signature(&let_var.ty);
                }

                result.extend(Self::get_solved_generics(&mut let_var.initializer));
            }
            Expression::NewEnum(new_enum) => {
                if !new_enum.ty.generics.is_empty() {
                    result.push(new_enum.ty.clone());
                    new_enum.ty = Self::get_solved_generic_type_signature(&new_enum.ty);
                }

                result.extend(Self::get_solved_generics(&mut new_enum.right));
            }
            _ => {}
        }

        result
    }

    pub fn convert_to_solved_type(
        original_signature: &RosarioTypeSignature,
        original_type: &RosarioType,
        solved_signature: &RosarioTypeSignature,
    ) -> RosarioType {
        let mut convert_to: BTreeMap<String, String> = BTreeMap::new();
        let mut count = 0;
        for i in &original_signature.generics {
            convert_to.insert(
                i.name.clone(),
                solved_signature.generics[count].name.clone(),
            );
            count += 1;
        }

        RosarioType {
            traits: original_type.traits.clone(),
            content: match &original_type.content {
                RosarioTypeContent::Enum(enumerable) => {
                    let mut final_enumerable = enumerable.clone();

                    for (_, enum_arguments) in &mut final_enumerable.contents {
                        for i in &mut enum_arguments.1 {
                            let mut final_enum_argument = EnumArgument::Unknown;
                            match i {
                                EnumArgument::Generic(generic) => {
                                    for (from, to) in &convert_to {
                                        if from == generic {
                                            final_enum_argument =
                                                EnumArgument::Type(RosarioTypeSignature {
                                                    name: to.clone(),
                                                    generics: vec![],
                                                });
                                        }
                                    }
                                }
                                _ => {}
                            }

                            if final_enum_argument != EnumArgument::Unknown {
                                *i = final_enum_argument;
                            }
                        }
                    }

                    RosarioTypeContent::Enum(final_enumerable)
                }
                _ => todo!(),
            },
        }
    }

    pub fn generic_solver_pass(&mut self) {
        let mut solved_generics: BTreeMap<String, Vec<RosarioTypeSignature>> = BTreeMap::new();
        for (name, package) in &mut self.result.packages {
            for procedure in &mut package.file.procedures {
                let content = Self::get_solved_generics(&mut procedure.body);
                match solved_generics.get_mut(name) {
                    Some(generics) => {
                        generics.extend(content);
                    }
                    None => {
                        solved_generics.insert(name.clone(), content);
                    }
                };
            }
        }

        for (package_name, signatures) in &solved_generics {
            'outer: for i in signatures {
                for (name, pack) in &self.generic_results {
                    if pack
                        .file
                        .public_types
                        .get(&Self::get_solved_generic_type_signature(i))
                        .is_some()
                    {
                        continue 'outer;
                    }
                }

                self.generic_results
                    .entry(package_name.clone())
                    .or_insert(Package::default());

                let ty = self.find_type_by_name(i.name.clone(), None, None, true);
                let solved_ty_signature = Self::get_solved_generic_type_signature(i);

                let solved_ty = Self::convert_to_solved_type(ty.0, ty.1, i);

                match self.generic_results.get_mut(package_name) {
                    Some(pkg) => {
                        pkg.file
                            .public_types
                            .insert(solved_ty_signature, solved_ty.clone());
                    }
                    None => unreachable!(),
                }
            }
        }
    }

    pub fn start(&mut self, external_packages: Option<BTreeMap<String, Package>>) {
        match external_packages {
            Some(e) => self.result.packages.extend(e),
            None => {}
        };

        self.main_pass();

        self.generic_solver_pass();

        self.static_number_solver_pass();
    }

    pub fn parse_implementation(
        &mut self,
        public_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
        local_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
    ) -> (RosarioTypeSignature, RosarioTypeImplementation) {
        self.token_pos += 1;

        let signature = self.parse_type_signature();

        self.token_pos += 1;

        let mut implementation = RosarioTypeImplementation::default();
        let mut is_public = false;
        let mut is_mutable_operator = false;
        while self.current_token().ty != TokenType::End {
            match self.current_token().ty {
                TokenType::Public => {
                    is_public = true;
                }
                TokenType::Procedure => {
                    let procedure =
                        self.parse_procedure(&signature.generics, public_types, local_types);
                    match is_public {
                        false => implementation.procedures.push(procedure),
                        true => implementation.public_procedures.push(procedure),
                    };

                    is_public = false;
                }
                TokenType::Function => {
                    let function =
                        self.parse_function(&signature.generics, public_types, local_types);
                    match is_public {
                        false => implementation.functions.push(function),
                        true => implementation.public_functions.push(function),
                    };

                    is_public = false;
                }
                TokenType::Operator => {
                    if !is_public {
                        panic!(
                            "Operators should always be `public`. {:?}",
                            self.current_token().begin_location
                        );
                    }

                    let operator =
                        self.parse_operator(&signature.generics, public_types, local_types);
                    match is_mutable_operator {
                        false => implementation.operators.push(operator),
                        true => implementation.mutable_operators.push(operator),
                    };

                    is_mutable_operator = false;
                    is_public = false;
                }
                TokenType::Mutable => {
                    self.token_pos += 1;

                    if self.current_token().ty != TokenType::Operator {
                        panic!(
                            "Expected the beginning of an operator, Found {:?} at {:?}",
                            self.current_token().ty,
                            self.current_token().begin_location
                        );
                    }

                    is_mutable_operator = true;
                    self.token_pos -= 1;
                }
                _ => todo!("{:?}", self.current_token().ty),
            }

            self.token_pos += 1;
        }

        self.check_ending(None, Some(TokenType::Implement));

        self.token_pos += 1;

        self.check_token(TokenType::Semicolon);

        (signature, implementation)
    }

    pub fn parse_operator(
        &mut self,
        generics: &Vec<Generic>,
        public_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
        local_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
    ) -> Operator {
        self.token_pos += 1;

        let signature = self.parse_signature(false, generics);

        self.token_pos += 1;

        self.check_token(TokenType::Is);

        self.token_pos += 1;

        let body = self.parse_expression(
            None,
            Some(signature.name.clone()),
            public_types,
            local_types,
        );

        self.token_pos += 1;

        self.check_token(TokenType::Semicolon);

        Operator {
            ty: signature.name.clone(),
            signature,
            body,
        }
    }

    pub fn get_identifier(&mut self) -> String {
        match &self.current_token().ty {
            TokenType::Identifier(id) => id.clone(),
            _ => panic!(
                "Expected Identifier, Found {:?} at {:?}",
                self.current_token().ty,
                self.current_token().begin_location
            ),
        }
    }

    pub fn check_token(&mut self, c: TokenType) {
        let current_ty = self.current_token().ty.clone();

        if current_ty != c {
            panic!(
                "Expected {:?}, Found {:?} at {:?}",
                c,
                current_ty,
                self.current_token().begin_location
            );
        }
    }

    pub fn check_arrow(&mut self) {
        self.check_token(TokenType::Arrow);
    }

    pub fn parse_signature(&mut self, is_procedure: bool, generics: &Vec<Generic>) -> Signature {
        let name = match &self.current_token().ty {
            TokenType::Identifier(id) => id.clone(),
            TokenType::StaticString(ss) => ss.clone(),
            _ => todo!("{:?}", self.current_token().ty),
        };

        self.token_pos += 1;

        self.check_token(TokenType::Character('('));

        let mut arguments = vec![];
        while self.current_token().ty != TokenType::Character(')') {
            self.token_pos += 1;

            if self.current_token().ty == TokenType::Character(')') {
                break;
            }

            let mut is_mutable = false;

            if self.current_token().ty == TokenType::Mutable {
                self.token_pos += 1;
                is_mutable = true;

                self.check_token(TokenType::SelfVariable);
                arguments.push(Argument {
                    argument_type: ArgumentType::SelfVariable,
                    is_mutable,
                });
            } else {
                if self.current_token().ty == TokenType::SelfVariable {
                    arguments.push(Argument {
                        argument_type: ArgumentType::SelfVariable,
                        is_mutable,
                    });
                } else {
                    let name = self.get_identifier();

                    self.token_pos += 1;

                    self.check_token(TokenType::Colon);

                    self.token_pos += 1;

                    if self.current_token().ty == TokenType::Mutable {
                        is_mutable = true;
                        self.token_pos += 1;
                    }

                    let ty = self.parse_type_signature();

                    arguments.push(Argument {
                        argument_type: ArgumentType::Variable(name, ty),
                        is_mutable,
                    });
                }
            }

            self.token_pos += 1;

            if self.current_token().ty != TokenType::Character(')')
                && self.current_token().ty != TokenType::Character(',')
            {
                panic!(
                    "Expected `)` or `,`. Found {:?} at {:?}",
                    self.current_token().ty,
                    self.current_token().begin_location
                );
            }

            if self.current_token().ty == TokenType::Character(')') {
                break;
            }
        }

        self.check_token(TokenType::Character(')'));

        self.token_pos += 1;

        let mut return_type = ReturnType::None;

        if !is_procedure {
            self.check_arrow();

            self.token_pos += 1;

            let mut is_mutable = IsMutable::False;
            if self.current_token().ty == TokenType::Mutable {
                is_mutable = IsMutable::True;
                self.token_pos += 1;
            }

            let return_signature = self.parse_type_signature();

            let mut get_generic_name = String::new();
            for i in generics {
                if i.name == return_signature.name {
                    get_generic_name = return_signature.name.clone();
                    break;
                }
            }

            return_type = if !get_generic_name.is_empty() {
                ReturnType::Generic(is_mutable, get_generic_name)
            } else {
                ReturnType::Type(is_mutable, return_signature)
            };
        } else {
            self.token_pos -= 1;
        }

        Signature {
            name,
            arguments: vec![],
            return_type,
        }
    }

    pub fn parse_procedure(
        &mut self,
        generics: &Vec<Generic>,
        public_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
        local_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
    ) -> Procedure {
        self.token_pos += 1;

        let signature = self.parse_signature(true, generics);

        self.token_pos += 1;

        self.check_token(TokenType::Is);

        self.token_pos += 1;

        let body = self.parse_expression(
            None,
            Some(signature.name.clone()),
            public_types,
            local_types,
        );

        self.token_pos += 1;

        self.check_token(TokenType::Semicolon);

        Procedure { signature, body }
    }

    pub fn parse_function(
        &mut self,
        generics: &Vec<Generic>,
        public_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
        local_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
    ) -> Function {
        self.token_pos += 1;

        let signature = self.parse_signature(false, generics);

        self.token_pos += 1;

        self.check_token(TokenType::Is);

        self.token_pos += 1;

        let body = self.parse_expression(
            None,
            Some(signature.name.clone()),
            public_types,
            local_types,
        );

        self.token_pos += 1;

        self.check_token(TokenType::Semicolon);

        Function { signature, body }
    }

    pub fn check_ending(&mut self, name: Option<String>, token: Option<TokenType>) {
        self.check_token(TokenType::End);
        match name {
            Some(name) => {
                self.token_pos += 1;
                if self.get_identifier() != name {
                    panic!(
                        "Expected the end of {:?}, Found {:?}",
                        name,
                        self.get_identifier()
                    );
                }
            }
            None => match token {
                Some(token) => {
                    self.token_pos += 1;
                    self.check_token(token);
                }
                None => {}
            },
        }
    }

    pub fn parse_variable(
        &mut self,
        id_name: String,
        public_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
        local_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
    ) -> Expression {
        // TODO: Add checking if variable exists.
        self.token_pos += 1;

        let mut ty = None;
        if self.current_token().ty == TokenType::LessThan {
            self.token_pos -= 1;

            ty = Some(self.parse_type_signature());

            self.token_pos += 1;
        }

        if self.current_token().ty == TokenType::DoubleColon {
            self.token_pos += 1;

            let right_name = self.get_identifier();

            let right = self.parse_variable(right_name.clone(), public_types, local_types);

            let ty_content = self
                .find_type_by_name(
                    ty.clone().unwrap().name,
                    Some(public_types),
                    Some(local_types),
                    false,
                )
                .1;

            match &ty_content.content {
                RosarioTypeContent::Enum(enumerable) => {
                    let mut enum_index = 0;
                    for (index, argument) in &enumerable.contents {
                        if argument.0 == right_name {
                            enum_index = *index;
                            break;
                        }
                    }

                    return Expression::NewEnum(NewEnum {
                        ty: ty.unwrap().clone(),
                        item: enum_index,
                        right: Box::new(right),
                    });
                }
                _ => todo!(),
            }
        }

        if self.current_token().ty != TokenType::Character('(') {
            self.token_pos -= 1;
            return Expression::Variable(id_name);
        }

        self.token_pos += 1;

        let mut args = Vec::new();
        while self.current_token().ty != TokenType::Character(')') {
            args.push(self.parse_expression(None, None, public_types, local_types));
            self.token_pos += 1;

            if self.current_token().ty == TokenType::Character(')') {
                break;
            }

            self.check_token(TokenType::Character(','));

            self.token_pos += 1;
        }

        Expression::FunctionCall(FunctionCall {
            name: id_name,
            arguments: args,
        })
    }

    pub fn find_type_by_name<'a>(
        &'a self,
        name: String,
        public_types: Option<&'a BTreeMap<RosarioTypeSignature, RosarioType>>,
        local_types: Option<&'a BTreeMap<RosarioTypeSignature, RosarioType>>,
        get_all_types: bool,
    ) -> (&'a RosarioTypeSignature, &'a RosarioType) {
        match public_types {
            Some(public_types) => {
                for (signature, ty) in public_types {
                    if signature.name == name {
                        return (signature, ty);
                    }
                }
            }
            None => {}
        };

        match local_types {
            Some(local_types) => {
                for (signature, ty) in local_types {
                    if signature.name == name {
                        return (signature, ty);
                    }
                }
            }
            None => {}
        };

        for (_, package) in &self.result.packages {
            for (signature, ty) in &package.file.public_types {
                if signature.name == name {
                    return (signature, ty);
                }
            }

            if get_all_types {
                for (signature, ty) in &package.file.types {
                    if signature.name == name {
                        return (signature, ty);
                    }
                }
            }
        }

        panic!("{} not found.", name)
    }

    pub fn parse_expression(
        &mut self,
        left: Option<Expression>,
        name: Option<String>,
        public_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
        local_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
    ) -> Expression {
        let final_name = name.clone();
        let expr = match &self.current_token().ty {
            TokenType::Number(left, right) => Expression::Number(
                left.parse::<i128>().unwrap_or(0),
                match right {
                    Some(right) => Some(right.parse::<i128>().unwrap_or(0)),
                    None => None,
                },
            ),
            TokenType::Let => self.parse_let(public_types, local_types),
            TokenType::Begin => {
                Expression::Body(self.parse_body(name, None, public_types, local_types))
            }
            TokenType::Clang => Expression::Clang({
                self.token_pos += 1;
                Box::new(self.parse_expression(None, name, public_types, local_types))
            }),
            TokenType::Match => self.parse_match(name, public_types, local_types),
            TokenType::SelfVariable => Expression::SelfVariable,
            TokenType::Identifier(id_name) => {
                self.parse_variable(id_name.clone(), public_types, local_types)
            }
            TokenType::StaticString(string) => Expression::StaticString(string.clone()),
            TokenType::Character('_') => Expression::MatchAll,
            TokenType::If => self.parse_if(name, public_types, local_types),
            TokenType::Sub => match left.clone() {
                None => Expression::Negative({
                    self.token_pos += 1;
                    Box::new(self.parse_expression(None, name, public_types, local_types))
                }),
                Some(left) => Expression::Sub(Box::new(left.clone()), {
                    self.token_pos += 1;
                    Box::new(self.parse_expression(Some(left), name, public_types, local_types))
                }),
            },
            TokenType::Add => match left.clone() {
                None => Expression::Positive({
                    self.token_pos += 1;
                    Box::new(self.parse_expression(None, name, public_types, local_types))
                }),
                Some(left) => Expression::Add(Box::new(left.clone()), {
                    self.token_pos += 1;
                    Box::new(self.parse_expression(Some(left), name, public_types, local_types))
                }),
            },
            TokenType::Character('(') => {
                self.token_pos += 1;

                let expr = self.parse_expression(left.clone(), name, public_types, local_types);

                self.token_pos += 1;

                self.check_token(TokenType::Character(')'));

                Expression::Parenthesis(Box::new(expr))
            }
            TokenType::IsEquals => {
                self.token_pos += 1;
                let condition = self.parse_expression(None, None, public_types, local_types);
                return Expression::Condition(Condition {
                    left: Box::new(left.unwrap()),
                    cond: ConditionType::Equals,
                    right: Box::new(condition),
                });
            }
            TokenType::ToThePowerOf => {
                self.token_pos += 1;
                let condition = self.parse_expression(None, None, public_types, local_types);

                match condition {
                    Expression::Add(l, r) | Expression::Sub(l, r) => {
                        return Expression::Sub(
                            Box::new(Expression::ToThePowerOf(Box::new(left.clone().unwrap()), l)),
                            r,
                        );
                    }
                    _ => Expression::ToThePowerOf(
                        Box::new(left.clone().unwrap()),
                        Box::new(condition),
                    ),
                }
            }
            TokenType::Not => Expression::Not({
                self.token_pos += 1;
                Box::new(self.parse_expression(None, None, public_types, local_types))
            }),
            TokenType::Dot => {
                self.token_pos += 1;
                let right = self.parse_expression(None, None, public_types, local_types);

                match right {
                    Expression::FunctionCall(call) => {
                        Expression::AccessFunction(Box::new(left.clone().unwrap()), call)
                    }
                    Expression::Variable(var) => {
                        Expression::AccessVariable(Box::new(left.clone().unwrap()), var)
                    }
                    _ => todo!("{right:?}"),
                }
            }
            _ => todo!(
                "{:?} at {:?} in file {:?}",
                self.current_token().ty,
                self.current_token().begin_location,
                self.lexer.file_name,
            ),
        };

        self.token_pos += 1;

        if self.current_token().ty == TokenType::Semicolon
            || self.current_token().ty == TokenType::RangeDots
            || self.current_token().ty == TokenType::Character(')')
            || self.current_token().ty == TokenType::Arrow
            || matches!(self.current_token().ty, TokenType::Number(_, _))
            || self.current_token().ty == TokenType::Character(',')
            || self.current_token().ty == TokenType::Then
            || self.current_token().ty == TokenType::Is
        {
            self.token_pos -= 1;
            return expr;
        }

        self.parse_expression(Some(expr), final_name, public_types, local_types)
    }

    pub fn parse_if(
        &mut self,
        name: Option<String>,
        public_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
        local_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
    ) -> Expression {
        let mut statements: Vec<(Expression, Body)> = vec![];

        let mut is_in_else = false;
        while self.current_token().ty != TokenType::End {
            self.token_pos += 1;

            let condition = self.parse_expression(None, None, public_types, local_types);

            self.token_pos += 1;

            if !is_in_else {
                self.check_token(TokenType::Then);
            }

            self.token_pos += 1;

            let body = self.parse_body(
                None,
                Some(vec![TokenType::ElsIf, TokenType::Else, TokenType::End]),
                public_types,
                local_types,
            );

            statements.push((condition, body));

            if is_in_else {
                self.check_token(TokenType::End);
                break;
            }

            is_in_else = self.current_token().ty == TokenType::Else;

            if self.current_token().ty != TokenType::ElsIf
                && self.current_token().ty != TokenType::Else
            {
                panic!(
                    "Expected `elsif` or `else`. Found {:?} at {:?}",
                    self.current_token().ty,
                    self.current_token().begin_location
                );
            }
        }

        self.check_ending(name, Some(TokenType::If));

        Expression::If(If { statements })
    }

    pub fn parse_match_option_variables(&mut self) -> Vec<String> {
        let mut result = vec![];

        self.token_pos += 1;

        if self.current_token().ty != TokenType::Arrow {
            if self.current_token().ty != TokenType::Character('(') {
                panic!(
                    "Expected `=>` or `(`. Found {:?} at {:?}",
                    self.current_token().ty,
                    self.current_token().begin_location
                );
            }

            self.token_pos += 1;

            while self.current_token().ty != TokenType::Character(')') {
                result.push(match &self.current_token().ty {
                    TokenType::Identifier(id) => id.clone(),
                    TokenType::Character('_') => "_".to_string(),
                    _ => todo!("{:?}", self.current_token().ty),
                });

                self.token_pos += 1;

                if self.current_token().ty != TokenType::Character(')')
                    && self.current_token().ty != TokenType::Character(',')
                {
                    panic!(
                        "Expected `)` or `,`. Found {:?} at {:?}",
                        self.current_token().ty,
                        self.current_token().begin_location
                    );
                }

                if self.current_token().ty == TokenType::Character(')') {
                    break;
                }

                self.token_pos += 1;
            }
        } else {
            self.token_pos -= 1;
        }

        result
    }

    pub fn parse_match_option(&mut self) -> MatchOption {
        let mut identifiers: Vec<(String, Vec<String>)> = vec![];
        while self.current_token().ty != TokenType::Arrow {
            match &self.current_token().ty {
                TokenType::Identifier(id) => {
                    let name = id.clone();

                    self.token_pos += 1;

                    let mut variables: Vec<String> = vec![];

                    if self.current_token().ty == TokenType::Character('(') {
                        self.token_pos += 1;
                        while self.current_token().ty != TokenType::Character(')') {
                            match &self.current_token().ty {
                                TokenType::Identifier(id) => variables.push(id.clone()),
                                TokenType::Character('_') => variables.push("_".to_string()),
                                _ => todo!("{:?}", &self.current_token().ty),
                            }

                            self.token_pos += 1;

                            if self.current_token().ty != TokenType::Character(')')
                                && self.current_token().ty != TokenType::Character(',')
                            {
                                panic!(
                                    "Expected `)` or `,`. Found {:?} at {:?}",
                                    self.current_token().ty,
                                    self.current_token().begin_location
                                );
                            }

                            if self.current_token().ty == TokenType::Character(')') {
                                self.token_pos += 1;
                                break;
                            }

                            self.token_pos += 1;
                        }
                    }

                    identifiers.push((name, variables));
                }
                _ => todo!("{:?}", self.current_token().ty),
            }
        }

        if self.current_token().ty == TokenType::Arrow {
            self.token_pos -= 1;
        }

        if identifiers.len() == 1 {
            return MatchOption::SingleIdentifier(
                identifiers[0].0.clone(),
                identifiers[0].1.clone(),
            );
        } else {
            return MatchOption::MultipleIdentifiers(identifiers);
        }
    }

    pub fn parse_match(
        &mut self,
        name: Option<String>,
        public_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
        local_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
    ) -> Expression {
        self.token_pos += 1;

        let condition = self.parse_expression(None, None, public_types, local_types);

        self.token_pos += 1;

        self.check_token(TokenType::Is);

        self.token_pos += 1;

        let mut elements = Vec::new();
        while self.current_token().ty != TokenType::End {
            let left = self.parse_match_option();

            self.token_pos += 1;

            self.check_arrow();

            self.token_pos += 1;

            let right = self.parse_expression(None, None, public_types, local_types);

            self.token_pos += 1;

            self.check_token(TokenType::Character(','));

            self.token_pos += 1;

            elements.push((left, right));
        }

        self.token_pos += 1;

        match name {
            Some(name) => {
                if self.get_identifier() != name {
                    panic!(
                        "Expected end of {:?}, Found {:?}",
                        name,
                        self.current_token()
                    );
                }
            }
            None => self.check_token(TokenType::Match),
        }

        Expression::Match(Match {
            condition: Box::new(condition),
            elements,
        })
    }

    pub fn parse_let(
        &mut self,
        public_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
        local_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
    ) -> Expression {
        self.token_pos += 1;
        let is_mutable = match matches!(self.current_token().ty, TokenType::Mutable) {
            true => {
                self.token_pos += 1;
                true
            }
            false => false,
        };

        let name = self.get_identifier();

        self.token_pos += 1;
        self.check_token(TokenType::Colon);

        self.token_pos += 1;
        let ty = self.parse_type_signature();

        self.token_pos += 1;
        let initializer = Box::new(match matches!(self.current_token().ty, TokenType::Equals) {
            true => {
                self.token_pos += 1;
                self.parse_expression(None, Some(name.clone()), public_types, local_types)
            }
            false => {
                panic!(
                    "Expected initializer. Found {:?} at {:?}",
                    self.current_token().ty,
                    self.current_token().begin_location
                );
            }
        });

        Expression::Let(Let {
            name,
            is_mutable,
            ty,
            initializer,
        })
    }

    pub fn parse_body(
        &mut self,
        name: Option<String>,
        no_body: Option<Vec<TokenType>>,
        public_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
        local_types: &BTreeMap<RosarioTypeSignature, RosarioType>,
    ) -> Body {
        if no_body.is_none() {
            self.check_token(TokenType::Begin);
            self.token_pos += 1;
        }

        let mut content = Vec::new();
        while match no_body {
            None => self.current_token().ty != TokenType::End,
            Some(ref vec) => {
                let mut can_advance = true;
                for i in vec {
                    if self.current_token().ty == *i {
                        can_advance = false;
                        break;
                    }
                }

                can_advance
            }
        } {
            content.push(self.parse_expression(None, None, public_types, local_types));
            self.token_pos += 1;

            self.check_token(TokenType::Semicolon);
            self.token_pos += 1;
        }

        if no_body.is_none() {
            self.check_ending(name, None);
        }

        Body { content }
    }
}
