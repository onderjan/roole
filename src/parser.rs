use std::{ops::ControlFlow, path::PathBuf};

use aws_smt_ir::{
    Constant, Symbol, SyntaxBuilder,
    smt2parser::{
        CommandStream,
        concrete::{Command, Identifier, QualIdentifier, Sort, Term},
        visitors::{AttributeValue, Index},
    },
};
use indexmap::IndexMap;

use crate::{
    domain::value::ThreeValued,
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
) -> Option<ThreeValued> {
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
        Some(parser.results[0])
    } else {
        None
    }
}

/// Parser structure.
///
/// Currently, only simple parsing without pushing/popping
/// of assertion scopes is implemented.
#[derive(Debug)]
struct Parser {
    /// Stack of binding scopes.
    scopes: Vec<Scope>,
    /// Bitvector variables.
    variables: Vec<Variable>,
    /// Operations on variables and other operation results.
    operations: Vec<Operation>,
    /// List of assertions.
    assertions: Vec<FormulaId>,
    /// Results of check-sat calls.
    results: Vec<ThreeValued>,

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

impl Parser {
    fn new(settings: SolverSettings) -> Self {
        Self {
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
        let mut result_assertion = None;
        for assertion in &self.assertions {
            result_assertion = match result_assertion {
                Some(result_assertion) => {
                    self.operations.push(Operation::BiOp(BiOp {
                        op: BiOperator::BitAnd,
                        input_width: 1,
                        left: result_assertion,
                        right: *assertion,
                    }));

                    Some(FormulaId::Operation(OperationId(self.operations.len() - 1)))
                }
                None => Some(*assertion),
            }
        }

        let Some(assertion) = result_assertion else {
            eprintln!("Checking satisfiability with no assertions is a no-op");
            return;
        };

        // call the solver
        let problem = Problem::new(self.variables.clone(), self.operations.clone(), assertion);

        let result = solver::solve(&problem, &self.settings);

        if result.is_known()
            && let Some(expected_result) = self.expected_result
        {
            let our_result = result.is_true();
            if expected_result != our_result {
                eprintln!(
                    "Wrong solver result, expected {}, but got {}",
                    expected_result, our_result
                );
                // immediately exit with a unique status code
                // TODO: make this less hacky by propagating the wrong result
                std::process::exit(64)
            }
        }

        self.results.push(result);
    }

    fn create_formula(&mut self, term: Term) -> FormulaId {
        match term {
            Term::Constant(constant) => match constant {
                Constant::Binary(items) => {
                    let mut value = 0u64;
                    for bit in items.iter().cloned() {
                        value = value.checked_mul(2).expect("Binary constant too big");
                        value += bit as u64;
                    }

                    self.add_operation(Operation::Constant(
                        value,
                        items
                            .len()
                            .try_into()
                            .expect("Binary constant width too big"),
                    ))
                }
                Constant::Numeral(_) | Constant::Decimal(_) | Constant::Hexadecimal(_) => {
                    todo!("Create formula for constant {:?}", constant)
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
            } => self.create_formula_from_application(qual_identifier, arguments),
            Term::Let { var_bindings, term } => {
                // push a new scope
                self.scopes.push(Scope::new());

                // add variable bindings
                for (symbol, term) in var_bindings {
                    let name = symbol.0;
                    let formula_id = self.create_formula(term);

                    self.current_scope_mut().names.insert(name, formula_id);
                }

                // evaluate subterm
                let result = self.create_formula(*term);

                // pop the scope
                self.scopes.pop();

                result
            }
            Term::Forall { .. } | Term::Exists { .. } => {
                panic!("Quantifiers not supported")
            }
            Term::Match { .. } => panic!("Match not supported"),
            Term::Attributes { .. } => panic!("Attributes not supported"),
        }
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
                    match name.as_str() {
                        "false" => self.add_operation(Operation::Constant(0, 1)),
                        "true" => self.add_operation(Operation::Constant(1, 1)),
                        _ => panic!("Identifier {:?} should be in variables", name),
                    }
                }
            }
            Identifier::Indexed { symbol, indices } => {
                // indexed identifiers are currently only supported
                // for defining bit-vector constants
                let Some(bitvector_width) = symbol.0.strip_prefix("bv") else {
                    panic!(
                        "Identifier {:?} with indices {:?} not supported",
                        symbol, indices
                    )
                };
                let Ok(value) = bitvector_width.parse() else {
                    panic!(
                        "Bitvector width of identifier {:?} could not be parsed",
                        symbol.0
                    );
                };

                assert_eq!(indices.len(), 1);

                let Some(Index::Numeral(width)) = indices.into_iter().next() else {
                    panic!("Unexpected non-numeral index in bit-vector constant")
                };

                let Ok(width) = width.try_into() else {
                    panic!("Bitvector width too big");
                };

                // save the constant as an operation
                self.add_operation(Operation::Constant(value, width))
            }
        }
    }

    fn declare_fun(&mut self, fn_symbol: Symbol, parameters: Vec<Sort>, sort: Sort) {
        // only bitvector variable declarations are currently supported here

        if !parameters.is_empty() {
            panic!("Function with params not supported");
        }

        let width = match sort {
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
        };

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
