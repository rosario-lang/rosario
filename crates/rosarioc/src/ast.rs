use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct CCompiler {
    pub name: String,
}

#[derive(Debug, Default, Clone)]
pub struct CResult {
    pub compiler: CCompiler,
    pub files: HashMap<CFileId, CFile>,
}

#[derive(Debug, Default, Clone)]
pub struct CFileId(pub usize);

#[derive(Debug, Default, Clone)]
pub struct CFile {
    pub path: String,
    pub includes: Vec<Include>,
    pub functions: Vec<Function>,
    pub types: HashMap<CTypeId, CType>,
}

#[derive(Debug, Default, Clone)]
pub enum CType {
    #[default]
    Void,
    Integer(usize),
    Float(usize),
    Pointer(CTypeId),
    Array(CTypeId, usize),
    Struct(CStruct),
}

#[derive(Debug, Default, Clone)]
pub struct CTypeId(pub usize);

#[derive(Debug, Default, Clone)]
pub struct CStruct {
    pub name: String,
    pub elements: Vec<CStructElement>,
}

#[derive(Debug, Default, Clone)]
pub struct CStructElement {
    pub name: String,
    pub ty: CTypeId,
}

#[derive(Debug, Default, Clone)]
pub struct Function {
    pub signature: FunctionSignature,
}

#[derive(Debug, Default, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub ty: CType,
    pub args: Vec<FunctionArgs>,
}

#[derive(Debug, Default, Clone)]
pub struct FunctionArgs {
    pub name: String,
    pub ty: CTypeId,
}

#[derive(Debug, Default, Clone)]
pub struct Include(pub String);
