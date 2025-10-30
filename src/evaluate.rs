use std::ops::ControlFlow;

use aws_smt_ir::smt2parser::visitors::Index;
use aws_smt_ir::{CommandStream, smt2parser::concrete};
use indexmap::IndexMap;

use crate::check;
use crate::formula::{BiOp, FormulaId, Operation, OperationId, UniOp, VariableId};

#[derive(Debug)]
struct Evaluator {
    names: IndexMap<String, FormulaId>,
    variables: Vec<u32>,
    operations: Vec<Operation>,
    assertions: Vec<FormulaId>,
}

pub fn evaluate(reader: impl std::io::BufRead, path: Option<String>) {
    let stream = CommandStream::new(reader, concrete::SyntaxBuilder, path);
    let commands = stream
        .collect::<Result<Vec<_>, _>>()
        .expect("File should be SMT-LIB-2 parseable");

    let mut evaluator = Evaluator {
        names: IndexMap::new(),
        variables: Vec::new(),
        operations: Vec::new(),
        assertions: Vec::new(),
    };

    for command in commands {
        if evaluator.evaluate(command).is_break() {
            break;
        }
    }
}

impl Evaluator {
    pub fn evaluate(&mut self, command: concrete::Command) -> ControlFlow<(), ()> {
        //println!("{:#?}", command);
        match command {
            concrete::Command::Assert { term } => {
                let formula_id = self.create_formula(term);

                self.assertions.push(formula_id);
            }
            concrete::Command::CheckSat => {
                self.check_sat();
            }
            concrete::Command::DeclareConst { symbol, sort } => {
                self.declare_fun(symbol, Vec::new(), sort);
            }
            concrete::Command::DeclareFun {
                symbol,
                parameters,
                sort,
            } => {
                self.declare_fun(symbol, parameters, sort);
            }
            concrete::Command::Exit => {
                return ControlFlow::Break(());
            }
            concrete::Command::SetInfo { .. } => {
                // ignore
            }
            concrete::Command::SetLogic { symbol } => {
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

    fn declare_fun(
        &mut self,
        fn_symbol: concrete::Symbol,
        parameters: Vec<concrete::Sort>,
        sort: concrete::Sort,
    ) {
        if !parameters.is_empty() {
            panic!("Function with params not supported");
        }

        let concrete::Sort::Simple {
            identifier:
                concrete::Identifier::Indexed {
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

        let variable_id = VariableId(self.variables.len());
        self.variables.push(width);

        if self
            .names
            .insert(fn_symbol.0.clone(), FormulaId::Variable(variable_id))
            .is_some()
        {
            panic!("Multiple variables with same name '{}'", fn_symbol.0);
        }
    }

    fn create_formula(&mut self, term: concrete::Term) -> FormulaId {
        match term {
            concrete::Term::Constant(constant) => {
                todo!("Create formula for constant {:?}", constant);
            }
            concrete::Term::QualIdentifier(qual_ident) => {
                let name = extract_qualified_identifier(qual_ident);
                let formula_id = self
                    .names
                    .get(&name)
                    .expect("Qualified identifier should be in variables");

                *formula_id
            }
            concrete::Term::Application {
                qual_identifier: qual_ident,
                arguments: term_arguments,
            } => {
                let mut arguments = Vec::new();
                for argument in term_arguments {
                    arguments.push(self.create_formula(argument));
                }

                let application_name = extract_qualified_identifier(qual_ident);
                let formula = match application_name.as_str() {
                    "not" => self.create_uni_op(UniOp::Not, arguments),
                    "=" => self.create_bi_op(BiOp::Eq, arguments),
                    "bvadd" => self.create_bi_op(BiOp::Add, arguments),
                    "bvsub" => self.create_bi_op(BiOp::Sub, arguments),
                    "and" => self.create_bi_op(BiOp::BitAnd, arguments),
                    "or" => self.create_bi_op(BiOp::BitOr, arguments),
                    "xor" => self.create_bi_op(BiOp::BitXor, arguments),
                    _ => {
                        panic!("Unsupported application '{}'", application_name);
                    }
                };

                self.operations.push(formula);
                FormulaId::Operation(OperationId(self.operations.len() - 1))
            }
            concrete::Term::Let { var_bindings, term } => {
                // TODO: scopes
                todo!("Let")
                /*for (symbol, term) in var_bindings {
                    let name = symbol.0;
                    let formula_id = self.create_formula(term);

                    self.names.insert(name, formula_id);
                }
                // TODO add variable bindings
                let result = self.create_formula(*term);
                result*/
            }
            concrete::Term::Forall { vars, term } => panic!("Quantifiers not supported"),
            concrete::Term::Exists { vars, term } => panic!("Quantifiers not supported"),
            concrete::Term::Match { term, cases } => panic!("Match not supported"),
            concrete::Term::Attributes { term, attributes } => panic!("Attributes not supported"),
        }
    }

    fn check_sat(&mut self) {
        let mut result_assertion = None;
        for assertion in &self.assertions {
            result_assertion = match result_assertion {
                Some(result_assertion) => {
                    self.operations.push(Operation::BiOp(
                        BiOp::BitAnd,
                        result_assertion,
                        *assertion,
                    ));

                    Some(FormulaId::Operation(OperationId(self.operations.len() - 1)))
                }
                None => Some(*assertion),
            }
        }

        let Some(assertion) = result_assertion else {
            println!("Checking satisfiability with no assertions is a no-op");
            return;
        };

        check::Checker {
            variable_widths: self.variables.clone(),
            operations: self.operations.clone(),
            assertion,
        }
        .check();
    }

    fn create_uni_op(&mut self, op: UniOp, arguments: Vec<FormulaId>) -> Operation {
        let mut iter = arguments.into_iter();
        let inner = iter
            .next()
            .expect("Binary operation should have first argument");

        if iter.next().is_some() {
            panic!("Binary operation should not have more than one argument");
        }

        Operation::UniOp(op, inner)
    }

    fn create_bi_op(&mut self, op: BiOp, arguments: Vec<FormulaId>) -> Operation {
        let mut iter = arguments.into_iter();
        let left = iter
            .next()
            .expect("Binary operation should have first argument");
        let right = iter
            .next()
            .expect("Binary operation should have second argument");

        if iter.next().is_some() {
            panic!("Binary operation should not have more than two arguments");
        }

        Operation::BiOp(op, left, right)
    }
}

fn extract_qualified_identifier(qualified_ident: concrete::QualIdentifier) -> String {
    let concrete::QualIdentifier::Simple {
        identifier: concrete::Identifier::Simple { symbol },
    } = qualified_ident
    else {
        panic!(
            "Only simple qualified identifier supported, not {:?}",
            qualified_ident
        );
    };

    symbol.0
}
