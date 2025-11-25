use std::collections::BTreeMap;

use crate::{
    ast::{
        Expression, ImplSignature, Package, ReturnType, TypeContent, TypeImplementation,
        TypeSignature,
    },
    parser::Parser,
};

pub fn find_type_implementations_by_signature(
    sig: &TypeSignature,
    packages: &BTreeMap<String, Package>,
) -> Vec<(ImplSignature, TypeImplementation)> {
    let mut result = vec![];
    for (_name, package) in packages {
        for (signature, implement) in &package.file.implementations {
            if signature.impl_for == *sig {
                result.push((signature.clone(), implement.clone()));
            }
        }
    }

    result
}

pub fn solve_expression_to_number(
    expr: &Expression,
    packages: &BTreeMap<String, Package>,
) -> Option<i128> {
    match expr {
        Expression::Number(l, r, _) => match r {
            Some(_) => todo!("Decimal numbers."),
            None => return Some(*l as i128),
        },
        Expression::Add(l, r) => {
            let left = solve_expression_to_number(l, packages);
            let right = solve_expression_to_number(r, packages);

            if left.is_none() || right.is_none() {
                return None;
            }

            Some(left.unwrap() + right.unwrap())
        }
        Expression::Sub(l, r) => {
            let left = solve_expression_to_number(l, packages);
            let right = solve_expression_to_number(r, packages);

            if left.is_none() || right.is_none() {
                return None;
            }

            Some(left.unwrap() - right.unwrap())
        }
        Expression::ToThePowerOf(l, r) => {
            let left = solve_expression_to_number(l, packages);
            let right = solve_expression_to_number(r, packages);

            if left.is_none() || right.is_none() {
                return None;
            }

            Some(left.unwrap().pow(right.unwrap() as u32))
        }
        Expression::Parenthesis(expr) | Expression::Positive(expr) => {
            solve_expression_to_number(expr, packages)
        }
        Expression::Negative(expr) => {
            let expr = solve_expression_to_number(expr, packages);

            if expr.is_none() {
                return None;
            }

            Some(0 - expr.unwrap())
        }
        Expression::AccessTypeImplementation(signature, expr) => match &**expr {
            Expression::FunctionCall(fn_call) => {
                let implementations = find_type_implementations_by_signature(signature, packages);

                for (impl_signature, implement) in implementations {
                    if impl_signature.impl_for == *signature {
                        'public_functions_loop: for i in implement.public_functions {
                            // TODO: Add const evaluation with arguments.
                            if i.signature.name == fn_call.name && fn_call.arguments.len() == 0 {
                                let body = solve_expression_to_number(&i.body, packages);

                                if body.is_none() {
                                    continue 'public_functions_loop;
                                }

                                return body;
                            }
                        }
                    }
                }

                return None;
            }
            _ => todo!("{:?}", expr),
        },
        _ => None,
    }
}

pub fn static_number_solver_pass(parser: &mut Parser) {
    let why_do_i_have_to_do_this = parser.result.packages.clone();
    for (_, package) in &mut parser.result.packages {
        let implements = &mut package.file.implementations;
        for (_, implement) in implements {
            'impl_functions_loop: for i in &mut implement.functions {
                let result = solve_expression_to_number(&i.body, &why_do_i_have_to_do_this);

                if result.is_none() {
                    continue 'impl_functions_loop;
                }

                let result_u = result.unwrap();

                i.body = Expression::Number(
                    result_u,
                    None,
                    Some(match &i.signature.return_type {
                        ReturnType::Type(_, ty) => ty.clone(),
                        _ => unreachable!(),
                    }),
                );
            }

            'impl_public_functions_loop: for i in &mut implement.public_functions {
                let result = solve_expression_to_number(&i.body, &why_do_i_have_to_do_this);

                if result.is_none() {
                    continue 'impl_public_functions_loop;
                }

                let result_u = result.unwrap();

                i.body = Expression::Number(
                    result_u,
                    None,
                    Some(match &i.signature.return_type {
                        ReturnType::Type(_, ty) => ty.clone(),
                        _ => unreachable!(),
                    }),
                );
            }
        }

        let types = &mut package.file.types;
        'types_loop: for (signature, ty) in types {
            match &mut ty.content {
                TypeContent::Range(range) => {
                    let left = solve_expression_to_number(&range.left, &why_do_i_have_to_do_this);
                    let right = solve_expression_to_number(&range.right, &why_do_i_have_to_do_this);

                    if left.is_none() || right.is_none() {
                        continue 'types_loop;
                    }

                    let left_u = left.unwrap();
                    let right_u = right.unwrap();

                    range.left = Expression::Number(left_u, None, Some(signature.clone()));
                    range.right = Expression::Number(right_u, None, Some(signature.clone()));
                }
                _ => {}
            }
        }

        let public_types = &mut package.file.public_types;
        'types_loop: for (signature, ty) in public_types {
            match &mut ty.content {
                TypeContent::Range(range) => {
                    let left = solve_expression_to_number(&range.left, &why_do_i_have_to_do_this);
                    let right = solve_expression_to_number(&range.right, &why_do_i_have_to_do_this);

                    if left.is_none() || right.is_none() {
                        continue 'types_loop;
                    }

                    let left_u = left.unwrap();
                    let right_u = right.unwrap();

                    range.left = Expression::Number(left_u, None, Some(signature.clone()));
                    range.right = Expression::Number(right_u, None, Some(signature.clone()));
                }
                _ => {}
            }
        }
    }
}
