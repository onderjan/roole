use crate::domain::bitvector::{
    RBound,
    abstr::{AbstractBitvector, BitvectorDomain},
    compute_u64_mask,
};

impl super::Checker {
    pub fn brute_force(&self) {
        let mut assignments = Vec::new();
        for width in self.variable_widths.iter().cloned() {
            assignments.push(AbstractBitvector::new(0, RBound::new(width)));
        }

        let mut iterators: Vec<_> = self
            .variable_widths
            .iter()
            .map(|width| (width, 0..compute_u64_mask(*width)))
            .collect();

        let mut satisfiable = false;

        loop {
            let mut early = false;
            for (index, (width, iterator)) in iterators.iter_mut().enumerate() {
                match iterator.next() {
                    Some(value) => {
                        assignments[index] = AbstractBitvector::new(value, RBound::new(**width));

                        early = true;
                        break;
                    }
                    None => {
                        *iterator = 0..compute_u64_mask(**width);
                        assignments[index] = AbstractBitvector::new(0, RBound::new(**width));
                    }
                }
            }
            if !early {
                break;
            }

            let result = self.eval_formula(&assignments, self.assertion);

            let Some(concrete_result) = result.concrete_value() else {
                panic!("Concrete values should produce concrete result");
            };

            //eprintln!("Assignments: {:?}, result: {:?}", assignments, result);

            if concrete_result.is_nonzero() {
                satisfiable = true;
                eprintln!("Satisfiable: {:?}", assignments);
                break;
            }
        }
        if !satisfiable {
            eprintln!("Unsatisfiable");
        }
    }
}
