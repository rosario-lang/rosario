use std::collections::BTreeMap;

use crate::{
    ast::{EnumArgument, Expression, Package, RosarioType, TypeContent, TypeSignature},
    parser::Parser,
};

pub fn get_solved_generic_type_signature(ty: &TypeSignature) -> TypeSignature {
    TypeSignature {
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

pub fn get_solved_generics(expr: &mut Expression) -> Vec<TypeSignature> {
    let mut result: Vec<TypeSignature> = vec![];
    match expr {
        Expression::Body(body) => {
            for i in &mut body.content {
                result.extend(get_solved_generics(i));
            }
        }
        Expression::Let(let_var) => {
            if !let_var.ty.generics.is_empty() {
                result.push(let_var.ty.clone());
                let_var.ty = get_solved_generic_type_signature(&let_var.ty);
            }

            result.extend(get_solved_generics(&mut let_var.initializer));
        }
        Expression::NewEnum(new_enum) => {
            if !new_enum.ty.generics.is_empty() {
                result.push(new_enum.ty.clone());
                new_enum.ty = get_solved_generic_type_signature(&new_enum.ty);
            }

            result.extend(get_solved_generics(&mut new_enum.right));
        }
        _ => {}
    }

    result
}

pub fn convert_to_solved_type(
    original_signature: &TypeSignature,
    original_type: &RosarioType,
    solved_signature: &TypeSignature,
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
            TypeContent::Enum(enumerable) => {
                let mut final_enumerable = enumerable.clone();

                for (_, enum_arguments) in &mut final_enumerable.contents {
                    for i in &mut enum_arguments.1 {
                        let mut final_enum_argument = EnumArgument::Unknown;
                        match i {
                            EnumArgument::Generic(generic) => {
                                for (from, to) in &convert_to {
                                    if from == generic {
                                        final_enum_argument = EnumArgument::Type(TypeSignature {
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

                TypeContent::Enum(final_enumerable)
            }
            _ => todo!(),
        },
    }
}

pub fn generic_solver_pass(parser: &mut Parser) {
    let mut solved_generics: BTreeMap<String, Vec<TypeSignature>> = BTreeMap::new();
    for (name, package) in &mut parser.result.packages {
        for procedure in &mut package.file.procedures {
            let content = get_solved_generics(&mut procedure.body);
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
            for (_name, pack) in &parser.generic_results {
                if pack
                    .file
                    .public_types
                    .get(&get_solved_generic_type_signature(i))
                    .is_some()
                {
                    continue 'outer;
                }
            }

            parser
                .generic_results
                .entry(package_name.clone())
                .or_insert(Package::default());

            let ty = parser.find_type_by_name(i.name.clone(), None, None, true);
            let solved_ty_signature = get_solved_generic_type_signature(i);

            let solved_ty = convert_to_solved_type(ty.0, ty.1, i);

            match parser.generic_results.get_mut(package_name) {
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
