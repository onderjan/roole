use crate::{
    domain::{
        bitvector::{BitvectorBound, abstr::BitvectorDomain},
        traits::forward::Bitwise,
    },
    problem::{
        domain::OperationDomain,
        operation::{LinearOperationType, LinearPolynomial},
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
        self.bit_linear(rhs, true)
    }
    fn bit_or(self, rhs: Self) -> Self {
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
                    return Self::from_polynomial(LinearPolynomial::single_bit(constant));
                }
            }
        }

        let (OperationDomain::Linear(lhs), OperationDomain::Linear(rhs)) = (self, rhs) else {
            return Self::top(bound);
        };

        match (lhs.into_type(), rhs.into_type()) {
            (LinearOperationType::Polynomial(lhs), LinearOperationType::Polynomial(rhs)) => {
                if let Ok(polynomial) = lhs.bitwise_combine(rhs, conjunction) {
                    return Self::from_polynomial(polynomial);
                }
            }
            (LinearOperationType::System(lhs), LinearOperationType::System(rhs)) => {
                let system = if conjunction {
                    lhs.and(rhs)
                } else {
                    lhs.or(rhs)
                };
                if let Some(system) = system {
                    return Self::from_system(system);
                }
            }
            _ => {}
        }
        Self::top(bound)
    }
}
