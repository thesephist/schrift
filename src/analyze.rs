use crate::err::InkErr;
use crate::lex::TokKind;
use crate::parse::Node;

pub fn analyze(nodes: &mut Vec<Node>) -> Result<(), InkErr> {
    for node in nodes.iter_mut() {
        analyze_node(node)?;
    }
    return Ok(());
}

fn analyze_node(node: &mut Node) -> Result<(), InkErr> {
    match node {
        Node::UnaryExpr { op: _, arg } => {
            analyze_node(arg)?;
        }
        Node::BinaryExpr {
            op: TokKind::DefineOp,
            left,
            right,
        } => {
            analyze_node(right)?;
            match *(left.clone()) {
                Node::Ident(_) => (),
                Node::BinaryExpr {
                    op: TokKind::AccessorOp,
                    left: mut comp_left,
                    right: mut comp_right,
                } => {
                    analyze_node(&mut comp_left)?;
                    analyze_node(&mut comp_right)?;
                }
                _ => return Err(InkErr::InvalidAssignment),
            }
        }
        Node::BinaryExpr { op: _, left, right } => {
            analyze_node(left)?;
            analyze_node(right)?;
        }
        Node::FnCall { func, args } => {
            analyze_node(func)?;
            for arg in args.iter_mut() {
                analyze_node(arg)?;
            }
        }
        Node::MatchClause { target, expr } => {
            analyze_node(target)?;
            analyze_node(expr)?;
        }
        Node::MatchExpr { cond, clauses } => {
            analyze_node(cond)?;
            for clause in clauses.iter_mut() {
                analyze_node(clause)?;
            }
        }
        Node::ExprList(exprs) => {
            for expr in exprs.iter_mut() {
                analyze_node(expr)?;
            }
        }

        Node::EmptyIdent => (),
        Node::Ident(_) => (),
        Node::NumberLiteral(_) => (),
        Node::StringLiteral(_) => (),
        Node::BooleanLiteral(_) => (),

        Node::ObjectLiteral(entries) => {
            for entry in entries.iter_mut() {
                analyze_node(entry)?;
            }
        }
        Node::ObjectEntry { key, val } => {
            analyze_node(key)?;
            analyze_node(val)?;
        }
        Node::ListLiteral(items) => {
            for item in items.iter_mut() {
                analyze_node(item)?;
            }
        }
        Node::FnLiteral { args, body } => {
            for arg in args.iter_mut() {
                analyze_node(arg)?;
            }
            analyze_node(body)?;
        }
    }
    return Ok(());
}
