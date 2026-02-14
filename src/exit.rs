use std::process::{ExitCode, Termination};

use num_derive::{FromPrimitive, ToPrimitive};

/// Result of running Roole.
#[derive(Clone, Copy, Debug)]
pub enum RooleResult {
    None,
    Unknown,
    Known(bool),
    Wrong(bool),
}

/// Roole exit value.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive)]
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
    pub fn from_roole_result(roole_result: RooleResult) -> Self {
        match roole_result {
            RooleResult::None => Self::Standard,
            RooleResult::Unknown => Self::Unknown,
            RooleResult::Known(value) => {
                if value {
                    Self::Satisfiable
                } else {
                    Self::Unsatisfiable
                }
            }
            RooleResult::Wrong(value) => {
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
