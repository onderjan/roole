use std::fmt::Debug;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VariableId(pub usize);

#[derive(Clone, Debug)]
pub enum UniOp {
    Not,
}

#[derive(Clone, Debug)]
pub enum BiOp {
    Add,
    Sub,

    BitAnd,
    BitOr,
    BitXor,

    Eq,
}

#[derive(Clone)]
pub enum Formula {
    Variable(VariableId),
    UniOp(UniOp, Box<Formula>),
    BiOp(BiOp, Box<Formula>, Box<Formula>),
}

impl Debug for VariableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl Debug for Formula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Variable(var_id) => var_id.fmt(f),
            Self::UniOp(op, inner) => {
                if matches!(inner.as_ref(), Formula::Variable(_)) {
                    write!(f, "{:?}({:?})", op, inner)
                } else {
                    write!(f, "{:?}", op)?;
                    let mut franz = f.debug_tuple("");
                    franz.field(inner);
                    franz.finish()
                }
            }
            Self::BiOp(op, left, right) => {
                if matches!(
                    (left.as_ref(), right.as_ref()),
                    (Formula::Variable(_), Formula::Variable(_))
                ) {
                    write!(f, "{:?}({:?},{:?})", op, left, right)
                } else {
                    write!(f, "{:?}", op)?;
                    let mut franz = f.debug_tuple("");
                    franz.field(left);
                    franz.field(right);
                    franz.finish()
                }
            }
        }
    }
}
