use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub struct Ast {
    pub definitions: BTreeMap<DefinitionSignature, ExpressionId>,
    pub types: BTreeMap<TypeSignature, RosarioType>,
    pub uses: BTreeMap<String, Vec<TypeSignature>>,
    pub expressions: BTreeMap<ExpressionId, Expression>,
    pub variables: BTreeMap<DefinitionSignature, BTreeMap<VariableId, Variable>>,
    expression_id: ExpressionId,
    variable_id: VariableId,
}

impl Ast {
    pub fn new_body(&mut self, contents: Vec<ExpressionId>) -> ExpressionId {
        self.expression_id.0 += 1;

        self.expressions
            .insert(self.expression_id, Expression::Body(Body { contents }));

        self.expression_id
    }

    pub fn new_definition(&mut self, signature: DefinitionSignature, body: ExpressionId) {
        self.definitions.insert(signature, body);
    }

    pub fn new_type(&mut self, name: String, owner: String, ty: RosarioType) {
        self.types.insert(TypeSignature { owner, name }, ty);
    }

    pub fn new_number(&mut self, natural: u128, decimal: Option<u128>) -> ExpressionId {
        self.expression_id.0 += 1;

        self.expressions
            .insert(self.expression_id, Expression::Number(natural, decimal));

        self.expression_id
    }

    pub fn new_binary_operator(
        &mut self,
        op: BinOpType,
        left: ExpressionId,
        right: ExpressionId,
    ) -> ExpressionId {
        self.expression_id.0 += 1;

        self.expressions.insert(
            self.expression_id,
            Expression::BinaryOperation(BinOp { op, left, right }),
        );

        self.expression_id
    }

    pub fn find_expression(&self, id: ExpressionId) -> Option<&Expression> {
        self.expressions.get(&id)
    }

    pub fn find_mut_expression(&mut self, id: ExpressionId) -> Option<&mut Expression> {
        self.expressions.get_mut(&id)
    }

    pub fn new_variable(
        &mut self,
        signature: DefinitionSignature,
        variable: Variable,
    ) -> ExpressionId {
        self.expression_id.0 += 1;
        self.variable_id.0 += 1;

        let def = self.variables.entry(signature).or_insert(BTreeMap::new());
        def.insert(self.variable_id, variable);

        self.expression_id
    }

    pub fn find_type_signature_owner(&self, name: String) -> String {
        for (signature, _) in &self.types {
            if signature.name == name {
                return signature.owner.clone();
            }
        }

        String::new()
    }
}

#[derive(Debug, Default, Clone)]
pub struct RosarioType {
    pub generics: Vec<Generic>,
    pub ty: TypeBody,
}

#[derive(Debug, Default, Clone)]
pub enum TypeBody {
    #[default]
    Unknown,
    Range(Range),
}

#[derive(Debug, Default, Clone)]
pub struct Range {
    pub min: ExpressionId,
    pub max: ExpressionId,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DefinitionOwner {
    #[default]
    Unknown,
    Path(String),
    Type(TypeSignature),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DefinitionSignature {
    pub owner: DefinitionOwner,
    pub name: String,
    pub args: Vec<VariableId>,
    pub return_type: Option<TypeSignature>,
}

#[derive(Debug, Default, Clone)]
pub struct Procedure {
    pub signature: DefinitionSignature,
    pub body: ExpressionId,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExpressionId(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VariableId(pub u64);

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable {
    pub name: String,
    pub ty: TypeSignature,
    pub variable_type: VariableType,
    pub generics: Vec<Generic>,
    pub initializer: Option<ExpressionId>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expression {
    #[default]
    None,
    Number(u128, Option<u128>),
    BinaryOperation(BinOp),
    NewVariable(VariableId),
    Variable(VariableId),
    Body(Body),
}

impl Expression {
    pub fn get_binary_operator(&self) -> Option<&BinOp> {
        match self {
            Expression::BinaryOperation(b) => Some(&b),
            _ => None,
        }
    }

    pub fn get_mut_binary_operator(&mut self) -> Option<&mut BinOp> {
        match self {
            Expression::BinaryOperation(b) => Some(b),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Body {
    pub contents: Vec<ExpressionId>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinOp {
    pub op: BinOpType,
    pub left: ExpressionId,
    pub right: ExpressionId,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinOpType {
    #[default]
    Unknown,
    Sub,
    Add,
    Div,
    Mul,
    ToThePowerOf,
    AddEquals,
    SubEquals,
    MulEquals,
    DivEquals,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeSignature {
    pub owner: String,
    pub name: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Generic {
    pub name: String,
    pub end: GenericEnd,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GenericEnd {
    #[default]
    Nothing,
    Comma,
    Of,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariableType {
    #[default]
    Unknown,
    Value,
    Reference,
    MutableValue,
    MutableReference,
}
