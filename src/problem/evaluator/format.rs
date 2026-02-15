use std::fmt::{Debug, Display, UpperHex};

use super::{EvaluableDomain, Evaluator};
use crate::problem::formula::{OperationId, VariableId};

impl<D: EvaluableDomain> Debug for Evaluator<'_, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, false)
    }
}

impl<D: EvaluableDomain> UpperHex for Evaluator<'_, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, true)
    }
}

impl<D: EvaluableDomain> Evaluator<'_, D> {
    fn format(&self, f: &mut std::fmt::Formatter<'_>, hex: bool) -> std::fmt::Result {
        let mut franz = f.debug_struct("Evaluator");

        struct FieldStr<'a>(&'a str);
        impl Debug for FieldStr<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.0, f)
            }
        }

        for (variable_id, variable) in self.problem.variables.iter().enumerate() {
            let variable_id = VariableId(variable_id);
            franz.field(format!("{:?}", variable_id).as_str(), &variable);
        }

        for (operation_id, operation) in self.problem.operations.iter().enumerate() {
            let result = &self.results[operation_id];
            let operation_id = OperationId(operation_id);
            let name = format!("{:?}", operation_id);

            let mut value = if hex {
                format!("{:#X}", operation)
            } else {
                format!("{:?}", operation)
            };
            if let Some(result) = result {
                if result.value.is_top() {
                    value += "*";
                } else {
                    value += &format!(" -({})-> ", result.remaining_uses);
                    if hex {
                        value += &format!("{:#X}", result.value)
                    } else {
                        value += &format!("{:?}", result.value)
                    }
                }
            }

            franz.field(&name, &FieldStr(&value));
        }

        franz.finish()
    }
}
