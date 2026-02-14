use std::process::{ExitCode, Termination};

use crate::parser::ParserResult;

#[repr(u8)]
pub enum ExitValue {
    Standard = 0,
    Satisfiable = 10,
    WrongSatisfiable = 11,
    Unsatisfiable = 20,
    WrongUnsatisfiable = 21,
    Unknown = 47,
    TimeLimitExceeded = 61,
    HeapLimitExceeded = 62,
}

impl ExitValue {
    pub fn from_parser_result(parser_result: ParserResult) -> Self {
        match parser_result {
            ParserResult::None => Self::Standard,
            ParserResult::Unknown => Self::Unknown,
            ParserResult::Known(value) => {
                if value {
                    Self::Satisfiable
                } else {
                    Self::Unsatisfiable
                }
            }
            ParserResult::Wrong(value) => {
                if value {
                    Self::WrongSatisfiable
                } else {
                    Self::WrongUnsatisfiable
                }
            }
        }
    }
}

impl Termination for ExitValue {
    fn report(self) -> ExitCode {
        ExitCode::from(self as u8)
    }
}
