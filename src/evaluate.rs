use std::ops::ControlFlow;

use aws_smt_ir::smt2parser::concrete::QualIdentifier;
use aws_smt_ir::smt2parser::visitors::Index;
use aws_smt_ir::{CommandStream, smt2parser::concrete};
use indexmap::IndexMap;

use crate::check;
use crate::formula::{
    BiOp, BiOperator, FormulaId, Operation, OperationId, UniOp, UniOperator, VariableId,
};

#[derive(Debug)]
struct Evaluator {
    scopes: Vec<Scope>,
    variables: Vec<u32>,
    operations: Vec<Operation>,
    assertions: Vec<FormulaId>,
}

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

pub fn evaluate(reader: impl std::io::BufRead, path: Option<String>) {
    let stream = CommandStream::new(reader, concrete::SyntaxBuilder, path);
    let commands = stream
        .collect::<Result<Vec<_>, _>>()
        .expect("File should be SMT-LIB-2 parseable");

    let mut evaluator = Evaluator {
        scopes: vec![Scope::new()],
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
            .current_scope_mut()
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
            concrete::Term::QualIdentifier(qual_ident) => match qual_ident {
                QualIdentifier::Simple { identifier } => match identifier {
                    concrete::Identifier::Simple { symbol } => self.find_name(&symbol.0),
                    concrete::Identifier::Indexed { symbol, indices } => {
                        if let Some(bitvector_width) = symbol.0.strip_prefix("bv")
                            && let Ok(width) = bitvector_width.parse()
                        {
                            assert_eq!(indices.len(), 1);

                            let Some(Index::Numeral(constant)) = indices.into_iter().next() else {
                                panic!("Unexpected non-numeral index in bit-vector constant")
                            };

                            let Ok(constant) = constant.try_into() else {
                                panic!("Bitvector constant too big for u64");
                            };

                            let formula = Operation::Constant(constant, width);
                            self.operations.push(formula);
                            FormulaId::Operation(OperationId(self.operations.len() - 1))
                        } else {
                            panic!(
                                "Qualified identifier {:?} with indices {:?} not supported",
                                symbol, indices
                            )
                        }
                    }
                },
                QualIdentifier::Sorted { identifier, sort } => {
                    panic!(
                        "Qualified identifier {:?} with sort {:?} not supported",
                        identifier, sort
                    );
                }
            },
            concrete::Term::Application {
                qual_identifier: qual_ident,
                arguments: term_arguments,
            } => {
                let mut arguments = Vec::new();
                for argument in term_arguments {
                    arguments.push(self.create_formula(argument));
                }

                let operation = if let Some(application_name) = qual_name(qual_ident.clone()) {
                    match application_name.as_str() {
                        "not" => self.create_uni_op(UniOperator::Not, arguments),
                        "=" => self.create_bi_op(BiOperator::Eq, arguments),
                        "bvadd" => self.create_bi_op(BiOperator::Add, arguments),
                        "bvsub" => self.create_bi_op(BiOperator::Sub, arguments),
                        "and" => self.create_bi_op(BiOperator::BitAnd, arguments),
                        "or" => self.create_bi_op(BiOperator::BitOr, arguments),
                        "xor" => self.create_bi_op(BiOperator::BitXor, arguments),
                        _ => {
                            panic!("Unsupported application '{}'", application_name);
                        }
                    }
                } else {
                    panic!(
                        "Only simple qualified identifier supported for application, not {:?}",
                        qual_ident
                    );
                };

                self.operations.push(operation);
                FormulaId::Operation(OperationId(self.operations.len() - 1))
            }
            concrete::Term::Let { var_bindings, term } => {
                // push a new scope with bindings
                self.scopes.push(Scope::new());

                for (symbol, term) in var_bindings {
                    let name = symbol.0;
                    let formula_id = self.create_formula(term);

                    self.current_scope_mut().names.insert(name, formula_id);
                }
                // TODO add variable bindings
                let result = self.create_formula(*term);

                // pop the scope
                self.scopes.pop();

                result
            }
            concrete::Term::Forall { .. } | concrete::Term::Exists { .. } => {
                panic!("Quantifiers not supported")
            }
            concrete::Term::Match { .. } => panic!("Match not supported"),
            concrete::Term::Attributes { .. } => panic!("Attributes not supported"),
        }
    }

    fn check_sat(&mut self) {
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

    fn create_uni_op(&mut self, op: UniOperator, arguments: Vec<FormulaId>) -> Operation {
        let mut iter = arguments.into_iter();
        let inner = iter
            .next()
            .expect("Binary operation should have first argument");

        if iter.next().is_some() {
            panic!("Binary operation should not have more than one argument");
        }

        let input_width = self.formula_result_width(inner);

        Operation::UniOp(UniOp {
            op,
            input_width,
            inner,
        })
    }

    fn create_bi_op(&mut self, op: BiOperator, arguments: Vec<FormulaId>) -> Operation {
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

        let left_result_width = self.formula_result_width(left);
        let right_result_width = self.formula_result_width(right);

        assert_eq!(left_result_width, right_result_width);

        Operation::BiOp(BiOp {
            op,
            input_width: left_result_width,
            left,
            right,
        })
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes
            .first_mut()
            .expect("A scope should be available")
    }

    fn find_name(&self, name: &str) -> FormulaId {
        // search the scopes in reverse order

        for scope in self.scopes.iter().rev() {
            if let Some(formula_id) = scope.names.get(name) {
                return *formula_id;
            }
        }
        panic!("Qualified identifier should be in variables");
    }

    fn formula_result_width(&self, id: FormulaId) -> u32 {
        match id {
            FormulaId::Variable(variable_id) => self.variables[variable_id.0],
            FormulaId::Operation(operation_id) => self.operations[operation_id.0].result_width(),
        }
    }
}

fn qual_name(qualified_ident: concrete::QualIdentifier) -> Option<String> {
    let concrete::QualIdentifier::Simple {
        identifier: concrete::Identifier::Simple { symbol },
    } = qualified_ident
    else {
        return None;
    };

    Some(symbol.0)
}
