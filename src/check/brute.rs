use crate::{
    check::Assignment,
    domain::bitvector::{
        RBound,
        abstr::{AbstractBitvector, BitvectorDomain},
        compute_u64_mask,
    },
};

impl super::Checker {
    #[allow(dead_code)]
    pub fn brute_force(&self) -> Option<Assignment> {
        let mut values = Vec::new();
        for width in self.variable_widths.iter().cloned() {
            values.push(AbstractBitvector::new(0, RBound::new(width)));
        }

        let mut assignment = Assignment { values };

        let mut iterators: Vec<_> = self
            .variable_widths
            .iter()
            .map(|width| (width, 0..compute_u64_mask(*width)))
            .collect();

        loop {
            let mut early = false;
            for (index, (width, iterator)) in iterators.iter_mut().enumerate() {
                match iterator.next() {
                    Some(value) => {
                        assignment.values[index] =
                            AbstractBitvector::new(value, RBound::new(**width));

                        early = true;
                        break;
                    }
                    None => {
                        *iterator = 0..compute_u64_mask(**width);
                        assignment.values[index] = AbstractBitvector::new(0, RBound::new(**width));
                    }
                }
            }
            if !early {
                break;
            }

            let result = self.eval_formula(&assignment, self.assertion);

            let Some(concrete_result) = result.concrete_value() else {
                panic!("Concrete values should produce concrete result");
            };

            //eprintln!("Assignments: {:?}, result: {:?}", assignments, result);

            if concrete_result.is_nonzero() {
                return Some(assignment);
            }
        }

        None
    }
}
