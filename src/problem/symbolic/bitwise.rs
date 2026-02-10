use crate::{
    domain::{
        bitvector::{BitvectorBound, abstr::BitvectorDomain},
        traits::forward::Bitwise,
    },
    problem::symbolic::SymbolicDomain,
};

impl Bitwise for SymbolicDomain {
    fn bit_not(self) -> Self {
        self.unary_op(|a| a.bit_not())
    }

    fn bit_and(self, rhs: Self) -> Self {
        self.bit_junction(rhs, true)
    }
    fn bit_or(self, rhs: Self) -> Self {
        self.bit_junction(rhs, false)
    }
    fn bit_xor(self, rhs: Self) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        // TODO: handle XOR
        SymbolicDomain::top(bound)
    }
}

impl SymbolicDomain {
    fn bit_junction(self, rhs: Self, conjunction: bool) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        // if we work with Booleans and at least one operand is a constant,
        // we will return tautology, contradiction, or the other value
        if bound.width() == 1 {
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
                    return Self::from_bool(constant);
                }
            }
        }

        // hand over to linear
        self.binary_op_try(rhs, |a, b| a.bit_junction(b, conjunction))
    }
}
