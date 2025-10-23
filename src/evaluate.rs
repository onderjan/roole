use std::collections::BTreeMap;
use std::ops::ControlFlow;

use aws_smt_ir::smt2parser::visitors::Index;
use aws_smt_ir::{CommandStream, smt2parser::concrete};
use indexmap::IndexMap;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct VariableId(pub u64);

#[derive(Clone, Debug)]
enum UniOp {
    Not,
}

#[derive(Clone, Debug)]
enum BiOp {
    Add,
    Sub,

    BitAnd,
    BitOr,
    BitXor,

    Eq,
}

#[derive(Clone, Debug)]
enum Formula {
    Variable(VariableId),
    UniOp(UniOp, Box<Formula>),
    BiOp(BiOp, Box<Formula>, Box<Formula>),
}

#[derive(Debug)]
struct Evaluator {
    next_variable_index: VariableId,
    variable_indices: IndexMap<String, VariableId>,
    variable_lengths: BTreeMap<VariableId, u32>,
    assertions: Vec<Formula>,
}

pub fn evaluate(reader: impl std::io::BufRead, path: Option<String>) {
    let stream = CommandStream::new(reader, concrete::SyntaxBuilder, path);
    let commands = stream
        .collect::<Result<Vec<_>, _>>()
        .expect("File should be SMT-LIB-2 parseable");

    let mut evaluator = Evaluator {
        next_variable_index: VariableId(0),
        variable_indices: IndexMap::new(),
        variable_lengths: BTreeMap::new(),
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
                let formula = self.create_formula(term);
                self.assertions.push(formula);
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

        let Ok(length) = std::convert::TryInto::<u32>::try_into(length) else {
            panic!("Bitvector length must fit into u32");
        };

        let variable_index = self.next_variable_index;
        self.next_variable_index.0 += 1;

        if self
            .variable_indices
            .insert(fn_symbol.0.clone(), variable_index)
            .is_some()
        {
            panic!("Multiple variables with same name '{}'", fn_symbol.0);
        }
        self.variable_lengths.insert(variable_index, length);
    }

    fn create_formula(&self, term: concrete::Term) -> Formula {
        match term {
            concrete::Term::Constant(constant) => {
                todo!("Create formula for constant {:?}", constant);
            }
            concrete::Term::QualIdentifier(qual_ident) => {
                let name = extract_qualified_identifier(qual_ident);
                let var_id = self
                    .variable_indices
                    .get(&name)
                    .expect("Qualified identifier should be in variables");

                Formula::Variable(*var_id)
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
                match application_name.as_str() {
                    "not" => create_uni_op(UniOp::Not, arguments),
                    "=" => create_bi_op(BiOp::Eq, arguments),
                    "bvadd" => create_bi_op(BiOp::Add, arguments),
                    "bvsub" => create_bi_op(BiOp::Sub, arguments),
                    "and" => create_bi_op(BiOp::BitAnd, arguments),
                    "or" => create_bi_op(BiOp::BitOr, arguments),
                    "xor" => create_bi_op(BiOp::BitXor, arguments),
                    _ => {
                        panic!("Unsupported application '{}'", application_name);
                    }
                }
            }
            concrete::Term::Let { var_bindings, term } => panic!("Let not supported"),
            concrete::Term::Forall { vars, term } => panic!("Quantifiers not supported"),
            concrete::Term::Exists { vars, term } => panic!("Quantifiers not supported"),
            concrete::Term::Match { term, cases } => panic!("Match not supported"),
            concrete::Term::Attributes { term, attributes } => panic!("Attributes not supported"),
        }
    }

    pub fn check_sat(&self) {
        eprintln!("Should check-sat with {:?}", self);
    }
}

fn create_uni_op(op: UniOp, arguments: Vec<Formula>) -> Formula {
    let mut iter = arguments.into_iter();
    let inner = iter
        .next()
        .expect("Binary operation should have first argument");

    if iter.next().is_some() {
        panic!("Binary operation should not have more than one argument");
    }

    Formula::UniOp(op, Box::new(inner))
}

fn create_bi_op(op: BiOp, arguments: Vec<Formula>) -> Formula {
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

    Formula::BiOp(op, Box::new(left), Box::new(right))
}

fn extract_qualified_identifier(qualified_ident: concrete::QualIdentifier) -> String {
    let concrete::QualIdentifier::Simple {
        identifier: concrete::Identifier::Simple { symbol },
    } = qualified_ident
    else {
        panic!("Only simple qualified identifier supported");
    };

    symbol.0
}
