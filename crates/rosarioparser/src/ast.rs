use std::collections::BTreeMap;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    #[default]
    Empty,
    Number(i128, Option<i128>, Option<TypeSignature>),
    Negative(Box<Expression>),
    Positive(Box<Expression>),
    Not(Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Let(Let),
    Body(Body),
    Match(Match),
    Variable(String),
    FunctionCall(FunctionCall),
    StaticString(String),
    MatchAll,
    If(If),
    Condition(Condition),
    Parenthesis(Box<Expression>),
    ToThePowerOf(Box<Expression>, Box<Expression>),
    SelfVariable,
    AccessFunction(Box<Expression>, FunctionCall),
    AccessVariable(Box<Expression>, String),
    AccessTypeImplementation(TypeSignature, Box<Expression>),
    Clang(Box<Expression>),
    NewEnum(NewEnum),
    Return(Box<Expression>),
}

impl Expression {
    pub fn get_type(&self) -> Option<TypeSignature> {
        match self {
            Self::Number(_, _, ty) => ty.clone(),
            Self::Let(l) => Some(l.ty.clone()),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct NewEnum {
    pub ty: TypeSignature,
    pub item: usize,
    pub right: Box<Expression>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum ConditionType {
    #[default]
    Equals, // "=="
    LessThan,         // "<"
    MoreThan,         // ">"
    LessThanOrEquals, // "<="
    MoreThanOrEquals, // ">="
    NotEquals,        // "!="
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Condition {
    pub left: Box<Expression>,
    pub cond: ConditionType,
    pub right: Box<Expression>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct If {
    pub statements: Vec<(Expression, Body)>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Match {
    pub condition: Box<Expression>,
    pub elements: Vec<(MatchOption, Expression)>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum MatchOption {
    #[default]
    AllCombinations,
    SingleIdentifier(String, Vec<String>),
    MultipleIdentifiers(Vec<(String, Vec<String>)>),
    SingleNumber(usize, Option<usize>),
    MultipleNumbers(Vec<(usize, Option<usize>)>),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Let {
    pub is_mutable: bool,
    pub name: String,
    pub ty: TypeSignature,
    pub initializer: Box<Expression>,
}

#[derive(Debug, Default, Clone)]
pub struct Procedure {
    pub signature: Signature,
    pub body: Expression,
}

#[derive(Debug, Default, Clone)]
pub struct Function {
    pub signature: Signature,
    pub body: Expression,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Body {
    pub content: Vec<Expression>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum IsMutable {
    #[default]
    False,
    True,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum ReturnType {
    #[default]
    None,
    Type(IsMutable, TypeSignature),
    Generic(IsMutable, String),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Signature {
    pub name: String,
    pub arguments: Vec<Argument>,
    pub return_type: ReturnType,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Argument {
    pub argument_type: ArgumentType,
    pub is_mutable: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum ArgumentType {
    #[default]
    Unknown,
    Variable(String, TypeSignature),
    SelfVariable,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Generic {
    pub name: String,
    pub ends_with: GenericEnd,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GenericEnd {
    #[default]
    None,
    Comma,
    Of,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum TypeContent {
    #[default]
    None,
    Range(Range),
    Enum(Enum),
    Modulo(Expression),
    TypeRef(TypeSignature),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Enum {
    pub contents: BTreeMap<usize, (String, Vec<EnumArgument>)>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum EnumArgument {
    #[default]
    Unknown,
    Generic(String),
    Type(TypeSignature),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Range {
    pub is_signed: bool,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, Default, Clone)]
pub struct Package {
    pub packages: Vec<(String, bool)>,
    pub c_includes: Vec<String>,
    pub file: File,
}

#[derive(Debug, Default, Clone)]
pub struct ParsedResult {
    pub packages: BTreeMap<String, Package>,
}

#[derive(Debug, Default, Clone)]
pub struct TypeImplementation {
    pub procedures: Vec<Procedure>,
    pub public_procedures: Vec<Procedure>,
    pub functions: Vec<Function>,
    pub public_functions: Vec<Function>,
    pub operators: Vec<Operator>,
    pub mutable_operators: Vec<Operator>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Operator {
    pub ty: String,
    pub signature: Signature,
    pub body: Expression,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeSignature {
    pub name: String,
    pub generics: Vec<Generic>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TraitSignature {
    pub name: String,
    pub generics: Vec<Generic>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct RosarioType {
    pub traits: Vec<String>,
    pub content: TypeContent,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Trait {
    pub signatures: Vec<Signature>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ImplSignature {
    pub impl_of: Option<TraitSignature>,
    pub impl_for: TypeSignature,
}

#[derive(Debug, Default, Clone)]
pub struct File {
    pub types: BTreeMap<TypeSignature, RosarioType>,
    pub public_types: BTreeMap<TypeSignature, RosarioType>,
    pub implementations: BTreeMap<ImplSignature, TypeImplementation>,
    pub traits: BTreeMap<TraitSignature, Trait>,
    pub public_traits: BTreeMap<TraitSignature, Trait>,
    pub procedures: Vec<Procedure>,
    pub public_procedures: Vec<Procedure>,
    pub functions: Vec<Function>,
    pub public_functions: Vec<Function>,
    pub public_packages: Vec<String>,
}
