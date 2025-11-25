use std::collections::BTreeMap;

use rosarioparser::{
    ast::{
        ArgumentType, EnumArgument, Expression, Package, ReturnType, RosarioType, Signature,
        TypeContent, TypeSignature,
    },
    parser::Parser,
};

pub struct CCompiler {
    parser: Parser,
}

#[derive(Debug, Clone)]
pub enum CIntegerType {
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
}

impl CIntegerType {
    pub fn from_limits(left: i128, right: i128) -> (bool, Self) {
        let is_signed = left < 0;

        if is_signed {
            if left >= -128 && right <= 127 {
                return (is_signed, Self::Int8);
            }
            if left >= -32768 && right <= 32767 {
                return (is_signed, Self::Int16);
            }
            if left >= -2147483648 && right <= 2147483647 {
                return (is_signed, Self::Int32);
            }
            if left >= -9223372036854775808 && right <= 9223372036854775807 {
                return (is_signed, Self::Int64);
            }
        } else {
            if right == 1 {
                return (is_signed, Self::Bool);
            }
            if right <= 255 {
                return (is_signed, Self::Int8);
            }
            if right <= 65535 {
                return (is_signed, Self::Int16);
            }
            if right <= 4294967295 {
                return (is_signed, Self::Int32);
            }
            if right <= 18446744073709551615 {
                return (is_signed, Self::Int64);
            }
        }

        todo!()
    }

    pub fn codegen(&self) -> String {
        let res = match self {
            Self::Bool => "bool",
            Self::Int8 => "char",
            Self::Int16 => "short int",
            Self::Int32 => "long int",
            Self::Int64 => "long long int",
        };

        res.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct CInteger {
    pub is_signed: bool,
    pub final_size: CIntegerType,
    pub limit_left: i128,
    pub limit_right: i128,
}

#[derive(Debug, Clone)]
pub struct CEnumItem {
    pub name: String,
    pub types: Vec<TypeSearch>,
}

#[derive(Debug, Clone)]
pub struct CEnum {
    pub items: Vec<CEnumItem>,
}

#[derive(Debug, Clone)]
pub enum CType {
    Integer(CInteger),
    Enum(CEnum),
    TypeDef(TypeSignature),
}

pub struct CSignature {
    name: String,
    ty: String,
    args: Vec<(String, String)>,
}

impl CSignature {
    pub fn codegen(&self, package_name: String) -> String {
        format!("{} {}_{}({})", self.ty, package_name, self.name, {
            let mut result = String::new();

            for (name, ty) in &self.args {
                result += &format!("{} {}, ", name, ty);
            }

            if result.ends_with(", ") {
                result.pop();
                result.pop();
            }

            result
        })
    }
}

impl CType {
    pub fn codegen(
        &self,
        package_name: String,
        name: String,
        comp: &CCompiler,
        types: &Vec<PackageTypes>,
    ) -> String {
        match self {
            Self::Integer(c_int) => {
                let mut result = "typedef ".to_string();

                if !matches!(c_int.final_size, CIntegerType::Bool) {
                    match c_int.is_signed {
                        false => result += "unsigned ",
                        true => result += "signed ",
                    };
                }

                result += &c_int.final_size.codegen();
                result += &format!(" {}_{};\n", &package_name, &name);

                return result;
            }
            Self::TypeDef(def) => {
                format!(
                    "typedef {} {}_{};\n",
                    match comp.find_type_by_signature(def.clone(), types) {
                        TypeSearch::Found(pkg, st) => format!("{}_{}", pkg, st),
                        _ => unreachable!(),
                    },
                    &package_name,
                    &name
                )
            }
            Self::Enum(c_enum) => {
                let mut item_count = 0_usize;
                let numeric_enum = {
                    let mut result = String::new();

                    for i in &c_enum.items {
                        result += &format!(
                            "#define {} {}\n",
                            format!("{}_{}_{}", &package_name, &name, i.name).to_ascii_uppercase(),
                            if c_enum.items.len() <= 2 {
                                match item_count {
                                    0 => "false".to_string(),
                                    1 => "true".to_string(),
                                    _ => unreachable!(),
                                }
                            } else {
                                item_count.to_string()
                            },
                        );

                        item_count += 1;
                    }

                    result
                };

                let enum_integer_ty = CIntegerType::from_limits(0, item_count as i128 - 1).1;

                let struct_enum = format!(
                    "typedef struct {{\n    {} kind;\n{}}} {}_{};\n",
                    &format!(
                        "{}{}",
                        if !matches!(enum_integer_ty, CIntegerType::Bool) {
                            "unsigned "
                        } else {
                            ""
                        },
                        enum_integer_ty.codegen(),
                    ),
                    {
                        let mut result = String::new();

                        let mut count = 0;
                        for i in &c_enum.items {
                            let mut items_count = 0_usize;
                            for j in &i.types {
                                result += &format!(
                                    "    {} {}_{};\n",
                                    match j {
                                        TypeSearch::Found(pkg, ty) => format!("{}_{}", pkg, ty),
                                        _ => panic!("Type {} not found", i.name),
                                    },
                                    i.name,
                                    items_count,
                                );
                                items_count += 1;
                            }
                            count += 1;
                        }

                        result
                    },
                    &package_name,
                    &name,
                );

                let enum_initializers = {
                    let mut result = String::new();

                    for i in &c_enum.items {
                        result += &format!(
                            "{}_{} New_{}_{}_{}({}) {{\n{}}}\n",
                            &package_name,
                            &name,
                            &package_name,
                            &name,
                            i.name,
                            {
                                let mut args_result = String::new();

                                let mut count = 0;
                                for j in &i.types {
                                    match j {
                                        TypeSearch::Found(pkg, st) => {
                                            args_result += &format!("{}_{} v{}, ", pkg, st, count)
                                        }
                                        _ => {}
                                    };
                                    count += 1;
                                }

                                if args_result.ends_with(", ") {
                                    args_result.pop();
                                    args_result.pop();
                                }

                                args_result
                            },
                            format!(
                                "    {}_{} result = {{ .kind = {}, {} }};\n    return result;\n",
                                &package_name,
                                &name,
                                format!("{}_{}_{}", &package_name, &name, i.name)
                                    .to_ascii_uppercase(),
                                {
                                    let mut params_result = String::new();

                                    let mut items_count = 0;
                                    for _j in &i.types {
                                        params_result += &format!(
                                            ".{}_{} = v{}",
                                            i.name, items_count, items_count
                                        );
                                        items_count += 1;
                                    }

                                    params_result
                                }
                            ),
                        )
                    }

                    result
                };

                return format!("{}\n{}\n{}", numeric_enum, struct_enum, enum_initializers);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct PackageTypes {
    pub package_name: String,
    pub dependencies: Vec<String>,
    pub types: BTreeMap<TypeSignature, CType>,
}

#[derive(Debug, Clone)]
pub enum TypeSearch {
    Unknown(String),
    Found(String, String),
}

impl CCompiler {
    pub fn new(parser: Parser) -> Self {
        Self { parser }
    }

    pub fn get_static_constant_number(expr: &Expression) -> i128 {
        match expr {
            Expression::Number(left, right, _) => match right {
                Some(_) => todo!("Decimal Numbers."),
                None => *left,
            },
            _ => panic!(
                "Expression is not a static constant number! Found {:?}.",
                expr
            ),
        }
    }

    pub fn codegen_type(&self, ty: RosarioType, types: &Vec<PackageTypes>) -> CType {
        match ty.content {
            TypeContent::Range(range) => {
                let left = Self::get_static_constant_number(&range.left);
                let right = Self::get_static_constant_number(&range.right);

                let (is_signed, final_size) = CIntegerType::from_limits(left, right);

                CType::Integer(CInteger {
                    is_signed,
                    final_size,
                    limit_left: left,
                    limit_right: right,
                })
            }
            TypeContent::Enum(enumerable) => CType::Enum(CEnum {
                items: {
                    let mut result = vec![];

                    for (_, (name, arguments)) in enumerable.contents {
                        let mut args_result = vec![];

                        for i in arguments {
                            args_result.push(match i {
                                EnumArgument::Type(ty) => self.find_type_by_signature(ty, types),
                                _ => panic!("No generics and/or unknown types allowed."),
                            })
                        }

                        result.push(CEnumItem {
                            name,
                            types: args_result,
                        });
                    }

                    dbg!(result)
                },
            }),
            TypeContent::TypeRef(reference) => CType::TypeDef(reference),
            _ => todo!("{:?}", ty),
        }
    }

    pub fn codegen_types(
        &self,
        package_name: String,
        package: &Package,
        package_types: &mut Vec<PackageTypes>,
    ) {
        package_types.push(PackageTypes {
            package_name,
            dependencies: vec![],
            types: BTreeMap::new(),
        });

        for (name, ty) in &package.file.types {
            if name.generics.len() != 0 {
                continue;
            }

            let final_ty = self.codegen_type(ty.clone(), package_types);

            package_types
                .last_mut()
                .unwrap()
                .types
                .insert(name.clone(), final_ty);
        }

        for (name, ty) in &package.file.public_types {
            if name.generics.len() != 0 {
                continue;
            }

            let final_ty = self.codegen_type(ty.clone(), package_types);

            package_types
                .last_mut()
                .unwrap()
                .types
                .insert(name.clone(), final_ty);
        }
    }

    pub fn codegen_signature(&self, signature: &Signature) -> CSignature {
        let ty = match &signature.return_type {
            ReturnType::None => "void".to_string(),
            ReturnType::Type(_, type_signature) => type_signature.name.clone(),
            ReturnType::Generic(_, _) => todo!("Generics"),
        };

        let name = signature.name.clone();

        let mut args = vec![];
        for i in &signature.arguments {
            match &i.argument_type {
                ArgumentType::Variable(name, type_signature) => {
                    args.push((type_signature.name.clone(), name.clone()));
                }
                _ => todo!(),
            }
        }

        CSignature { name, ty, args }
    }

    pub fn find_type_by_signature(
        &self,
        name: TypeSignature,
        types: &Vec<PackageTypes>,
    ) -> TypeSearch {
        for ty in types {
            for (signature, _) in &ty.types {
                if *signature == name {
                    return TypeSearch::Found(ty.package_name.clone(), name.name);
                }
            }
        }

        TypeSearch::Unknown(name.name)
    }

    pub fn find_type_content_by_signature(
        &self,
        name: TypeSignature,
        types: &Vec<PackageTypes>,
    ) -> CType {
        for ty in types {
            for (signature, content) in &ty.types {
                if *signature == name {
                    return content.clone();
                }
            }
        }

        panic!("{} not found.", name.name);
    }

    pub fn find_unknown_type_search(&self, search: &mut TypeSearch, types: &Vec<PackageTypes>) {
        if matches!(search, TypeSearch::Found(_, _)) {
            return;
        }

        for ty in types {
            for (signature, _) in &ty.types {
                match search {
                    TypeSearch::Found(_, _) => return,
                    TypeSearch::Unknown(name) => {
                        if *signature.name == *name {
                            *search =
                                TypeSearch::Found(ty.package_name.clone(), signature.name.clone());
                        }
                    }
                }
            }
        }

        panic!(
            "Type {} not found.",
            match search {
                TypeSearch::Unknown(name) => name,
                _ => unreachable!(),
            }
        );
    }

    pub fn tabs(tabs: usize) -> String {
        let mut result = String::new();

        for _ in 0..tabs {
            result += "    ";
        }

        result
    }

    pub fn codegen_expression(
        &self,
        tabs: usize,
        expr: &Expression,
        types: &Vec<PackageTypes>,
    ) -> String {
        let mut result = Self::tabs(tabs);

        match expr {
            Expression::Body(body) => {
                let mut first_tab = true;
                for i in &body.content {
                    result +=
                        &self.codegen_expression(if first_tab { tabs - 1 } else { tabs }, i, types);

                    if !result.ends_with("}") {
                        result += ";";
                    }

                    result += "\n";
                    first_tab = false;
                }
            }
            Expression::Let(let_var) => {
                result += &format!(
                    "{}{} {} = {}",
                    match let_var.is_mutable {
                        true => "",
                        false => "const ",
                    },
                    match self.find_type_by_signature(let_var.ty.clone(), types) {
                        TypeSearch::Found(pkg, st) => format!("{}_{}", pkg, st),
                        _ => panic!("Type {} not found", let_var.name),
                    },
                    let_var.name,
                    &self.codegen_expression(0, &let_var.initializer, types)
                );
            }
            Expression::Add(left, right) => {
                result += &format!(
                    "{} + {}",
                    self.codegen_expression(0, left, types),
                    self.codegen_expression(0, right, types),
                )
            }
            Expression::Variable(name) => result += &name,
            Expression::Number(left, right, _) => match right {
                Some(r) => result += &format!("{}.{}f", left.to_string(), r.to_string()),
                None => result += &left.to_string(),
            },
            Expression::Clang(expr) => {
                result += &format!(
                    "{{\n{};\n{}}}",
                    self.codegen_expression(tabs + 1, expr, types),
                    Self::tabs(tabs)
                )
            }
            Expression::FunctionCall(fn_call) => {
                result += &format!("{}({})", &fn_call.name, {
                    let mut call_args = String::new();

                    for i in &fn_call.arguments {
                        call_args += &format!("{}, ", self.codegen_expression(0, i, types));
                    }

                    if call_args.ends_with(", ") {
                        call_args.pop();
                        call_args.pop();
                    }

                    call_args
                })
            }
            Expression::StaticString(string) => result += &format!("\"{}\"", string),
            Expression::NewEnum(new_enum) => {
                result += &format!(
                    "New_{}_{}",
                    match self.find_type_by_signature(new_enum.ty.clone(), types) {
                        TypeSearch::Found(pkg, st) => format!("{}_{}", pkg, st),
                        _ => panic!("{:?}", new_enum.ty),
                    },
                    self.codegen_expression(0, &new_enum.right, types),
                )
            }
            Expression::AccessVariable(left, right) => {
                result += &format!("{}.{}", self.codegen_expression(0, left, types), right);
            }
            _ => todo!("{:?}", &expr),
        }

        result
    }

    pub fn start(&mut self) -> String {
        let mut c_includes = String::new();
        let mut c_file = String::new();
        let mut main_name = String::new();
        let mut final_types: Vec<PackageTypes> = vec![];

        for (package_name, package) in &self.parser.result.packages {
            for include in &package.c_includes {
                c_includes += &format!("#include \"{}\"\n", include);
            }

            self.codegen_types(package_name.replace("::", "_"), package, &mut final_types);
        }

        for (package_name, package) in &self.parser.generic_results {
            for include in &package.c_includes {
                c_includes += &format!("#include \"{}\"\n", include);
            }

            self.codegen_types(package_name.replace("::", "_"), package, &mut final_types);
        }

        let mut new_final_types = final_types.clone();
        for i in &mut new_final_types {
            for (_, j) in &mut i.types {
                match j {
                    CType::Enum(e) => {
                        for k in &mut e.items {
                            'outer: for m in &mut k.types {
                                for ty in &final_types {
                                    for (signature, _) in &ty.types {
                                        match m {
                                            TypeSearch::Found(_, _) => {
                                                continue 'outer;
                                            }
                                            TypeSearch::Unknown(name) => {
                                                if *signature.name == *name {
                                                    *m = TypeSearch::Found(
                                                        ty.package_name.clone(),
                                                        signature.name.clone(),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        final_types = new_final_types;

        c_file += &format!(
            "#ifndef ROSARIO_BOOLEANS_DEFINED
    #define ROSARIO_BOOLEANS_DEFINED
    #if __STDC_VERSION__ < 202311l
        #define bool _Bool
        #define true 1
        #define false 0
    #endif

    #define __bool_true_false_are_defined 1
#endif\n"
        );

        c_file += &format!("{}\n", c_includes);

        for final_type in &final_types {
            for (name, ty) in &final_type.types {
                c_file += &(ty.codegen(
                    final_type.package_name.replace("::", "_"),
                    name.name.clone(),
                    self,
                    &final_types,
                ) + "\n");
            }
        }

        for (package_name, package) in &self.parser.result.packages {
            for (signature, implement) in &package.file.implementations {
                for i in &implement.public_functions {}
            }

            for procedure in &package.file.procedures {
                let signature = self.codegen_signature(&procedure.signature);
                if procedure.signature.name == "Main"
                    && procedure.signature.return_type == ReturnType::None
                {
                    main_name = format!(
                        "{}_{}",
                        package_name.replace("::", "_"),
                        signature.name.clone()
                    );
                }
                let body = self.codegen_expression(1, &procedure.body, &final_types);
                c_file += &format!(
                    "{} {{\n{}}}\n",
                    signature.codegen(package_name.clone()),
                    body
                );
            }
        }

        if !main_name.is_empty() {
            c_file += &format!("int main() {{\n    {}();\n    return 0;\n}}", main_name);
        }

        c_file
    }
}

#[cfg(test)]
mod tests {
    use rosarioparser::{lexer::Lexer, parser::Parser};

    use crate::CCompiler;

    #[test]
    fn basic_test() {
        let mut lexer = Lexer::from_file("tests/c_main.ros");
        lexer.start();

        dbg!(&lexer.contents);

        let mut parser = Parser::new(lexer);
        parser.start(Some(rosarioparser::parse_core().result.packages));

        dbg!(&parser.result);

        let mut compiler = CCompiler::new(parser);

        std::fs::write("tests/c_main.c", compiler.start()).unwrap();
    }

    #[test]
    fn core_compiling() {
        let mut lexer = Lexer::from_file("../../core/library.ros");
        lexer.start();

        dbg!(&lexer.contents);

        let mut parser = Parser::new(lexer);
        parser.start(None);

        dbg!(&parser.result);

        let mut compiler = CCompiler::new(parser);

        compiler.start();
    }
}
