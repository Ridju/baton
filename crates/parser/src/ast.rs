#[derive(Debug, PartialEq)]
pub enum Type<'a> {
    Int,
    Bool,
    Float,
    String,
    Struct(&'a str),
    Array(Box<Type<'a>>),
}

#[derive(Debug, PartialEq)]
pub struct Parameter<'a> {
    pub name: &'a str,
    pub param_typ: Type<'a>,
}

#[derive(Debug, PartialEq)]
pub struct FunctionDeclData<'a> {
    pub name: &'a str,
    pub return_type: Type<'a>,
    pub parameter: Vec<Parameter<'a>>,
    pub body: Box<AstNode<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    DoubleEqual,
    LessThan,
    GreaterThan,
    LessOrEqual,
    GreaterOrEqual,
}

#[derive(Debug, PartialEq)]
pub struct BinaryExpData<'a> {
    pub left: Box<AstNode<'a>>,
    pub right: Box<AstNode<'a>>,
    pub operator: BinaryOperator,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct VarDeclData<'a> {
    pub name: &'a str,
    pub var_typ: Type<'a>,
    pub initializer: Option<Box<AstNode<'a>>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct AssignmentData<'a> {
    pub target: Box<AstNode<'a>>,
    pub value: Box<AstNode<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct IfElseData<'a> {
    pub condition_expr: Box<AstNode<'a>>,
    pub if_branch: Box<AstNode<'a>>,
    pub else_branch: Option<Box<AstNode<'a>>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct CallData<'a> {
    pub name: &'a str,
    pub arguments: Vec<AstNode<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct MemberAccessData<'a> {
    pub object: Box<AstNode<'a>>,
    pub member: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct ArrayIndexData<'a> {
    pub array: Box<AstNode<'a>>,
    pub index: Box<AstNode<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct WhileLoopData<'a> {
    pub condition_expr: Box<AstNode<'a>>,
    pub body: Box<AstNode<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct StructDeclData<'a> {
    pub name: &'a str,
    pub fields: Vec<Parameter<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct BlockStatementData<'a> {
    pub statements: Vec<AstNode<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct ReturnData<'a> {
    pub value: Option<Box<AstNode<'a>>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub enum AstNode<'a> {
    Programm(Vec<AstNode<'a>>),
    FunctionDecl(FunctionDeclData<'a>),
    ReturnStatement(ReturnData<'a>),
    IntLiteralExpr(i16),
    BoolLiteralExpr(bool),
    BlockStatement(BlockStatementData<'a>),
    BinaryExpr(BinaryExpData<'a>),
    IfElseStatement(IfElseData<'a>),
    VariableExpr(&'a str),
    VarDeclStatement(VarDeclData<'a>),
    AssignmentStatement(AssignmentData<'a>),
    CallExpr(CallData<'a>),
    FloatLiteralExpr(f64),
    StringLiteralExpr(&'a str),
    StructDecl(StructDeclData<'a>),
    MemberAccessExpr(MemberAccessData<'a>),
    ArrayIndexExpr(ArrayIndexData<'a>),
    WhileStatement(WhileLoopData<'a>),
    PrintStatement(Box<AstNode<'a>>),
}
