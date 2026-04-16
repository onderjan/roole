use std::{ops::ControlFlow, path::PathBuf, str::FromStr};

use aws_smt_ir::{
    Constant, Symbol, SyntaxBuilder,
    smt2parser::{
        CommandStream,
        concrete::{Command, Identifier, QualIdentifier, Sort, Term},
        visitors::{AttributeValue, FunctionDec, Index},
    },
};
use indexmap::IndexMap;
use itertools::Itertools;
use num::{BigUint, Zero};

use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
        value::ThreeValued,
    },
    exit::RooleResult,
    problem::{
        Problem,
        formula::{
            FormulaId, OperationId, Variable, VariableId,
            operation::{BiOp, BiOperator, Operation},
        },
    },
    solver::{self, SolverSettings},
};

mod operation;

/// Parses a SMT-LIB-2 file.
///
/// Typically, the file will consist of constant and function declarations
/// and assertions followed by the check-sat command. The SAT solver will be
/// called then.
///
/// Returns a three-valued result of a single check-sat call or none
/// if there was not exactly one check-sat call.
pub fn parse(
    reader: impl std::io::BufRead,
    path: PathBuf,
    settings: SolverSettings,
) -> RooleResult {
    // construct the parser
    let mut parser = Parser::new(settings);

    let stream = CommandStream::new(
        reader,
        SyntaxBuilder,
        Some(path.to_string_lossy().to_string()),
    );
    for command_result in stream {
        match command_result {
            Ok(command) => {
                // parse the command normally
                if parser.parse_command(command).is_break() {
                    break;
                }
            }
            Err(err) => {
                panic!("Cannot parse SMT-LIB2 command: {:?}", err);
            }
        }
    }

    if parser.results.len() == 1 {
        parser.results[0]
    } else {
        RooleResult::None
    }
}

#[derive(Debug)]
struct Func {
    param_names: Vec<String>,
    term: Term,
}

/// Parser structure.
///
/// Currently, only simple parsing without pushing/popping
/// of assertion scopes is implemented.
#[derive(Debug)]
struct Parser {
    /// Defined functions.
    functions: IndexMap<String, Func>,
    /// Stack of binding scopes.
    scopes: Vec<Scope>,
    /// Bitvector variables.
    variables: Vec<Variable>,
    /// Operations on variables and other operation results.
    operations: Vec<Operation>,
    /// List of assertions.
    assertions: Vec<FormulaId>,
    /// Results of check-sat calls.
    results: Vec<RooleResult>,

    /// Solver settings.
    settings: SolverSettings,

    /// Expected result of solving.
    expected_result: Option<bool>,
}

// Binding scope.
//
// This binds variable names to formula ids.
#[derive(Debug)]
struct Scope {
    names: IndexMap<String, FormulaId>,
}

impl Scope {
    fn new() -> Scope {
        Scope {
            names: IndexMap::new(),
        }
    }
}

enum ReversePolishElement {
    Term(Term),
    Application(QualIdentifier, usize),
    LetStart(Vec<Symbol>),
    LetEnd,
}

impl Parser {
    fn new(settings: SolverSettings) -> Self {
        Self {
            functions: IndexMap::new(),
            scopes: vec![Scope::new()],
            variables: Vec::new(),
            operations: Vec::new(),
            assertions: Vec::new(),
            results: Vec::new(),
            settings,
            expected_result: None,
        }
    }

    pub fn parse_command(&mut self, command: Command) -> ControlFlow<(), ()> {
        match command {
            Command::CheckSat => {
                self.check_sat();
            }
            Command::Assert { term } => {
                // parse the assertion formula and add it to assertions
                let formula_id = self.create_formula(term);
                self.assertions.push(formula_id);
            }
            Command::DeclareConst { symbol, sort } => {
                self.declare_fun(symbol, Vec::new(), sort);
            }
            Command::DeclareFun {
                symbol,
                parameters,
                sort,
            } => {
                self.declare_fun(symbol, parameters, sort);
            }
            Command::DefineFun { sig, term } => self.define_fun(sig, term),
            Command::Exit => {
                return ControlFlow::Break(());
            }
            Command::SetInfo { keyword, value } => {
                // TODO: only check info based on a command-line argument
                if keyword.0 == "status" {
                    let AttributeValue::Symbol(symbol) = value else {
                        panic!("Expected status value to be a symbol");
                    };
                    self.expected_result = match symbol.0.as_str() {
                        "sat" => Some(true),
                        "unsat" => Some(false),
                        "unknown" => None,
                        _ => panic!("Expected status value to be sat, unsat, or unknown"),
                    };
                }
            }
            Command::SetLogic { symbol } => {
                if symbol.0 != "QF_BV" {
                    panic!("Logic '{:?}' not supported, only QF_BV supported", symbol.0);
                }
            }
            _ => {
                panic!("Command not supported: {:?}", command);
            }
        }

        ControlFlow::Continue(())
    }

    fn check_sat(&mut self) {
        // construct one result assertion by doing bit-ands
        // of all assertions
        let mut assertion = None;
        for partial_assertion in &self.assertions {
            assertion = match assertion {
                Some(result_assertion) => {
                    self.operations.push(Operation::BiOp(BiOp {
                        op: BiOperator::BitAnd,
                        input_width: 1,
                        left: result_assertion,
                        right: *partial_assertion,
                    }));

                    Some(OperationId(self.operations.len() - 1).formula_id())
                }
                None => Some(*partial_assertion),
            }
        }

        let assertion = assertion.unwrap_or_else(|| {
            // if there are no assertions, the assertion is a trivial tautology
            let value = ConcreteBitvector::from_bool(true, RBound::single_bit_bound());
            self.operations.push(Operation::Constant(value));
            OperationId(self.operations.len() - 1).formula_id()
        });

        // call the solver
        let problem = Problem::new(self.variables.clone(), self.operations.clone(), assertion);

        let result = solver::solve(&problem, &self.settings);

        self.results.push(self.solver_result(result));
    }

    fn solver_result(&self, result: ThreeValued) -> RooleResult {
        let Some(result) = result.to_opt_bool() else {
            return RooleResult::Unknown;
        };
        if let Some(expected_result) = self.expected_result
            && expected_result != result
        {
            // definitely wrong result
            return RooleResult::Wrong(result);
        }
        RooleResult::Known(result)
    }

    fn create_formula(&mut self, term: Term) -> FormulaId {
        // To convert terms to formulas without recursion, we use an operation stack
        // in a Polish notation with reversed order of arguments and a value stack.
        // We evaluate the operation stack from the back to the front.
        // Note that the operation stack has arguments in right-to-left order,
        // while the value stack has results in left-to-right order.

        let mut op_deque = Vec::new();
        let mut value_stack = Vec::new();

        op_deque.push(ReversePolishElement::Term(term));

        while let Some(polish_element) = op_deque.pop() {
            let formula_id = match polish_element {
                ReversePolishElement::Term(term) => self.create_formula_inner(term, &mut op_deque),
                ReversePolishElement::Application(qual_identifier, num_arguments) => {
                    // take the arguments from the back of evaluated values, no need to reverse them
                    assert!(num_arguments <= value_stack.len());
                    let arguments = value_stack.split_off(value_stack.len() - num_arguments);
                    assert_eq!(num_arguments, arguments.len());
                    Some(self.create_formula_from_application(qual_identifier, arguments))
                }
                ReversePolishElement::LetStart(names) => {
                    // take the binding values from the back of evaluated values, no need to reverse them
                    assert!(names.len() <= value_stack.len());
                    let values = value_stack.split_off(value_stack.len() - names.len());
                    assert_eq!(names.len(), values.len());

                    // push a new scope with the variable bindings
                    let mut new_scope = Scope::new();
                    for (name, value) in names.into_iter().zip(values) {
                        new_scope.names.insert(name.0, value);
                    }
                    self.scopes.push(new_scope);

                    // the next elements will be resolved
                    None
                }
                ReversePolishElement::LetEnd => {
                    // just pop the scope
                    self.scopes.pop();

                    None
                }
            };

            // Evaluated terms place the resulting formula id at the end of the value stack.

            if let Some(formula_id) = formula_id {
                value_stack.push(formula_id);
            }
        }

        let Ok(formula_id) = value_stack.into_iter().exactly_one() else {
            panic!("Expected exactly one formula id on the value stack");
        };

        formula_id
    }

    fn create_formula_inner(
        &mut self,
        term: Term,
        op_stack: &mut Vec<ReversePolishElement>,
    ) -> Option<FormulaId> {
        let result = match term {
            Term::Constant(constant) => match constant {
                Constant::Binary(mut bits) => {
                    let mut big = BigUint::zero();

                    // make little-endian
                    bits.reverse();

                    for (bit_index, bit) in bits.iter().enumerate().rev() {
                        big.set_bit(bit_index.try_into().unwrap(), *bit);
                    }

                    let width = bits
                        .len()
                        .try_into()
                        .expect("Binary constant width too big");

                    let value = ConcreteBitvector::from_big(big, RBound::new(width));
                    self.add_operation(Operation::Constant(value))
                }
                Constant::Hexadecimal(nibbles) => {
                    // construct big uint from big-endian nibbles
                    let mut big = BigUint::zero();

                    let width = (nibbles.len() * 4)
                        .try_into()
                        .expect("Hexadecimal constant width too big");

                    for nibble in nibbles {
                        assert!(nibble < 16);
                        big = (big << 4) + nibble;
                    }

                    let value = ConcreteBitvector::from_big(big, RBound::new(width));
                    self.add_operation(Operation::Constant(value))
                }
                Constant::Numeral(_) | Constant::Decimal(_) => {
                    panic!("Numeral or decimal constants not supported in QF_BV");
                }
                Constant::String(_) => panic!("String literal {:?} not supported", constant),
            },
            Term::QualIdentifier(qual_ident) => match qual_ident {
                QualIdentifier::Simple { identifier } => {
                    self.create_formula_from_identifier(identifier)
                }
                QualIdentifier::Sorted { identifier, sort } => {
                    panic!(
                        "Qualified identifier {:?} with sort {:?} not supported",
                        identifier, sort
                    );
                }
            },
            Term::Application {
                qual_identifier,
                arguments,
            } => {
                // Application terms first push the application element to operation stack,
                // followed by the argument terms in reverse order.
                op_stack.push(ReversePolishElement::Application(
                    qual_identifier,
                    arguments.len(),
                ));
                op_stack.extend(arguments.into_iter().rev().map(ReversePolishElement::Term));
                return None;
            }
            Term::Let { var_bindings, term } => {
                let mut var_symbols = Vec::new();
                let mut var_terms = Vec::new();
                for (var_symbol, var_term) in var_bindings {
                    var_symbols.push(var_symbol);
                    var_terms.push(var_term);
                }

                // Let terms first push a let-and element that pops the scope,
                // followed by the term inside let that will be evaluated in the scope,
                // followed by a let-start element that adds a new scope with bindings from the value stack,
                // followed by binding terms in reverse order.

                op_stack.push(ReversePolishElement::LetEnd);
                op_stack.push(ReversePolishElement::Term(*term));
                op_stack.push(ReversePolishElement::LetStart(var_symbols));

                op_stack.extend(var_terms.into_iter().rev().map(ReversePolishElement::Term));

                return None;
            }
            Term::Forall { .. } | Term::Exists { .. } => {
                panic!("Quantifiers not supported")
            }
            Term::Match { .. } => panic!("Match not supported"),
            Term::Attributes { .. } => panic!("Attributes not supported"),
        };

        Some(result)
    }

    fn create_formula_from_identifier(&mut self, identifier: Identifier) -> FormulaId {
        match identifier {
            Identifier::Simple { symbol } => {
                // this is just a symbol name that should be defined within the scope
                // find the formula id for it
                let name = symbol.0;
                if let Some(ident) = self.find_by_name(&name) {
                    ident
                } else {
                    let single_bit_bound = RBound::single_bit_bound();
                    match name.as_str() {
                        "false" => self.add_operation(Operation::Constant(
                            ConcreteBitvector::from_bool(false, single_bit_bound),
                        )),
                        "true" => self.add_operation(Operation::Constant(
                            ConcreteBitvector::from_bool(true, single_bit_bound),
                        )),
                        _ => {
                            // can be a call of a function with no parameters
                            let Some(func) = self.functions.get(&name) else {
                                panic!("Identifier {:?} should be in variables or functions", name);
                            };

                            assert_eq!(func.param_names.len(), 0);

                            // replace with the term
                            self.create_formula(func.term.clone())
                        }
                    }
                }
            }
            Identifier::Indexed { symbol, indices } => {
                // indexed identifiers are currently only supported
                // for defining bit-vector constants
                let Some(value) = symbol.0.strip_prefix("bv") else {
                    panic!(
                        "Identifier {:?} with indices {:?} not supported",
                        symbol, indices
                    )
                };

                assert_eq!(indices.len(), 1);

                let Some(Index::Numeral(width)) = indices.into_iter().next() else {
                    panic!("Unexpected non-numeral index in bit-vector constant")
                };

                let Ok(width) = width.try_into() else {
                    panic!("Bitvector width too big");
                };

                let Ok(value) = BigUint::from_str(value) else {
                    panic!(
                        "Bitvector value of identifier {:?} could not be parsed",
                        symbol.0
                    );
                };

                // TODO: parse as big

                let value = ConcreteBitvector::from_big(value, RBound::new(width));

                // save the constant as an operation
                self.add_operation(Operation::Constant(value))
            }
        }
    }

    fn declare_fun(&mut self, fn_symbol: Symbol, parameters: Vec<Sort>, sort: Sort) {
        // only bitvector variable declarations are currently supported here

        if !parameters.is_empty() {
            panic!("Function with params not supported");
        }

        let width = sort_width(sort);

        // this declares a bitvector variable with a given width and name
        // add the variable with given width into our variables

        let variable_id = VariableId(self.variables.len());
        self.variables.push(Variable { width });

        // add the variable name mapping to the current scope
        if self
            .current_scope_mut()
            .names
            .insert(fn_symbol.0.clone(), FormulaId::Variable(variable_id))
            .is_some()
        {
            panic!("Multiple variables with same name '{}'", fn_symbol.0);
        }
    }

    fn define_fun(&mut self, sig: FunctionDec, term: Term) {
        let fn_name = sig.name.0;

        let mut param_names = Vec::new();

        for (symbol, sort) in sig.parameters {
            let param_name = symbol.0;
            let _param_width = sort_width(sort);
            param_names.push(param_name);
        }

        let _result_width = sort_width(sig.result);
        self.functions.insert(fn_name, Func { param_names, term });
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes
            .first_mut()
            .expect("A scope should be available")
    }

    fn find_by_name(&self, name: &str) -> Option<FormulaId> {
        // search the scopes in reverse order
        for scope in self.scopes.iter().rev() {
            if let Some(formula_id) = scope.names.get(name) {
                return Some(*formula_id);
            }
        }

        None
    }

    fn add_operation(&mut self, operation: Operation) -> FormulaId {
        self.operations.push(operation);
        FormulaId::Operation(OperationId(self.operations.len() - 1))
    }
}

fn sort_width(sort: Sort) -> u32 {
    match sort {
        Sort::Simple {
            identifier:
                Identifier::Indexed {
                    symbol: bitvec_symbol,
                    indices,
                },
        } => {
            if bitvec_symbol.0 != "BitVec" {
                panic!("Only bitvector indexed sort supported");
            }

            if indices.len() != 1 {
                panic!("Bitvector sort should have exactly one index");
            }

            let Index::Numeral(length) = &indices[0] else {
                panic!("Bitvector width should be numeric");
            };

            let Ok(width) = std::convert::TryInto::<u32>::try_into(length) else {
                panic!("Bitvector width should fit into u32");
            };
            width
        }
        Sort::Simple {
            identifier: Identifier::Simple { symbol },
        } => {
            if symbol.0 != "Bool" {
                panic!("Only bool non-indexed sort supported");
            }
            1
        }
        _ => panic!("Only bool and bitvector sort supported"),
    }
}
