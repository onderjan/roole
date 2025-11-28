use std::ops::ControlFlow;

use aws_smt_ir::{
    Symbol, SyntaxBuilder,
    smt2parser::{
        CommandStream,
        concrete::{Command, Identifier, QualIdentifier, Sort, Term},
        visitors::Index,
    },
};
use indexmap::IndexMap;

use crate::{
    problem::{
        Problem,
        formula::{BiOp, BiOperator, FormulaId, Operation, OperationId, VariableId},
    },
    solver::{self},
};

mod operation;

/// Parses a SMT-LIB-2 file.
///
/// Typically, the file will consist of constant and function declarations
/// and assertions followed by the check-sat command. The SAT solver will be
/// called then.
pub fn parse(reader: impl std::io::BufRead, path: Option<String>) {
    // construct the parser
    let mut parser = Parser::new();

    let stream = CommandStream::new(reader, SyntaxBuilder, path);
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
}

/// Parser structure.
///
/// Currently, only simple parsing without pushing/popping
/// of assertion scopes is implemented.
#[derive(Debug)]
struct Parser {
    /// Stack of binding scopes.
    scopes: Vec<Scope>,
    /// Widths of defined bitvector variables.
    variables: Vec<u32>,
    /// Operations on variables and other operation results.
    operations: Vec<Operation>,
    /// List of assertions.
    assertions: Vec<FormulaId>,
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
    fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            variables: Vec::new(),
            operations: Vec::new(),
            assertions: Vec::new(),
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
            Command::SetInfo { .. } => {
                // ignore
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
        solver::solve(&problem);
    }

    fn create_formula(&mut self, term: Term) -> FormulaId {
        match term {
            Term::Constant(constant) => {
                todo!("Create formula for constant {:?}", constant);
            }
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
                self.find_by_name(&symbol.0)
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
                let formula = Operation::Constant(value, width);
                self.operations.push(formula);
                FormulaId::Operation(OperationId(self.operations.len() - 1))
            }
        }
    }

    fn declare_fun(&mut self, fn_symbol: Symbol, parameters: Vec<Sort>, sort: Sort) {
        // only bitvector variable declarations are currently supported here

        if !parameters.is_empty() {
            panic!("Function with params not supported");
        }

        let Sort::Simple {
            identifier:
                Identifier::Indexed {
                    symbol: bitvec_symbol,
                    indices,
                },
        } = sort
        else {
            panic!("Only bitvector sort supported");
        };

        if bitvec_symbol.0 != "BitVec" {
            panic!("Only bitvector sort supported");
        }

        if indices.len() != 1 {
            panic!("Only bitvector sort supported");
        }

        let Index::Numeral(length) = &indices[0] else {
            panic!("Only bitvector sort supported");
        };

        let Ok(width) = std::convert::TryInto::<u32>::try_into(length) else {
            panic!("Bitvector width must fit into u32");
        };

        // this declares a bitvector variable with a given width and name
        // add the variable with given width into our variables

        let variable_id = VariableId(self.variables.len());
        self.variables.push(width);

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

    fn find_by_name(&self, name: &str) -> FormulaId {
        // search the scopes in reverse order
        for scope in self.scopes.iter().rev() {
            if let Some(formula_id) = scope.names.get(name) {
                return *formula_id;
            }
        }
        panic!("Identifier {:?} should be in variables", name);
    }
}
