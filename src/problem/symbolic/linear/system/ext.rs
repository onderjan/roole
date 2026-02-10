use crate::domain::bitvector::RBound;

use super::LinearSystem;

impl LinearSystem {
    pub fn uext(self, new_bound: RBound) -> Result<Self, ()> {
        // we can only extend single expressions
        if let Ok(expression) = self.try_into_expression() {
            expression
                .uext(new_bound)
                .map(LinearSystem::from_expression)
                .map_err(|_| ())
        } else {
            Err(())
        }
    }

    pub fn sext(self, new_bound: RBound) -> Result<Self, ()> {
        // we can only extend single expressions
        if let Ok(expression) = self.try_into_expression() {
            expression
                .sext(new_bound)
                .map(LinearSystem::from_expression)
                .map_err(|_| ())
        } else {
            Err(())
        }
    }
}
