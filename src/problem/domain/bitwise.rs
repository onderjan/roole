use crate::{
    domain::{
        bitvector::{BitvectorBound, abstr::BitvectorDomain},
        traits::forward::Bitwise,
    },
    problem::{
        domain::OperationDomain,
        operation::{LinearCombination, LinearSystem},
    },
};

impl Bitwise for OperationDomain {
    fn bit_not(self) -> Self {
        let linear = match self {
            OperationDomain::Top(_) => return self,
            OperationDomain::Linear(linear) => linear,
        };

        OperationDomain::Linear(linear.bit_not())
    }

    fn bit_and(self, rhs: Self) -> Self {
        eprintln!(
            "Bitwise AND (width {:?}): {:?}, {:?}",
            self.bound().width(),
            self,
            rhs
        );

        self.bit_linear(rhs, true)
    }
    fn bit_or(self, rhs: Self) -> Self {
        eprintln!(
            "Bitwise OR (width {:?}): {:?}, {:?}",
            self.bound().width(),
            self,
            rhs
        );

        self.bit_linear(rhs, false)
    }
    fn bit_xor(self, rhs: Self) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        // TODO: handle masking situations

        OperationDomain::top(bound)
    }
}

impl OperationDomain {
    fn bit_linear(self, rhs: Self, conjunction: bool) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        if bound.width() == 1 {
            // resolve constants

            let mut constant = self.constant_value().map(|lhs| (lhs.is_nonzero(), false));
            if constant.is_none() {
                constant = rhs.constant_value().map(|rhs| (rhs.is_nonzero(), true));
            }

            if let Some((constant, is_constant_right)) = constant {
                // for conjunction, return the other if constant is 1
                // for disjunction, return the other if constant is 0
                if constant == conjunction {
                    return if is_constant_right { self } else { rhs };
                } else {
                    // for conjunction, return 0 as the constant is 0
                    // for disjunction, return 1 as the constant is 1
                    return Self::from_combination(LinearCombination::single_bit(constant));
                }
            }
        }

        let (Ok(lhs), Ok(rhs)) = (self.try_system(), rhs.try_system()) else {
            return Self::top(bound);
        };

        let op = if conjunction {
            |a: LinearSystem, b: LinearSystem| a.and(b)
        } else {
            |a: LinearSystem, b: LinearSystem| a.or(b)
        };

        let Some(system) = (op)(lhs, rhs) else {
            return Self::top(bound);
        };

        OperationDomain::from_system(system)
    }
}
