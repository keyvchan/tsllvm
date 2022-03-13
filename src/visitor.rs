use tree_sitter::Node;
pub enum ControlFlow {
    Continue,
    Skip,
    Quit,
}

pub enum Step<'a> {
    In(Node<'a>),
    Out(Node<'a>),
}

pub fn visit_node<'a, 'ctx, F>(node: &Node<'a>, mut visitor: F)
where
    F: FnMut(Step<'a>) -> ControlFlow,
{
    let mut cursor = node.walk();
    visitor(Step::In(cursor.node()));
    let mut recurse = true;
    loop {
        if recurse && cursor.goto_first_child() {
            recurse = match visitor(Step::In(cursor.node())) {
                ControlFlow::Continue => true,
                ControlFlow::Skip => false,
                ControlFlow::Quit => return,
            };
        } else {
            visitor(Step::Out(cursor.node()));
            if cursor.goto_next_sibling() {
                recurse = match visitor(Step::In(cursor.node())) {
                    ControlFlow::Continue => true,
                    ControlFlow::Skip => false,
                    ControlFlow::Quit => return,
                };
            } else if cursor.goto_parent() {
                recurse = false;
            } else {
                break;
            }
        }
    }
}
