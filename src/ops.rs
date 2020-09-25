use serde::Serialize;
use std::path::PathBuf;

use crate::checker::Checker;
use crate::getter::Getter;
use crate::node::Node;
use crate::spaces::SpaceKind;

use crate::halstead::{Halstead, HalsteadMaps};

use crate::dump_ops::*;
use crate::traits::*;

/// All operands and operators of a space.
#[derive(Debug, Clone, Serialize)]
pub struct Ops {
    /// The name of a function space.
    ///
    /// If `None`, an error is occured in parsing
    /// the name of a function space.
    pub name: Option<String>,
    /// The first line of a function space.
    pub start_line: usize,
    /// The last line of a function space.
    pub end_line: usize,
    /// The space kind.
    pub kind: SpaceKind,
    /// All subspaces contained in a function space.
    pub spaces: Vec<Ops>,
    /// All operands of a space.
    pub operands: Vec<String>,
    /// All operators of a space.
    pub operators: Vec<String>,
}

impl Ops {
    fn new<T: Getter>(node: &Node, code: &[u8], kind: SpaceKind) -> Self {
        let (start_position, end_position) = match kind {
            SpaceKind::Unit => {
                if node.object().child_count() == 0 {
                    (0, 0)
                } else {
                    (
                        node.object().start_position().row + 1,
                        node.object().end_position().row,
                    )
                }
            }
            _ => (
                node.object().start_position().row + 1,
                node.object().end_position().row + 1,
            ),
        };
        Self {
            name: T::get_func_space_name(&node, code).map(|name| name.to_string()),
            spaces: Vec::new(),
            kind,
            start_line: start_position,
            end_line: end_position,
            operators: Vec::new(),
            operands: Vec::new(),
        }
    }

    pub(crate) fn merge_ops(&mut self, other: &Ops) {
        self.operands.extend_from_slice(&other.operands);
        self.operators.extend_from_slice(&other.operators);
    }
}

#[derive(Debug, Clone)]
struct State<'a> {
    ops: Ops,
    halstead_maps: HalsteadMaps<'a>,
}

fn finalize<'a, T: ParserTrait>(state_stack: &mut Vec<State<'a>>, diff_level: usize) {
    for _ in 0..diff_level {
        if state_stack.len() <= 1 {
            break;
        }

        let state = state_stack.pop().unwrap();
        let last_state = state_stack.last_mut().unwrap();

        // Merge Halstead maps
        last_state.halstead_maps.merge(&state.halstead_maps);

        // Merge operands and operators between spaces
        last_state.ops.merge_ops(&state.ops);
        last_state.ops.spaces.push(state.ops);
    }
}

/// Retrieves all the operators and operands of a code.
///
/// If `None`, it was not possible to retrieve the operators and operands
/// of a code.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
///
/// use rust_code_analysis::{operands_and_operators, CppParser, ParserTrait};
///
/// # fn main() {
/// let source_code = "int a = 42;";
///
/// // The path to a dummy file used to contain the source code
/// let path = PathBuf::from("foo.c");
/// let source_as_vec = source_code.as_bytes().to_vec();
///
/// // The parser of the code, in this case a CPP parser
/// let parser = CppParser::new(source_as_vec, &path, None);
///
/// // Returns the operands and operators of each space in a code.
/// operands_and_operators(&parser, &path).unwrap();
/// # }
/// ```
pub fn operands_and_operators<'a, T: ParserTrait>(parser: &'a T, path: &'a PathBuf) -> Option<Ops> {
    let code = parser.get_code();
    let node = parser.get_root();
    let mut cursor = node.object().walk();
    let mut stack = Vec::new();
    let mut children = Vec::new();
    let mut state_stack: Vec<State> = Vec::new();
    let mut last_level = 0;

    stack.push((node, 0));

    while let Some((node, level)) = stack.pop() {
        if level < last_level {
            finalize::<T>(&mut state_stack, last_level - level);
            last_level = level;
        }

        let kind = T::Getter::get_space_kind(&node);

        let func_space = T::Checker::is_func(&node) || T::Checker::is_func_space(&node);

        let new_level = if func_space {
            let state = State {
                ops: Ops::new::<T::Getter>(&node, code, kind),
                halstead_maps: HalsteadMaps::new(),
            };
            state_stack.push(state);
            last_level = level + 1;
            last_level
        } else {
            level
        };

        if let Some(state) = state_stack.last_mut() {
            T::Halstead::compute(&node, code, &mut state.halstead_maps);
            state.ops.operators = state
                .halstead_maps
                .operators
                .keys()
                .map(|k| T::Getter::get_operator_id_as_str(*k).to_string())
                .collect();
            state.ops.operands = state
                .halstead_maps
                .operands
                .keys()
                .map(|k| std::str::from_utf8(k).unwrap().to_string())
                .collect();
        }

        cursor.reset(node.object());
        if cursor.goto_first_child() {
            loop {
                children.push((Node::new(cursor.node()), new_level));
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            for child in children.drain(..).rev() {
                stack.push(child);
            }
        }
    }

    finalize::<T>(&mut state_stack, std::usize::MAX);

    state_stack.pop().map(|mut state| {
        state.ops.name = path.to_str().map(|name| name.to_string());
        state.ops
    })
}

/// Configuration options for retrieving
/// all the operands and operators in a code.
pub struct OpsCfg {
    /// Path to the file containing the code.
    pub path: PathBuf,
}

pub struct OpsCode {
    _guard: (),
}

impl Callback for OpsCode {
    type Res = std::io::Result<()>;
    type Cfg = OpsCfg;

    fn call<T: ParserTrait>(cfg: Self::Cfg, parser: &T) -> Self::Res {
        if let Some(ops) = operands_and_operators(parser, &cfg.path) {
            dump_ops(&ops)
        } else {
            Ok(())
        }
    }
}
