use crate::{
    domain::{bitvector::abstr::BitvectorDomain, traits::forward::HwArith},
    problem::{domain::OperationDomain, linear::LinearCombination},
};

impl HwArith for OperationDomain {
    fn arith_neg(self) -> Self {
        let combination = match self.try_combination() {
            Ok(ok) => ok,
            Err(err) => return err,
        };

        Self::from_combination(combination.arith_neg())
    }
    fn add(self, rhs: Self) -> Self {
        self.linear_combine(rhs, |a, b| a.add(b))
    }

    fn sub(self, rhs: Self) -> Self {
        self.linear_combine(rhs, |a, b| a.sub(b))
    }

    fn mul(self, rhs: Self) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        let (Ok(lhs), Ok(rhs)) = (self.try_combination(), rhs.try_combination()) else {
            // return top value
            return Self::top(bound);
        };

        let (constant, mut combination) = if lhs.monomials.is_empty() {
            (lhs.constant, rhs)
        } else if rhs.monomials.is_empty() {
            (rhs.constant, lhs)
        } else {
            // return top value
            return Self::top(bound);
        };

        // multiply combination by constant
        combination.scale(constant);
        Self::from_combination(combination)
    }

    fn udiv(self, _rhs: Self) -> Self {
        todo!("udiv")
    }

    fn sdiv(self, _rhs: Self) -> Self {
        todo!("sdiv")
    }

    fn urem(self, _rhs: Self) -> Self {
        todo!("urem")
    }

    fn srem(self, _rhs: Self) -> Self {
        todo!("srem")
    }
}

impl OperationDomain {
    fn linear_combine(
        self,
        rhs: OperationDomain,
        op: fn(LinearCombination, LinearCombination) -> LinearCombination,
    ) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        let (Ok(lhs), Ok(rhs)) = (self.try_combination(), rhs.try_combination()) else {
            return OperationDomain::top(bound);
        };

        let combination = op(lhs, rhs);
        OperationDomain::from_combination(combination)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        domain::bitvector::{RBound, concr::ConcreteBitvector},
        problem::formula::{FormulaId, VariableId},
    };

    use super::*;

    #[test]
    fn test_addsub() {
        let bound = RBound::new(32);
        let a = OperationDomain::from_combination(LinearCombination {
            constant: ConcreteBitvector::new(38, bound),
            monomials: BTreeMap::from_iter([(
                FormulaId::Variable(VariableId(0)),
                ConcreteBitvector::new(12, bound),
            )]),
        });
        let b = OperationDomain::from_combination(LinearCombination {
            constant: ConcreteBitvector::new(17, bound),
            monomials: BTreeMap::from_iter([(
                FormulaId::Variable(VariableId(0)),
                ConcreteBitvector::new(7, bound),
            )]),
        });
        let add_result = OperationDomain::from_combination(LinearCombination {
            constant: ConcreteBitvector::new(55, bound),
            monomials: BTreeMap::from_iter([(
                FormulaId::Variable(VariableId(0)),
                ConcreteBitvector::new(19, bound),
            )]),
        });
        let sub_result = OperationDomain::from_combination(LinearCombination {
            constant: ConcreteBitvector::new(21, bound),
            monomials: BTreeMap::from_iter([(
                FormulaId::Variable(VariableId(0)),
                ConcreteBitvector::new(5, bound),
            )]),
        });
        assert_eq!(a.clone().add(b.clone()), add_result);
        assert_eq!(a.sub(b), sub_result);
    }
}
