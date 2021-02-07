use crate::mip_solution::MIPSolution;
use crate::solver::{Solver, SolverConfig};

use efg_lite::game::Player;
use efg_lite::sse::{BoundedProblem, ValueBound};
use efg_lite::strategy::SequenceFormStrategy;
use efg_lite::treeplex::SequenceId;
use efg_lite::vector::TreeplexVector;

use std::ffi::CString;
use std::slice;

use optimizers::cbc_bindings::bindings::*;


/// Variables (columns) are ordered in the order
///
/// |L| = num_leaves
/// |S1| = num_sequences (leader)
/// |S2| = num_sequences (follower)
/// |I2| = num_infosets (follower)
///
/// Probability of reaching leaves: [0,..., |L|) --- Technically we only need leaf variables within the (follower's) trunk.
/// Number of slack variables for follower: [|L|,...|L|+|S2|) --- Technically we do not need one for the empty sequence, but w/e.
/// Value of information sets for follower: [|L|+|S2|,...|L|+|S2|+|I2|)
/// Sequence form representation of leader: [|L|+|S2|+|I2|,...|L|+|S1|+|S2|+|I2|)
/// Sequence form representation of the follower: [|L|+|S1|+|S2|+|I2|,...|L|+|S1|+2|S2|+|I2|)
///
pub struct CbcSolver<'a> {
    problem: &'a BoundedProblem,
    model: *mut core::ffi::c_void,
}

impl<'a> CbcSolver<'a> {

    fn get_leaf_prob_index(&self, leaf_index: usize) -> usize {
        leaf_index
    }

    fn get_value_slack_index(&self, sequence_id: SequenceId) -> usize {
        sequence_id + self.problem.leaves_within_trunk.len()
    }

    fn get_value_infoset_index(&self, infoset_id: usize) -> usize {
        infoset_id
            + self.problem.leaves_within_trunk.len()
            + self.problem.game.treeplex(Player::Player2).num_sequences()
    }

    fn get_seq_form_index(&self, player: Player, sequence_id: SequenceId) -> usize {
        match player {
            Player::Player1 => self.get_seq_form_index_pl1(sequence_id),
            Player::Player2 => self.get_seq_form_index_pl2(sequence_id),
        }
    }

    fn get_seq_form_index_pl1(&self, sequence_id: SequenceId) -> usize {
        sequence_id
            + self.problem.leaves_within_trunk.len()
            + self.problem.game.treeplex(Player::Player2).num_sequences()
            + self.problem.game.treeplex(Player::Player2).num_infosets()
    }

    fn get_seq_form_index_pl2(&self, sequence_id: SequenceId) -> usize {
        sequence_id
            + self.problem.leaves_within_trunk.len()
            + self.problem.game.treeplex(Player::Player2).num_sequences()
            + self.problem.game.treeplex(Player::Player2).num_infosets()
            + self.problem.game.treeplex(Player::Player1).num_sequences()
    }

    fn get_num_variables(&self) -> usize {
        self.problem.leaves_within_trunk.len()
            + self.problem.game.treeplex(Player::Player2).num_sequences()
            + self.problem.game.treeplex(Player::Player2).num_infosets()
            + self.problem.game.treeplex(Player::Player1).num_sequences()
            + self.problem.game.treeplex(Player::Player2).num_sequences()
    }

    fn get_M(&self) -> f64 {
        100.0 // TODO (chunkail) use a better value...
    }

    fn get_solution_strategies(&self, player: Player) -> SequenceFormStrategy<'a> {
        let treeplex = self.problem.game.treeplex(player);
        let num_sequences = self.problem.game.treeplex(player).num_sequences();
        unsafe {
            let ptr = Cbc_getColSolution(self.model);
            let array: &[f64] = match player {
                Player::Player1 => slice::from_raw_parts(
                    ptr.offset(self.get_seq_form_index_pl1(0) as isize),
                    num_sequences,
                ),
                Player::Player2 => slice::from_raw_parts(
                    ptr.offset(self.get_seq_form_index_pl2(0) as isize),
                    num_sequences,
                ),
            };
            let treeplex_vector = TreeplexVector::from_array(treeplex, array.clone());
            SequenceFormStrategy::from_treeplex_vector(treeplex_vector)
        }
    }

    fn get_objective_value(&self) -> f64 {
        unsafe { Cbc_getObjValue(self.model) }
    }

    fn get_solution_leaf_probs(&self) -> Vec<f64> {
        unsafe {
            let ptr = Cbc_getColSolution(self.model);
            let array: &[f64] = slice::from_raw_parts(
                ptr.offset(self.get_leaf_prob_index(0) as isize),
                self.problem.game.payoff_matrix().entries.len(),
            );
            array.to_vec()
        }
    }

    fn get_solution_slack(&self) -> TreeplexVector<'a> {
        let follower_treeplex = self.problem.game.treeplex(Player::Player2);
        let num_sequences = follower_treeplex.num_sequences();
        unsafe {
            let ptr = Cbc_getColSolution(self.model);
            let array: &[f64] = slice::from_raw_parts(
                ptr.offset(self.get_value_slack_index(0) as isize),
                num_sequences,
            );
            TreeplexVector::from_array(follower_treeplex, array)
        }
    }

    /// Follower's value
    fn get_solution_follower_value(&self) -> Vec<f64> {
        let follower_treeplex = self.problem.game.treeplex(Player::Player2);
        let num_infosets = follower_treeplex.num_infosets();
        unsafe {
            let ptr = Cbc_getColSolution(self.model);
            let array: &[f64] = slice::from_raw_parts(
                ptr.offset(self.get_value_infoset_index(0) as isize),
                num_infosets,
            );
            array.to_vec()
        }
    }

    fn make_bounds_constraints(&self) {
        for (infoset_id, value_bound) in self.problem.bounds.iter() {
            let col_indices: Vec<i32> = vec![self.get_value_infoset_index(*infoset_id) as i32];
            let col_coeffs: Vec<f64> = vec![1.0];

            match value_bound {
                ValueBound::LowerBound(lb) => unsafe {
                    Cbc_addRow(
                        self.model,
                        CString::new(format!("value_bounds_{}", infoset_id))
                            .unwrap()
                            .as_ptr(),
                        1,
                        col_indices.as_slice().as_ptr(),
                        col_coeffs.as_slice().as_ptr(),
                        'G' as i8,
                        *lb,
                    );
                },
                ValueBound::UpperBound(ub) => unsafe {
                    Cbc_addRow(
                        self.model,
                        CString::new(format!("value_bounds_{}", infoset_id))
                            .unwrap()
                            .as_ptr(),
                        1,
                        col_indices.as_slice().as_ptr(),
                        col_coeffs.as_slice().as_ptr(),
                        'L' as i8,
                        *ub,
                    );
                },
                ValueBound::None => {}
            }
        }
    }

    /// Constraints (17)
    fn make_slack_constraints(&self) {
        // Constraints for slack variables.
        for (parent_infoset_id, parent_infoset) in self
            .problem
            .game
            .treeplex(Player::Player2)
            .infosets()
            .iter()
            .enumerate()
        {
            for sequence_id in parent_infoset.start_sequence..=parent_infoset.end_sequence {
                let child_infoset_id_range = self
                    .problem
                    .treeplex_follower_tools
                    .seq_to_infoset_range(sequence_id);
                let leaf_indices = self
                    .problem
                    .game_tools
                    .leaf_indices_at_sequence(Player::Player2, sequence_id);
                let nz: i32 = 2 + child_infoset_id_range.len() as i32 + leaf_indices.len() as i32;

                // Get cols and coeffs for cols.
                let mut col_indices: Vec<i32> = vec![];
                let mut col_coeffs: Vec<f64> = vec![];

                // We go about the constraints from the left to right from the
                // original paper by Bonsansky and Cermak.
                col_indices.push(self.get_value_infoset_index(parent_infoset_id) as i32);
                col_coeffs.push(1.0);

                col_indices.push(self.get_value_slack_index(sequence_id) as i32);
                col_coeffs.push(-1.0);

                for child_infoset_id in child_infoset_id_range {
                    col_indices.push(self.get_value_infoset_index(child_infoset_id) as i32);
                    col_coeffs.push(-1.0);
                }
                for leaf_index in leaf_indices.iter() {
                    let leaf = self.problem.game.payoff_matrix().entries[*leaf_index];
                    let leader_sequence = leaf.seq_pl1;
                    assert!(
                        leader_sequence
                            < self.problem.game.treeplex(Player::Player1).num_sequences()
                    );
                    col_indices.push(self.get_seq_form_index_pl1(leader_sequence) as i32);
                    col_coeffs.push(-leaf.payoff_pl2 * leaf.chance_factor);
                }

                assert_eq!(col_indices.len(), col_coeffs.len());
                assert_eq!(col_indices.len(), nz as usize);
                unsafe {
                    Cbc_addRow(
                        self.model,
                        CString::new(format!("slack_constraints_{}", sequence_id))
                            .unwrap()
                            .as_ptr(),
                        nz,
                        col_indices.as_slice().as_ptr(),
                        col_coeffs.as_slice().as_ptr(),
                        'E' as i8,
                        0f64,
                    );

                }
            }
        }
    }

    /// Constraints (18, 19).
    fn make_sequence_form_constraints(&self, player: Player) {
        let treeplex = self.problem.game.treeplex(player);
        let empty_sequence_id = treeplex.empty_sequence_id();

        // Get cols and coeffs for empty sequence constraint.
        let col_indices: Vec<i32> = vec![self.get_seq_form_index(player, empty_sequence_id) as i32];
        let col_coeffs: Vec<f64> = vec![1.0];
        unsafe {
            Cbc_addRow(
                self.model,
                CString::new(format!("seq_form_constraints_empty_seq"))
                    .unwrap()
                    .as_ptr(),
                1,
                col_indices.as_slice().as_ptr(),
                col_coeffs.as_slice().as_ptr(),
                'E' as i8,
                1f64,
            );
        }

        for (infoset_id, infoset) in treeplex.infosets().iter().enumerate() {
            // Get cols and coeffs for non-empty sequence constraint.
            let mut col_indices: Vec<i32> =
                vec![self.get_seq_form_index(player, infoset.parent_sequence) as i32];
            let mut col_coeffs: Vec<f64> = vec![1.0];

            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                col_indices.push(self.get_seq_form_index(player, sequence_id) as i32);
                col_coeffs.push(-1.0);
            }

            let nz = col_indices.len() as i32;

            assert_eq!(col_indices.len(), col_coeffs.len());
            assert_eq!(col_indices.len(), nz as usize);
            unsafe {
                Cbc_addRow(
                    self.model,
                    CString::new(format!("seq_form_constraints_{}", infoset_id))
                        .unwrap()
                        .as_ptr(),
                    nz,
                    col_indices.as_slice().as_ptr(),
                    col_coeffs.as_slice().as_ptr(),
                    'E' as i8,
                    0f64,
                );
            }
        }
    }

    /// Constraints (20).
    fn make_on_off_constraints(&self) {
        for sequence_id in 0..self.problem.game.treeplex(Player::Player2).num_sequences() {
            let col_indices: Vec<i32> = vec![
                self.get_value_slack_index(sequence_id) as i32,
                self.get_seq_form_index_pl2(sequence_id) as i32,
            ];
            let col_coeffs: Vec<f64> = vec![1.0, self.get_M()];

            unsafe {
                Cbc_addRow(
                    self.model,
                    CString::new(format!("on_off_constraints_{}", sequence_id))
                        .unwrap()
                        .as_ptr(),
                    2,
                    col_indices.as_slice().as_ptr(),
                    col_coeffs.as_slice().as_ptr(),
                    'L' as i8,
                    self.get_M(),
                );
            }
        }
    }

    /// Constraints (21, 22).
    fn make_leaf_max_prob_constraints(&self) {
        for (idx, leaf_index) in self.problem.leaves_within_trunk.iter().enumerate() {
            // println!("trunk_index {:?}, leaf_index {:?}", idx, leaf_index);
            let leaf = self.problem.game.payoff_matrix().entries[*leaf_index];

            // Player 1.
            let col_indices: Vec<i32> = vec![
                self.get_leaf_prob_index(idx) as i32,
                self.get_seq_form_index_pl1(leaf.seq_pl1) as i32,
            ];
            let col_coeffs: Vec<f64> = vec![1.0, -1.0];
            // println!("Pl1 col indices {:?}", col_indices);

            unsafe {
                Cbc_addRow(
                    self.model,
                    CString::new(format!(
                        "prob_max_constraints_pl1_trunk_{}_leaf_{}",
                        idx, *leaf_index
                    ))
                    .unwrap()
                    .as_ptr(),
                    2,
                    col_indices.as_slice().as_ptr(),
                    col_coeffs.as_slice().as_ptr(),
                    'L' as i8,
                    0f64,
                );
            }

            // Player 2.
            let col_indices: Vec<i32> = vec![
                self.get_leaf_prob_index(idx) as i32,
                self.get_seq_form_index_pl2(leaf.seq_pl2) as i32,
            ];
            let col_coeffs: Vec<f64> = vec![1.0, -1.0];
            // println!("Pl2 col indices {:?}", col_indices);

            unsafe {
                Cbc_addRow(
                    self.model,
                    CString::new(format!(
                        "prob__max_constraints_pl2_trunk_{}_leaf_{}",
                        idx, *leaf_index
                    ))
                    .unwrap()
                    .as_ptr(),
                    2,
                    col_indices.as_slice().as_ptr(),
                    col_coeffs.as_slice().as_ptr(),
                    'L' as i8,
                    0f64,
                );
            }
        }
    }

    /// Constraint (23).
    fn make_leaf_sum_prob_constraints(&self) {
        let mut col_indices: Vec<i32> = vec![];
        let mut col_coeffs: Vec<f64> = vec![];

        for (idx, leaf_index) in self.problem.leaves_within_trunk.iter().enumerate() {
            let leaf = self.problem.game.payoff_matrix().entries[*leaf_index];
            col_indices.push(self.get_leaf_prob_index(idx) as i32);
            col_coeffs.push(leaf.chance_factor);
        }

        unsafe {
            Cbc_addRow(
                self.model,
                CString::new(format!("prob_sum_constraints"))
                    .unwrap()
                    .as_ptr(),
                col_indices.len() as i32,
                col_indices.as_slice().as_ptr(),
                col_coeffs.as_slice().as_ptr(),
                'E' as i8,
                self.problem.input_mass,
            );
        }
    }


    fn make_variables(&self) {
        // Add leaf probabilities.
        for leaf_index in self.problem.leaves_within_trunk.iter() {
            let leaf = self.problem.game.payoff_matrix().entries[*leaf_index];
            unsafe {
                Cbc_addCol(
                    self.model,
                    CString::new(format!("leaf_probabilities_{}", leaf_index))
                        .unwrap()
                        .as_ptr(),
                    0f64,                                 // Probabilities must be >= 0.
                    std::f64::INFINITY, // Will add upper bounds later on (depends on other variables).
                    leaf.chance_factor * leaf.payoff_pl1, // Expression (16)
                    0,                  // Continuous variable
                    0,                  // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                );

            }
        }
        // Slack variables (for each follower sequence).
        for follower_sequence in 0..self.problem.game.treeplex(Player::Player2).num_sequences() {
            unsafe {
                Cbc_addCol(
                    self.model,
                    CString::new(format!("follower_slack_{}", follower_sequence))
                        .unwrap()
                        .as_ptr(),
                    0f64,                 // Slack must be >= 0
                    std::f64::INFINITY, // Will add upper bounds later on (depends on other variables).
                    0f64,               // No objective.
                    0,                  // Continuous variable.
                    0,                  // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                );

            }
        }
        // Infoset values (followers).
        for follower_infoset in 0..self.problem.game.treeplex(Player::Player2).num_infosets() {
            unsafe {
                Cbc_addCol(
                    self.model,
                    CString::new(format!("follower_infoset_value_{}", follower_infoset))
                        .unwrap()
                        .as_ptr(),
                    std::f64::NEG_INFINITY, // Values can range from -INF to INF
                    std::f64::INFINITY,     // Values can range from -INF to INF
                    0f64,                   // No objective.
                    0,                      // Continuous variable.
                    0,                      // Will add constraints later on.
                    std::ptr::null_mut(),   // Will add constraints later on.
                    std::ptr::null_mut(),   // Will add constraints later on.
                );

            }
        }
        // Sequence form representation (leader).
        for leader_sequence in 0..self.problem.game.treeplex(Player::Player1).num_sequences() {
            unsafe {
                Cbc_addCol(
                    self.model,
                    CString::new(format!("leader_sequence_form_{}", leader_sequence))
                        .unwrap()
                        .as_ptr(),
                    0f64,                 // Sequence probability must be >= 0
                    1f64, // Sequence probability may not be more than 1 (not really required in theory).
                    0f64, // No objective.
                    0,    // Continuous variable.
                    0,    // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                );

            }
        }
        // Sequence form representation (follower).
        for follower_sequence in 0..self.problem.game.treeplex(Player::Player2).num_sequences() {
            unsafe {
                Cbc_addCol(
                    self.model,
                    CString::new(format!("follower_sequence_form_{}", follower_sequence))
                        .unwrap()
                        .as_ptr(),
                    0f64,                 // Sequence probability must be >= 0
                    1f64, // Sequence probability may not be more than 1 (not really required in theory).
                    0f64, // No objective.
                    1,    // Integer variable.
                    0,    // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                );

            }
        }

        unsafe {
            let num_cols = Cbc_getNumCols(self.model);
            println!("Number of variables {:?}", num_cols);
            assert_eq!(num_cols, self.get_num_variables() as i32);
        }
    }

}

impl<'a> Solver<'a> for CbcSolver<'a> {
    fn new(problem: &'a BoundedProblem, solver_config: &SolverConfig) -> CbcSolver<'a> {
        unsafe {
            let model = Cbc_newModel();
            Cbc_setProblemName(model, CString::new("Skinny-sse-model").unwrap().as_ptr());
            let cbc_solver = CbcSolver { problem, model };

            println!(
                "Num sequences P1 {:?}",
                problem.game.treeplex(Player::Player1).num_sequences()
            );
            println!(
                "Num sequences P2 {:?}",
                problem.game.treeplex(Player::Player2).num_sequences()
            );
            println!(
                "Num infosets P1 {:?}",
                problem.game.treeplex(Player::Player1).num_infosets()
            );
            println!(
                "Num infosets P2 {:?}",
                problem.game.treeplex(Player::Player2).num_infosets()
            );

            println!("Making variables");
            cbc_solver.make_variables();

            println!("Making slack constraints");
            cbc_solver.make_slack_constraints();

            println!("Making sequence form constraints");
            cbc_solver.make_sequence_form_constraints(Player::Player1);
            cbc_solver.make_sequence_form_constraints(Player::Player2);

            println!("Making on-off constraints");
            cbc_solver.make_on_off_constraints();

            println!("Making leaf max prob constraints");
            cbc_solver.make_leaf_max_prob_constraints();

            println!("Making leaf sum prob constraints");
            cbc_solver.make_leaf_sum_prob_constraints();

            cbc_solver.make_bounds_constraints();

            Cbc_setObjSense(model, -1f64); // Maximization objective.

            // Cbc_setLogLevel(model, 0); // Set verbose mode to 0.

            // println!("BABABABA {:?}", cbc_solver.problem.leaves_within_trunk.len() + cbc_solver.problem.game.treeplex(Player::Player2).num_sequences());
            cbc_solver
        }
    }

    fn solve(&self) {
        unsafe {
            Cbc_solve(self.model);
        }
    }

    fn get_solution(&self) -> MIPSolution<'a> {
        MIPSolution::new(
            self.get_solution_strategies(Player::Player1),
            self.get_solution_strategies(Player::Player2),
            self.get_solution_leaf_probs(),
            self.get_objective_value(),
            self.get_solution_slack(),
            self.get_solution_follower_value(),
        )
    }

}

/*
impl<'a> Drop for CbcSolver<'a> {
    fn drop(&mut self) {
        unsafe {
            Cbc_deleteModel(self.model);
        }
    }
}
*/