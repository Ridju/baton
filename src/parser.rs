use crate::scanner::Token;

#[derive(Debug, PartialEq)]
enum Type {
    Int,
}

#[derive(Debug, PartialEq)]
struct Parameter {
    name: String,
    param_typ: Type,
}

#[derive(Debug, PartialEq)]
struct FunctionDeclData {
    name: String,
    return_type: Type,
    parameter: Vec<Parameter>,
    body: Box<AstNode>,
}

#[derive(Debug, PartialEq)]
enum AstNode {
    Programm(Vec<AstNode>),
    FunctionDecl(FunctionDeclData),
    ReturnStatement(Box<AstNode>),
    IntLiteralExpr(i16),
    BlockStatement(Vec<AstNode>),
}


