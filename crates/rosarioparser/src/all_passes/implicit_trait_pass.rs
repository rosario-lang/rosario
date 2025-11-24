use std::collections::BTreeMap;

use crate::{
    ast::{
        Expression, Function, ImplSignature, IsMutable, Package, ReturnType, RosarioType,
        Signature, Trait, TraitSignature, TypeContent, TypeImplementation, TypeSignature,
    },
    parser::Parser,
};

pub fn find_original_rosario_type_by_signature(
    sig: &TypeSignature,
    packages: &BTreeMap<String, Package>,
) -> RosarioType {
    for (_name, package) in packages {
        for (signature, ty) in &package.file.types {
            if signature == sig {
                match &ty.content {
                    TypeContent::TypeRef(new_signature) => {
                        return find_original_rosario_type_by_signature(&new_signature, packages);
                    }
                    _ => return ty.clone(),
                }
            }
        }

        for (signature, ty) in &package.file.public_types {
            if signature == sig {
                match &ty.content {
                    TypeContent::TypeRef(new_signature) => {
                        return find_original_rosario_type_by_signature(&new_signature, packages);
                    }
                    _ => return ty.clone(),
                }
            }
        }
    }

    todo!()
}

pub fn implicit_trait_pass(parser: &mut Parser) {
    let why_do_i_have_to_do_this = parser.result.packages.clone();

    for i in &mut parser.result.packages {
        for (signature, ty) in &i.1.file.public_types {
            match &ty.content {
                TypeContent::Range(range) => {
                    let range_trait = i
                        .1
                        .file
                        .implementations
                        .entry(ImplSignature {
                            impl_of: Some(find_trait_by_name("Range", &why_do_i_have_to_do_this)),
                            impl_for: signature.clone(),
                        })
                        .or_insert(TypeImplementation::default());

                    range_trait.public_functions.push(Function {
                        signature: Signature {
                            name: "Min".to_string(),
                            arguments: vec![],
                            return_type: ReturnType::Type(IsMutable::False, signature.clone()),
                        },
                        body: range.left.clone(),
                    });

                    range_trait.public_functions.push(Function {
                        signature: Signature {
                            name: "Max".to_string(),
                            arguments: vec![],
                            return_type: ReturnType::Type(IsMutable::False, signature.clone()),
                        },
                        body: range.right.clone(),
                    });
                }
                TypeContent::TypeRef(ref_signature) => {
                    let ty = find_original_rosario_type_by_signature(
                        ref_signature,
                        &why_do_i_have_to_do_this,
                    );

                    match ty.content {
                        TypeContent::Range(range) => {
                            let range_trait =
                                i.1.file
                                    .implementations
                                    .entry(ImplSignature {
                                        impl_of: Some(find_trait_by_name(
                                            "Range",
                                            &why_do_i_have_to_do_this,
                                        )),
                                        impl_for: signature.clone(),
                                    })
                                    .or_insert(TypeImplementation::default());

                            range_trait.public_functions.push(Function {
                                signature: Signature {
                                    name: "Min".to_string(),
                                    arguments: vec![],
                                    return_type: ReturnType::Type(
                                        IsMutable::False,
                                        signature.clone(),
                                    ),
                                },
                                body: range.left.clone(),
                            });

                            range_trait.public_functions.push(Function {
                                signature: Signature {
                                    name: "Max".to_string(),
                                    arguments: vec![],
                                    return_type: ReturnType::Type(
                                        IsMutable::False,
                                        signature.clone(),
                                    ),
                                },
                                body: range.right.clone(),
                            });
                        }
                        _ => todo!("{:?}", ty),
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn find_trait_by_name(tr: &str, packages: &BTreeMap<String, Package>) -> TraitSignature {
    for (_name, pkg) in packages {
        for (signature, _) in &pkg.file.public_traits {
            if signature.name == tr {
                return signature.clone();
            }
        }
    }

    panic!("Trait \"{}\" not found.", tr);
}
