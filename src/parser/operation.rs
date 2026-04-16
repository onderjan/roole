use std::num::NonZeroU32;

use aws_smt_ir::smt2parser::{
    concrete::{Identifier, QualIdentifier},
    visitors::Index,
};
use indexmap::IndexMap;
use itertools::Itertools;

use crate::{
    parser::Scope,
    problem::formula::{
        FormulaId,
        operation::{
            BiOp, BiOperator, ConcatOp, ExtOp, ExtractOp, IteOp, Operation, RotateOp, UniOp,
            UniOperator,
        },
    },
};

impl super::Parser {
    pub(super) fn create_formula_from_application(
        &mut self,
        qual_identifier: QualIdentifier,
        arguments: Vec<FormulaId>,
    ) -> FormulaId {
        let operation = match qual_identifier {
            QualIdentifier::Simple {
                identifier: Identifier::Simple { symbol },
            } => match symbol.0.as_str() {
                "not" | "bvnot" => self.create_uni_op(UniOperator::Not, arguments),

                "=" | "bvcomp" => self.create_bi_op(BiOperator::Eq, arguments),
                "distinct" => self.create_bi_op(BiOperator::Ne, arguments),
                "=>" => self.create_bi_op(BiOperator::Implies, arguments),

                "bvult" => self.create_bi_op(BiOperator::Ult, arguments),
                "bvule" => self.create_bi_op(BiOperator::Ule, arguments),
                "bvugt" => self.create_bi_op(BiOperator::Ugt, arguments),
                "bvuge" => self.create_bi_op(BiOperator::Uge, arguments),

                "bvslt" => self.create_bi_op(BiOperator::Slt, arguments),
                "bvsle" => self.create_bi_op(BiOperator::Sle, arguments),
                "bvsgt" => self.create_bi_op(BiOperator::Sgt, arguments),
                "bvsge" => self.create_bi_op(BiOperator::Sge, arguments),

                "bvneg" => self.create_uni_op(UniOperator::Neg, arguments),
                "bvadd" => self.create_bi_op(BiOperator::Add, arguments),
                "bvsub" => self.create_bi_op(BiOperator::Sub, arguments),
                "bvmul" => self.create_bi_op(BiOperator::Mul, arguments),
                "bvudiv" => self.create_bi_op(BiOperator::Udiv, arguments),
                "bvurem" => self.create_bi_op(BiOperator::Urem, arguments),
                "bvsdiv" => self.create_bi_op(BiOperator::Sdiv, arguments),
                "bvsrem" => self.create_bi_op(BiOperator::Srem, arguments),

                "and" | "bvand" => self.create_bi_op(BiOperator::BitAnd, arguments),
                "or" | "bvor" => self.create_bi_op(BiOperator::BitOr, arguments),
                "xor" | "bvxor" => self.create_bi_op(BiOperator::BitXor, arguments),

                "bvshl" => self.create_bi_op(BiOperator::Shl, arguments),
                "bvlshr" => self.create_bi_op(BiOperator::Lshr, arguments),
                "bvashr" => self.create_bi_op(BiOperator::Ashr, arguments),

                "ite" => self.create_ite_op(arguments),
                "concat" => self.create_concat_op(arguments),
                name => {
                    // try to see if it is in the functions
                    let Some(func) = self.functions.get(name) else {
                        panic!("Unsupported application '{}'", name);
                    };

                    assert_eq!(func.param_names.len(), arguments.len());

                    // push a new scope with the parameters
                    let mut names = IndexMap::new();
                    for (param_name, arg) in func.param_names.iter().zip(arguments) {
                        names.insert(param_name.clone(), arg);
                    }

                    let scope = Scope { names };

                    self.scopes.push(scope);

                    let formula = self.create_formula(func.term.clone());

                    // pop the scope
                    self.scopes.pop();

                    return formula;
                }
            },
            QualIdentifier::Simple {
                identifier: Identifier::Indexed { symbol, indices },
            } => {
                let name = symbol.0.as_str();

                let rotate_left = name == "rotate_left";
                let rotate_right = name == "rotate_right";

                if rotate_left || rotate_right {
                    self.create_rotate_op(rotate_left, indices, arguments)
                } else {
                    match name {
                        "zero_extend" => self.create_ext_op(false, indices, arguments),
                        "sign_extend" => self.create_ext_op(true, indices, arguments),
                        "extract" => self.create_extract_op(indices, arguments),
                        _ => {
                            panic!(
                                "Unsupported qualified identifier {:?} with indices {:?}",
                                symbol, indices
                            )
                        }
                    }
                }
            }
            QualIdentifier::Sorted { identifier, sort } => {
                panic!(
                    "Qualified identifier {:?} with sort {:?} not supported within application",
                    identifier, sort
                );
            }
        };

        self.add_operation(operation)
    }

    fn create_uni_op(&mut self, op: UniOperator, arguments: Vec<FormulaId>) -> Operation {
        let Ok(inner) = arguments.into_iter().exactly_one() else {
            panic!("Unary operation should have exactly one argument");
        };

        let input_width = self.formula_result_width(inner);

        Operation::UniOp(UniOp {
            op,
            input_width,
            inner,
        })
    }

    fn create_bi_op(&mut self, op: BiOperator, mut arguments: Vec<FormulaId>) -> Operation {
        // bvand, bvor, bvxor, bvadd, bvmul are left-associative
        if matches!(
            op,
            BiOperator::BitAnd
                | BiOperator::BitOr
                | BiOperator::BitXor
                | BiOperator::Add
                | BiOperator::Mul
        ) {
            while arguments.len() > 2 {
                // replace the first two arguments with the formula

                let mut next_arguments = arguments.split_off(2);

                let first_bi_op = self.create_bi_op(op, std::mem::take(&mut arguments));
                let first_bi_op = self.add_operation(first_bi_op);
                next_arguments.insert(0, first_bi_op);
                arguments = next_arguments;
            }
        }

        let Some((left, right)) = arguments.into_iter().collect_tuple() else {
            panic!(
                "Binary operation {:?} should have exactly two arguments",
                op
            );
        };

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

    fn create_ext_op(
        &self,
        signed: bool,
        indices: Vec<Index>,
        arguments: Vec<FormulaId>,
    ) -> Operation {
        let Ok(extend_by) = indices.into_iter().exactly_one() else {
            panic!("Extension operation should have exactly one index");
        };
        let Index::Numeral(extend_by) = extend_by else {
            panic!("Only numeral extension index supported");
        };
        let Ok(extend_by) = TryInto::<u32>::try_into(extend_by) else {
            panic!("Numeral extension index should fit into u32");
        };

        let Ok(inner) = arguments.into_iter().exactly_one() else {
            panic!("Extension operation should have exactly one argument");
        };

        self.create_ext_op_inner(signed, extend_by, inner)
    }

    fn create_ext_op_inner(&self, signed: bool, extend_by: u32, inner: FormulaId) -> Operation {
        let input_width = self.formula_result_width(inner);
        let output_width = input_width + extend_by;

        Operation::ExtOp(ExtOp {
            signed,
            input_width,
            output_width,
            inner,
        })
    }

    fn create_ite_op(&mut self, arguments: Vec<FormulaId>) -> Operation {
        let Some((condition, left, right)) = arguments.into_iter().collect_tuple() else {
            panic!("If-then-else operation should have exactly three arguments");
        };

        let condition_width = self.formula_result_width(condition);
        assert_eq!(condition_width, 1);

        let left_result_width = self.formula_result_width(left);
        let right_result_width = self.formula_result_width(right);

        assert_eq!(left_result_width, right_result_width);

        Operation::IteOp(IteOp {
            condition,
            width: left_result_width,
            formula_then: left,
            formula_else: right,
        })
    }

    fn create_concat_op(&mut self, arguments: Vec<FormulaId>) -> Operation {
        let Some((left, right)) = arguments.into_iter().collect_tuple() else {
            panic!("Concat operation should have exactly two arguments");
        };

        // perform an unsigned extension of both by the other width

        let left_width = self.formula_result_width(left);
        let right_width = self.formula_result_width(right);

        Operation::ConcatOp(ConcatOp {
            left_width,
            left,
            right_width,
            right,
        })
    }

    fn create_extract_op(&self, indices: Vec<Index>, arguments: Vec<FormulaId>) -> Operation {
        let Some((msb, lsb)) = indices.into_iter().collect_tuple() else {
            panic!("Extract operation should have exactly two indices");
        };
        let (Index::Numeral(msb), Index::Numeral(lsb)) = (msb, lsb) else {
            panic!("Only numeral extract msb&lsb supported");
        };
        let Ok(msb) = TryInto::<u32>::try_into(msb) else {
            panic!("Extract msb should fit into u32");
        };
        let Ok(lsb) = TryInto::<u32>::try_into(lsb) else {
            panic!("Extract lsb should fit into u32");
        };

        let Ok(inner) = arguments.into_iter().exactly_one() else {
            panic!("Extract operation should have exactly one argument");
        };

        let width =
            NonZeroU32::new(msb - lsb + 1).expect("Extract msb should be greater or equal to lsb");

        Operation::ExtractOp(ExtractOp { inner, lsb, width })
    }

    fn create_rotate_op(
        &self,
        rotate_left: bool,
        indices: Vec<Index>,
        arguments: Vec<FormulaId>,
    ) -> Operation {
        let Ok(inner) = arguments.into_iter().exactly_one() else {
            panic!("Rotate operation should have exactly one argument");
        };
        let width = self.formula_result_width(inner);

        let Ok(Index::Numeral(amount)) = indices.into_iter().exactly_one() else {
            panic!("Rotate operation should have exactly one numeric index (rotation amount)");
        };

        let Ok(amount) = TryInto::<u32>::try_into(amount) else {
            panic!("Rotate amount is too big");
        };

        // the amount can be represented modulo width in range 0 <= amount < width
        // handle the corner case of zero width
        let amount = amount.checked_rem(width).unwrap_or(0);

        // convert right rotation to left rotation
        let left_rotate_amount = if rotate_left { amount } else { width - amount };

        Operation::RotateOp(RotateOp {
            inner,
            width,
            left_rotate_amount,
        })
    }

    fn formula_result_width(&self, id: FormulaId) -> u32 {
        match id {
            FormulaId::Variable(variable_id) => self.variables[variable_id.0].width,
            FormulaId::Operation(operation_id) => self.operations[operation_id.0].result_width(),
        }
    }
}
