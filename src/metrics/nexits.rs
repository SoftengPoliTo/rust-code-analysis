use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use std::fmt;

use crate::checker::Checker;
use crate::*;

/// The `NExit` metric.
///
/// This metric counts the number of possible exit points
/// from a function/method.
#[derive(Debug, Clone)]
pub struct Stats {
    fn_nexits: usize,
    closure_nexits: usize,
    total_space_functions: usize,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            fn_nexits: 0,
            closure_nexits: 0,
            total_space_functions: 1,
        }
    }
}

impl Serialize for Stats {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut st = serializer.serialize_struct("nexits", 4)?;
        st.serialize_field("functions", &self.fn_exits())?;
        st.serialize_field("closures", &self.closure_exits())?;
        st.serialize_field("total", &self.total())?;
        st.serialize_field("average", &self.nexits_average())?;
        st.end()
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "functions: {}, closures: {}, total: {}, average: {}",
            self.fn_exits(),
            self.closure_exits(),
            self.total(),
            self.nexits_average()
        )
    }
}

impl Stats {
    /// Merges a second `NExit` metric into the first one
    pub fn merge(&mut self, other: &Stats) {
        self.fn_nexits += other.fn_nexits;
        self.closure_nexits += other.closure_nexits;
    }

    /// Returns the number of exit points of a function.
    pub fn fn_exits(&self) -> f64 {
        self.fn_nexits as f64
    }

    /// Returns the number of exit points of a closure.
    #[inline(always)]
    pub fn closure_exits(&self) -> f64 {
        self.closure_nexits as f64
    }

    /// Returns the total number of exit points of each function and
    /// closure in a space.
    #[inline(always)]
    pub fn total(&self) -> f64 {
        self.fn_exits() + self.closure_exits()
    }

    /// Returns the `NExit` metric average value
    ///
    /// This value is computed dividing the `NExit` value
    /// for the total number of functions/closures in a space.
    pub fn nexits_average(&self) -> f64 {
        self.total() / self.total_space_functions as f64
    }

    pub(crate) fn finalize(&mut self, total_space_functions: usize) {
        self.total_space_functions = total_space_functions;
    }
}

#[doc(hidden)]
pub trait Exit
where
    Self: Checker,
{
    fn compute(_node: &Node, _stats: &mut Stats) {}
}

impl Exit for PythonCode {
    fn compute(node: &Node, stats: &mut Stats) {
        if let Python::ReturnStatement = node.object().kind_id().into() {
            stats.fn_nexits += 1;
        }

        if let Python::Lambda = node.object().kind_id().into() {
            stats.closure_nexits += 1;
        }
    }
}

impl Exit for MozjsCode {
    fn compute(node: &Node, stats: &mut Stats) {
        if let Mozjs::ReturnStatement = node.object().kind_id().into() {
            stats.fn_nexits += 1;
        }
    }
}

impl Exit for JavascriptCode {
    fn compute(node: &Node, stats: &mut Stats) {
        if let Javascript::ReturnStatement = node.object().kind_id().into() {
            stats.fn_nexits += 1;
        }
    }
}

impl Exit for TypescriptCode {
    fn compute(node: &Node, stats: &mut Stats) {
        if let Typescript::ReturnStatement = node.object().kind_id().into() {
            stats.fn_nexits += 1;
        }
    }
}

impl Exit for TsxCode {
    fn compute(node: &Node, stats: &mut Stats) {
        if let Tsx::ReturnStatement = node.object().kind_id().into() {
            stats.fn_nexits += 1;
        }
    }
}

impl Exit for RustCode {
    fn compute(node: &Node, stats: &mut Stats) {
        if Self::is_func(node) {
            if let Some(block) = node.object().child_by_field_name("block") {
                if block
                    .object()
                    .child_by_field_name("return_expression")
                    .is_some()
                {
                    stats.fn_nexits += 1;
                }
            }
            if node.object().child_by_field_name("return_type").is_some() {
                stats.fn_nexits += 1;
            }
        }

        if Self::is_closure(node) && node.object().child_by_field_name("->").is_some() {
            stats.closure_nexits += 1;
        }
    }
}

impl Exit for CppCode {
    fn compute(node: &Node, stats: &mut Stats) {
        if let Cpp::ReturnStatement = node.object().kind_id().into() {
            stats.fn_nexits += 1;
        }

        //if Self::is_closure(Node)
    }
}

impl Exit for PreprocCode {}
impl Exit for CcommentCode {}
impl Exit for CSharpCode {}
impl Exit for JavaCode {}
impl Exit for GoCode {}
impl Exit for CssCode {}
impl Exit for HtmlCode {}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn python_single_function() {
        check_metrics!(
            "def f(a, b):
                 if a:
                     return a",
            "foo.py",
            PythonParser,
            nexits,
            [
                (fn_exits, 1, usize),
                (closure_exits, 0, usize),
                (total, 1, usize)
            ],
            [(nexits_average, 1.0)] // 1 function
        );
    }

    #[test]
    fn rust_single_function() {
        check_metrics!(
            "fn f(a: bool, b: usize) {
                 if a {
                     return a;
                }
             }",
            "foo.rs",
            RustParser,
            nexits,
            [
                (fn_exits, 1, usize),
                (closure_exits, 0, usize),
                (total, 1, usize)
            ],
            [(nexits_average, 1.0)] // 1 function
        );
    }

    #[test]
    fn c_single_function() {
        check_metrics!(
            "int f(int a, int b) {
                 if (a) {
                     return a;
                }
             }",
            "foo.c",
            CppParser,
            nexits,
            [
                (fn_exits, 1, usize),
                (closure_exits, 0, usize),
                (total, 1, usize)
            ],
            [(nexits_average, 1.0)] // 1 function
        );
    }

    #[test]
    fn javascript_single_function() {
        check_metrics!(
            "function f(a, b) {
                 return a * b;
             }",
            "foo.js",
            JavascriptParser,
            nexits,
            [
                (fn_exits, 1, usize),
                (closure_exits, 0, usize),
                (total, 1, usize)
            ],
            [(nexits_average, 2.0)] // 1 function
        );
    }

    #[test]
    fn python_single_lambda() {
        check_metrics!(
            "bar = lambda a: True",
            "foo.py",
            PythonParser,
            nexits,
            [
                (fn_exits, 0, usize),
                (closure_exits, 1, usize),
                (total, 1, usize)
            ],
            [(exits_average, 1.0)] // 1 lambda
        );
    }

    #[test]
    fn rust_closures() {
        check_metrics!(
            "let bar = |i: i32| -> i32 { i + 1 };
             let bar = |i: i32| -> i32 { return i + 1 };",
            "foo.rs",
            RustParser,
            nexits,
            [
                (fn_exits, 0, usize),
                (closure_exits, 2, usize),
                (total, 2, usize)
            ],
            [(nexits_average, 1.0)] // 2 lambdas
        );
    }

    #[test]
    fn cpp_single_lambda() {
        check_metrics!(
            "auto bar = [](int x, int y) -> int { return x + y; };",
            "foo.cpp",
            CppParser,
            nexits,
            [
                (fn_exits, 0, usize),
                (closure_exits, 1, usize),
                (total, 1, usize)
            ],
            [(nexits_average, 1.0)] // 1 lambda
        );
    }

    #[test]
    fn javascript_single_closure() {
        check_metrics!(
            "var bar = function (a, b) {return a + b};",
            "foo.js",
            JavascriptParser,
            nexits,
            [
                (fn_exits, 0, usize),
                (closure_exits, 1, usize),
                (total, 1, usize)
            ],
            [(nexits_average, 2.0)] // 1 lambda
        );
    }

    #[test]
    fn python_functions() {
        check_metrics!(
            "def f(a, b):
                 if a:
                     return a
            def f(a, b):
                 if b:
                     return b",
            "foo.py",
            PythonParser,
            nexits,
            [
                (fn_exits, 2, usize),
                (closure_exits, 0, usize),
                (total, 2, usize)
            ],
            [(nexits_average, 1.0)] // 2 functions
        );
    }

    #[test]
    fn rust_functions() {
        check_metrics!(
            "fn f(a: bool, b: usize) {
                 if a {
                     a
                }
             }
             fn f1(a: bool, b: usize) {
                 if a {
                     return a;
                }
             }",
            "foo.rs",
            RustParser,
            nexits,
            [
                (fn_exits, 2, usize),
                (closure_exits, 0, usize),
                (total, 2, usize)
            ],
            [(nexits_average, 1.0)] // 2 functions
        );
    }

    #[test]
    fn c_functions() {
        check_metrics!(
            "int f(int a, int b) {
                 if (a) {
                     return a;
                }
             }
             int f1(int a, int b) {
                 if (a) {
                     return a;
                }
             }",
            "foo.c",
            CppParser,
            nexits,
            [
                (fn_exits, 2, usize),
                (closure_exits, 0, usize),
                (total, 2, usize)
            ],
            [(nexits_average, 1.0)] // 2 functions
        );
    }

    #[test]
    fn javascript_functions() {
        check_metrics!(
            "function f(a, b) {
                 return a * b;
             }
             function f1(a, b) {
                 return a * b;
             }",
            "foo.js",
            JavascriptParser,
            nargs,
            [
                (fn_args, 4, usize),
                (closure_args, 0, usize),
                (total, 4, usize)
            ],
            [(nargs_average, 2.0)] // 2 functions
        );

        check_metrics!(
            "function f(a, b) {
                 return a * b;
             }
             function f1(a, b, c) {
                 return a * b;
             }",
            "foo.js",
            JavascriptParser,
            nargs,
            [
                (fn_args, 5, usize),
                (closure_args, 0, usize),
                (total, 5, usize)
            ],
            [(nargs_average, 2.5)] // 2 functions
        );
    }

    #[test]
    fn python_nested_functions() {
        check_metrics!(
            "def f(a, b):
                 def foo(a):
                     if a:
                         return 1
                 bar = lambda a: lambda b: b or True or True
                 return bar(foo(a))(a)",
            "foo.py",
            PythonParser,
            nargs,
            [
                (fn_args, 3, usize),
                (closure_args, 2, usize),
                (total, 5, usize)
            ],
            [(nargs_average, 1.25)] // 2 functions + 2 lambdas = 4
        );
    }

    #[test]
    fn rust_nested_functions() {
        check_metrics!(
            "fn f(a: i32, b: i32) -> i32 {
                 fn foo(a: i32) -> i32 {
                     return a;
                 }
                 let bar = |a: i32, b: i32| -> i32 { a + 1 };
                 let bar1 = |b: i32| -> i32 { b + 1 };
                 return bar(foo(a), a);
             }",
            "foo.rs",
            RustParser,
            nargs,
            [
                (fn_args, 3, usize),
                (closure_args, 3, usize),
                (total, 6, usize)
            ],
            [(nargs_average, 1.5)] // 2 functions + 1 lambda = 3
        );
    }

    #[test]
    fn cpp_nested_functions() {
        check_metrics!(
            "int f(int a, int b, int c) {
                 auto foo = [](int x) -> int { return x; };
                 auto bar = [](int x, int y) -> int { return x + y; };
                 return bar(foo(a), a);
             }",
            "foo.cpp",
            CppParser,
            nargs,
            [
                (fn_args, 3, usize),
                (closure_args, 3, usize),
                (total, 6, usize)
            ],
            [(nargs_average, 2.0)] // 1 function + 2 lambdas = 3
        );
    }

    #[test]
    fn javascript_nested_functions() {
        check_metrics!(
            "function f(a, b) {
                 function foo(a) {
                     return a;
                 }
                 var bar = function (a, b) {return a + b};
                 var bar1 = function (a) {return a};
                 return bar(foo(a), a);
             }",
            "foo.js",
            JavascriptParser,
            nargs,
            [
                (fn_args, 3, usize),
                (closure_args, 3, usize),
                (total, 6, usize)
            ],
            [(nargs_average, 1.5)] // 2 functions + 2 lambdas = 4
        );
    }
}
