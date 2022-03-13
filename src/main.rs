mod visitor;

use std::{fmt, fs::read_to_string};

use crate::visitor::ControlFlow;

#[derive(Debug, Default)]
struct TopLevelModule {
    name: String,
    global_variables: Vec<Variable>,
    functions: Vec<Function>,
}
#[derive(Debug, Default)]
struct Function {
    return_type: String,
    name: String,
    args: Vec<Variable>,
    body: Vec<Statement>,
}

#[derive(Debug, Default)]
struct Statement {
    kind: String,
    expr: Vec<Expr>,
}

struct Expr {
    kind: String,
    left: Variable,
    operator: String,
    right: Option<Box<dyn Expressable>>,
}
impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "kind: {:?}, left: {:?}, operator: {:?} right: {:?}",
            self.kind,
            self.left,
            self.operator,
            self.right.as_ref().map(|r| r.to_expr())
        )
    }
}
impl Default for Expr {
    fn default() -> Self {
        Self {
            kind: "Expr".to_string(),
            left: Variable::default(),
            operator: "".to_string(),
            right: None,
        }
    }
}

trait Expressable {
    fn to_expr(&self) -> Expr;
}

#[derive(Debug, Default)]
struct Variable {
    name: String,
    type_name: String,
    value: Option<String>,
}

impl Expressable for Variable {
    fn to_expr(&self) -> Expr {
        Expr {
            kind: "Variable".to_string(),
            left: Variable {
                name: self.name.clone(),
                type_name: self.type_name.clone(),
                value: self.value.as_ref().cloned(),
            },
            operator: "".to_string(),
            right: None,
        }
    }
}

fn main() {
    // Read file to string
    let file = read_to_string("assets/main.c").unwrap();

    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(tree_sitter_c::language()).is_err() {
        panic!("Failed to load parser");
    };

    let ast = parser.parse(file.as_bytes(), None).unwrap();

    let mut module = TopLevelModule {
        name: "main".to_string(),
        ..Default::default()
    };

    let mut parent = String::new();
    let mut expected = String::new();
    visitor::visit_node(&ast.root_node(), |step| match step {
        visitor::Step::In(node) => {
            if !node.is_named() {
                return ControlFlow::Skip;
            }
            match node.kind() {
                "function_definition" => {
                    module.functions.push(Function::default());
                    parent = node.kind().to_string();
                    expected = "return_type".to_string();
                    ControlFlow::Continue
                }
                "primitive_type" => {
                    // Check parent, and expect
                    match parent.as_str() {
                        "function_definition" => {
                            if expected == "return_type" {
                                module.functions.last_mut().unwrap().return_type =
                                    node.utf8_text(file.as_bytes()).unwrap().to_string();
                            }
                            ControlFlow::Continue
                        }
                        "parameter_declaration" => {
                            if expected == "parameter_type" {
                                module.functions.last_mut().unwrap().args.push(Variable {
                                    type_name: node.utf8_text(file.as_bytes()).unwrap().to_string(),
                                    name: "".to_string(),
                                    value: None,
                                });
                                expected = "parameter_name".to_string();
                            }
                            ControlFlow::Continue
                        }
                        "local_declaration" => {
                            if expected == "variable_type" {
                                module
                                    .functions
                                    .last_mut()
                                    .unwrap()
                                    .body
                                    .last_mut()
                                    .unwrap()
                                    .expr
                                    .last_mut()
                                    .unwrap()
                                    .left
                                    .type_name =
                                    node.utf8_text(file.as_bytes()).unwrap().to_string();
                                expected = "variable_name".to_string();
                            }
                            ControlFlow::Continue
                        }
                        _ => ControlFlow::Quit,
                    }
                }
                "function_declarator" => {
                    parent = node.kind().to_string();
                    expected = "function_name".to_string();
                    ControlFlow::Continue
                }
                "identifier" => match parent.as_str() {
                    "function_declarator" => {
                        if expected == "function_name" {
                            module.functions.last_mut().unwrap().name =
                                node.utf8_text(file.as_bytes()).unwrap().to_string();
                        }
                        ControlFlow::Continue
                    }
                    "parameter_declaration" => {
                        if expected == "parameter_name" {
                            module
                                .functions
                                .last_mut()
                                .unwrap()
                                .args
                                .last_mut()
                                .unwrap()
                                .name = node.utf8_text(file.as_bytes()).unwrap().to_string();
                        }
                        ControlFlow::Continue
                    }
                    "local_init_declarator" => {
                        if expected == "variable_name" {
                            module
                                .functions
                                .last_mut()
                                .unwrap()
                                .body
                                .last_mut()
                                .unwrap()
                                .expr
                                .last_mut()
                                .unwrap()
                                .left
                                .name = node.utf8_text(file.as_bytes()).unwrap().to_string();
                            expected = "local_init_declarator_right".to_string();
                        }
                        ControlFlow::Continue
                    }
                    _ => ControlFlow::Quit,
                },
                "parameter_list" => {
                    parent = node.kind().to_string();
                    expected = "parameter".to_string();
                    ControlFlow::Continue
                }
                "parameter_declaration" => {
                    parent = node.kind().to_string();
                    expected = "parameter_type".to_string();
                    ControlFlow::Continue
                }
                "compound_statement" => {
                    parent = node.kind().to_string();
                    expected = "statement".to_string();
                    ControlFlow::Continue
                }
                "declaration" => match parent.as_str() {
                    "compound_statement" => {
                        if expected == "statement" {
                            module.functions.last_mut().unwrap().body.push(Statement {
                                kind: node.kind().to_string(),
                                expr: vec![Expr::default()],
                            });
                        }
                        parent = "local_declaration".to_string();
                        expected = "variable_type".to_string();
                        ControlFlow::Continue
                    }
                    _ => ControlFlow::Quit,
                },
                "init_declarator" => {
                    parent = "local_init_declarator".to_string();
                    expected = "variable_name".to_string();
                    ControlFlow::Continue
                }
                "number_literal" => {
                    if expected == "local_init_declarator_right" {
                        module
                            .functions
                            .last_mut()
                            .unwrap()
                            .body
                            .last_mut()
                            .unwrap()
                            .expr
                            .last_mut()
                            .unwrap()
                            .right = Some(Box::new(Variable {
                            name: "".to_string(),
                            type_name: "".to_string(),
                            value: Some(node.utf8_text(file.as_bytes()).unwrap().to_string()),
                        }));
                    }
                    ControlFlow::Continue
                }

                _ => ControlFlow::Quit,
            }
        }
        visitor::Step::Out(node) => {
            if !node.is_named() {
                return ControlFlow::Skip;
            }
            match node.kind() {
                "function_definition" => {
                    parent = "".to_string();
                    expected = "".to_string();
                    ControlFlow::Continue
                }
                "parameter_list" => {
                    parent = "".to_string();
                    expected = "".to_string();
                    ControlFlow::Continue
                }
                "declaration" => {
                    parent = "compound_statement".to_string();
                    expected = "statement".to_string();
                    ControlFlow::Continue
                }
                _ => ControlFlow::Quit,
            }
        }
    });
    println!("{:?}", module);
}
