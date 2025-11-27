use num::{BigUint, One, ToPrimitive, Zero};

use crate::problem::Problem;

pub struct Stats {
    progress_bar: indicatif::ProgressBar,
    total_width: u64,
    num_leaves: BigUint,
    num_nodes: BigUint,
    opened_nodes: BigUint,
    closed_leaves: BigUint,
    num_learned: usize,
    num_backtracked: usize,
}
const PRECISION_CONST: u64 = 1_000_000;

impl Stats {
    pub fn new(problem: &Problem) -> Self {
        eprintln!("Solving SAT problem");

        let total_width: u64 = problem
            .variable_widths()
            .iter()
            .map(|width| *width as u64)
            .sum();

        let num_leaves = BigUint::one() << total_width;
        let num_nodes = (num_leaves.clone() * 2u32) - 1u32;

        let progress_bar = indicatif::ProgressBar::new(PRECISION_CONST);
        progress_bar.set_style(
            indicatif::ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {msg}")
                .unwrap(),
        );

        Self {
            progress_bar,
            total_width,
            num_leaves,
            num_nodes,
            opened_nodes: BigUint::zero(),
            closed_leaves: BigUint::zero(),
            num_learned: 0,
            num_backtracked: 0,
        }
    }

    pub fn inc_opened_nodes(&mut self) {
        self.opened_nodes += BigUint::one();
    }

    pub fn inc_learned(&mut self) {
        self.num_learned += 1;
    }

    pub fn inc_backtracked(&mut self) {
        self.num_backtracked += 1;
    }

    pub fn add_closed_leaves(&mut self, leaf_width: u64) {
        self.closed_leaves += BigUint::one() << leaf_width;
    }

    pub fn update_progress_bar(&self) {
        let progress = (self.closed_leaves.clone() * PRECISION_CONST) / self.num_leaves.clone();

        let progress_ratio = progress.to_f32().unwrap_or(f32::NAN) / PRECISION_CONST as f32;
        let progress_percent = progress_ratio * 100.;

        self.progress_bar
            .set_position(progress.to_u64().unwrap_or(0));
        self.progress_bar
            .set_message(format!("{:.2}%", progress_percent));
    }

    pub fn finish(&self) {
        self.update_progress_bar();
        self.progress_bar.finish();

        let percent_opened_nodes = percent(&self.opened_nodes, &self.num_nodes);
        let percent_closed_leaves = percent(&self.closed_leaves, &self.num_leaves);

        eprintln!(
            "Info: {} nodes, {} opened ({:.3}%); {} leaves, {} closed ({:.3}%), learned: {}, backtracked: {}",
            self.num_nodes,
            self.opened_nodes,
            percent_opened_nodes,
            self.num_leaves,
            self.closed_leaves,
            percent_closed_leaves,
            self.num_learned,
            self.num_backtracked
        );
    }

    pub fn total_width(&self) -> u64 {
        self.total_width
    }
}

fn percent(dividend: &BigUint, divisor: &BigUint) -> f32 {
    (dividend.clone() * PRECISION_CONST / divisor.clone())
        .to_f32()
        .unwrap_or(f32::NAN)
        / (PRECISION_CONST as f32)
        * 100.
}
