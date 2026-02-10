use super::LinearSystem;

impl LinearSystem {
    pub fn ult(self, rhs: Self) -> Result<Self, ()> {
        let (Ok(lhs), Ok(rhs)) = (self.try_into_expression(), rhs.try_into_expression()) else {
            return Err(());
        };
        let result = lhs.ult(rhs)?;
        Ok(Self::from_expression(result))
    }

    pub fn ule(self, rhs: Self) -> Result<Self, ()> {
        let (Ok(lhs), Ok(rhs)) = (self.try_into_expression(), rhs.try_into_expression()) else {
            return Err(());
        };
        let result = lhs.ule(rhs)?;
        Ok(Self::from_expression(result))
    }
}
