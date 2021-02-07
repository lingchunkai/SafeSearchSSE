use crate::mip_solution::MIPSolution;
use crate::solver::{Solver, SolverConfig};

use efg_lite::game::Player;
use efg_lite::sse::{BoundedProblem, ValueBound, BlueprintBr};
use efg_lite::strategy::SequenceFormStrategy;
use efg_lite::treeplex::SequenceId;
use efg_lite::vector::TreeplexVector;

use std::ffi::CString;
use std::slice;

use optimizers::gurobi_bindings::bindings::*;

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
pub struct GurobiSolver<'a> {
    problem: &'a BoundedProblem,
    env: *mut GRBenv, // TODO: make static, so we don't have to reconstruct each time.
    model: *mut GRBmodel, // Gurobi model.
}

impl<'a> GurobiSolver<'a> {
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
            let mut dst = Vec::<f64>::new();
            dst.resize(treeplex.num_sequences(), 0f64);
            match player {
                Player::Player1 => {
                    GRBgetdblattrarray(
                        self.model,
                        CString::new("X").unwrap().as_ptr(),
                        self.get_seq_form_index_pl1(0) as i32,
                        treeplex.num_sequences() as i32,
                        dst.as_mut_ptr(),
                    );
                }
                Player::Player2 => {
                    GRBgetdblattrarray(
                        self.model,
                        CString::new("X").unwrap().as_ptr(),
                        self.get_seq_form_index_pl2(0) as i32,
                        treeplex.num_sequences() as i32,
                        dst.as_mut_ptr(),
                    );
                }
            }
            let treeplex_vector = TreeplexVector::from_vec(treeplex, dst);
            SequenceFormStrategy::from_treeplex_vector(treeplex_vector)
        }
    }

    fn get_objective_value(&self) -> f64 {
        unsafe {
            let mut objective_value: f64 = 0f64;
            GRBgetdblattr(
                self.model,
                CString::new("ObjVal").unwrap().as_ptr(),
                &mut objective_value,
            );
            objective_value
        }
    }

    fn get_solution_leaf_probs(&self) -> Vec<f64> {
        let num_leaves = self.problem.game.payoff_matrix().entries.len();
        unsafe {
            let mut dst = Vec::<f64>::new();
            dst.resize(num_leaves, 0f64);
            GRBgetdblattrarray(
                self.model,
                CString::new("X").unwrap().as_ptr(),
                self.get_leaf_prob_index(0) as i32,
                num_leaves as i32,
                dst.as_mut_ptr(),
            );
            dst
        }
    }

    fn get_solution_slack(&self) -> TreeplexVector<'a> {
        let follower_treeplex = self.problem.game.treeplex(Player::Player2);
        let num_sequences = follower_treeplex.num_sequences();
        unsafe {
            let mut dst = Vec::<f64>::new();
            dst.resize(num_sequences, 0f64);
            GRBgetdblattrarray(
                self.model,
                CString::new("X").unwrap().as_ptr(),
                self.get_value_slack_index(0) as i32,
                num_sequences as i32,
                dst.as_mut_ptr(),
            );
            TreeplexVector::from_vec(follower_treeplex, dst)
        }
    }

    /// Follower's value
    fn get_solution_follower_value(&self) -> Vec<f64> {
        let follower_treeplex = self.problem.game.treeplex(Player::Player2);
        let num_infosets = follower_treeplex.num_infosets();
        unsafe {
            let mut dst = Vec::<f64>::new();
            dst.resize(num_infosets, 0f64);
            GRBgetdblattrarray(
                self.model,
                CString::new("X").unwrap().as_ptr(),
                self.get_value_infoset_index(0) as i32,
                num_infosets as i32,
                dst.as_mut_ptr(),
            );
            dst
        }
    }

    fn make_bounds_constraints(&self) {
        for (infoset_id, value_bound) in self.problem.bounds.iter() {
            let mut col_indices: Vec<i32> = vec![self.get_value_infoset_index(*infoset_id) as i32];
            let mut col_coeffs: Vec<f64> = vec![1.0];

            match value_bound {
                ValueBound::LowerBound(lb) => unsafe {
                    let err = GRBaddconstr(
                        self.model,
                        1,
                        col_indices.as_mut_ptr(),
                        col_coeffs.as_mut_ptr(),
                        GRB_GREATER_EQUAL as i8,
                        *lb,
                        CString::new(format!("value_bounds_{}", infoset_id))
                            .unwrap()
                            .as_ptr(),
                    );
                    assert_eq!(err, 0);
                },
                ValueBound::UpperBound(ub) => unsafe {
                    let err = GRBaddconstr(
                        self.model,
                        1,
                        col_indices.as_mut_ptr(),
                        col_coeffs.as_mut_ptr(),
                        GRB_LESS_EQUAL as i8,
                        *ub,
                        CString::new(format!("value_bounds_{}", infoset_id))
                            .unwrap()
                            .as_ptr(),
                    );
                    assert_eq!(err, 0);
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
                    // println!("{:?}", leaf);
                    // println!("bloody nan {:?}", -leaf.payoff_pl2);
                }

                assert_eq!(col_indices.len(), col_coeffs.len());
                assert_eq!(col_indices.len(), nz as usize);
                unsafe {
                    let err = 
                    GRBaddconstr(
                        self.model,
                        nz,
                        col_indices.as_mut_ptr(),
                        col_coeffs.as_mut_ptr(),
                        GRB_EQUAL as i8,
                        0f64,
                        CString::new(format!("slack_constraints_{}", sequence_id))
                            .unwrap()
                            .as_ptr(),
                    );
                    // println!("{:?}, {:?}", col_indices, col_coeffs);
                    assert_eq!(err, 0);
                }
            }
        }
    }

    /// Constraints (18, 19).
    fn make_sequence_form_constraints(&self, player: Player) {
        println!("================= PLAYER {:?} ======================", player);

        let treeplex = self.problem.game.treeplex(player);
        let empty_sequence_id = treeplex.empty_sequence_id();

        println!("EMPTY SEQUENCE ID {:?} {:?}", empty_sequence_id, treeplex.num_sequences());

        // Get cols and coeffs for empty sequence constraint.
        let mut col_indices: Vec<i32> =
            vec![self.get_seq_form_index(player, empty_sequence_id) as i32];
        let mut col_coeffs: Vec<f64> = vec![1.0];
        unsafe {
                let err = GRBupdatemodel(self.model);
                // println!("wtf {:?}", err);
                let mut r : i32 = 0;
                let err2 = GRBgetintattr(self.model, 
                            CString::new("NumVars").unwrap().as_ptr(), 
                            &mut r);
                // println!("err 2 {:?}", err2);
                assert!(err2 == 0);
                println!("Num variables {:?}", r);
                println!("{:?} {:?}", col_indices, self.get_num_variables());
            let err = 
            GRBaddconstr(
                self.model,
                1,
                col_indices.as_mut_ptr(),
                col_coeffs.as_mut_ptr(),
                GRB_EQUAL as i8,
                1f64,
                CString::new(format!("seq_form_constraints_empty_seq"))
                    .unwrap()
                    .as_ptr(),
            );
            if err != 0 {
                let mut r : i32 = 0;
                let err2 = GRBgetintattr(self.model, 
                            CString::new("NumVars").unwrap().as_ptr(), 
                            &mut r);
                // println!("err 2 {:?}", err2);
                assert!(err2 == 0);
                println!("Num variables {:?}", r);
                println!("{:?} {:?}", col_indices, self.get_num_variables());
            }
            assert_eq!(err, 0);
        }

        let mut sequences_touched = std::vec::from_elem(false, treeplex.num_sequences());

        for (infoset_id, infoset) in treeplex.infosets().iter().enumerate() {
            // Get cols and coeffs for non-empty sequence constraint.
            let mut col_indices: Vec<i32> =
                vec![self.get_seq_form_index(player, infoset.parent_sequence) as i32];
            let mut col_coeffs: Vec<f64> = vec![1.0];

            // println!("{:?}   {:?}", infoset_id, infoset.parent_sequence);
            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                assert_eq!(sequences_touched[sequence_id], false);
                sequences_touched[sequence_id] = true;
                col_indices.push(self.get_seq_form_index(player, sequence_id) as i32);
                col_coeffs.push(-1.0);
                // println!("      {:?}", sequence_id);
            }

            let nz = col_indices.len() as i32;

            assert_eq!(col_indices.len(), col_coeffs.len());
            assert_eq!(col_indices.len(), nz as usize);
            // println!("{:?}", col_indices);
            unsafe {
                let err = 
                GRBaddconstr(
                    self.model,
                    nz,
                    col_indices.as_mut_ptr(),
                    col_coeffs.as_mut_ptr(),
                    GRB_EQUAL as i8,
                    0f64,
                    CString::new(format!("seq_form_constraints_{}_{:?}", infoset_id, player))
                        .unwrap()
                        .as_ptr(),
                );
                assert_eq!(err, 0);
            }
        }
    }

    fn make_on_off_constraints(&self) {
        // self._make_on_off_constraints_using_indicator();
        self._make_on_off_constraints_using_big_M();
    }

    /// Constraints (20) using big M method.
    fn _make_on_off_constraints_using_big_M(&self) {
        for sequence_id in 0..self.problem.game.treeplex(Player::Player2).num_sequences() {
            let mut col_indices: Vec<i32> = vec![
                self.get_value_slack_index(sequence_id) as i32,
                self.get_seq_form_index_pl2(sequence_id) as i32,
            ];
            let mut col_coeffs: Vec<f64> = vec![1.0, self.get_M()];

            unsafe {
                let err = GRBaddconstr(
                    self.model,
                    2,
                    col_indices.as_mut_ptr(),
                    col_coeffs.as_mut_ptr(),
                    GRB_LESS_EQUAL as i8,
                    self.get_M(),
                    CString::new(format!("on_off_constraints_{}", sequence_id))
                        .unwrap()
                        .as_ptr(),
                );
                assert_eq!(err, 0);
            }
        }
    }

    /// Implement (2) use gurobi general constraints (indicator ).
    fn _make_on_off_constraints_using_indicator(&self) {
        for sequence_id in 0..self.problem.game.treeplex(Player::Player2).num_sequences() {
            let mut col_indices: Vec<i32> = vec![self.get_value_slack_index(sequence_id) as i32];
            let mut col_coeffs: Vec<f64> = vec![1.0];

            unsafe {
                GRBaddgenconstrIndicator(
                    self.model,
                    CString::new(format!("on_off_constraints_{}", sequence_id))
                        .unwrap()
                        .as_ptr(),
                    self.get_seq_form_index_pl2(sequence_id) as i32,
                    1 as i32,
                    1 as i32,
                    col_indices.as_mut_ptr(),
                    col_coeffs.as_mut_ptr(),
                    GRB_EQUAL as i8,
                    0f64,
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
            let mut col_indices: Vec<i32> = vec![
                self.get_leaf_prob_index(idx) as i32,
                self.get_seq_form_index_pl1(leaf.seq_pl1) as i32,
            ];
            let mut col_coeffs: Vec<f64> = vec![1.0, -1.0];
            // println!("Pl1 col indices {:?}", col_indices);

            // if idx % 100 == 0 {
            //    println!("{:?}/{:?}", idx, self.problem.leaves_within_trunk.len());
            //}

            unsafe {
                let err = GRBaddconstr(
                    self.model,
                    2,
                    col_indices.as_mut_ptr(),
                    col_coeffs.as_mut_ptr(),
                    GRB_LESS_EQUAL as i8,
                    0f64,
                    std::ptr::null_mut(),
                    /*
                    CString::new(format!(
                        "prob_max_constraints_pl1_trunk_{}_leaf_{}",
                        idx, *leaf_index
                    )
                    .unwrap()
                    .as_ptr(),
                    */
                );
                assert_eq!(err, 0);
            }

            // Player 2.
            let mut col_indices: Vec<i32> = vec![
                self.get_leaf_prob_index(idx) as i32,
                self.get_seq_form_index_pl2(leaf.seq_pl2) as i32,
            ];
            let mut col_coeffs: Vec<f64> = vec![1.0, -1.0];
            // println!("Pl2 col indices {:?}", col_indices);

            unsafe {
                let err = GRBaddconstr(
                    self.model,
                    2,
                    col_indices.as_mut_ptr(),
                    col_coeffs.as_mut_ptr(),
                    GRB_LESS_EQUAL as i8,
                    0f64,
                    std::ptr::null_mut(),
                    /*
                    CString::new(format!(
                        "prob__max_constraints_pl2_trunk_{}_leaf_{}",
                        idx, *leaf_index
                    )
                    )
                    .unwrap()
                    .as_ptr(),
                    */
                );
                assert_eq!(err, 0);
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
            let err = GRBaddconstr(
                self.model,
                col_indices.len() as i32,
                col_indices.as_mut_ptr(),
                col_coeffs.as_mut_ptr(),
                GRB_EQUAL as i8,
                self.problem.input_mass,
                CString::new(format!("prob_sum_constraints"))
                    .unwrap()
                    .as_ptr(),
            );
            assert_eq!(err, 0);
        }
    }

    fn make_variables(&self) {
        // Add leaf probabilities.
        for leaf_index in self.problem.leaves_within_trunk.iter() {
            let leaf = self.problem.game.payoff_matrix().entries[*leaf_index];
            unsafe {
                let err = GRBaddvar(
                    self.model,
                    0,                                    // Will add constraints later on.
                    std::ptr::null_mut(),                 // Will add constraints later on.
                    std::ptr::null_mut(),                 // Will add constraints later on.
                    leaf.chance_factor * leaf.payoff_pl1, // Expression (16)
                    0f64,                                 // Probabilities must be >= 0.
                    GRB_INFINITY,
                    'C' as i8, // GRB_CONTINUOUS,
                    CString::new(format!("leaf_probabilities_{}", leaf_index))
                        .unwrap()
                        .as_ptr(),
                );
                assert_eq!(err, 0);
            }
        }
        // Slack variables (for each follower sequence).
        for follower_sequence in 0..self.problem.game.treeplex(Player::Player2).num_sequences() {
            unsafe {
                let err = GRBaddvar(
                    self.model,
                    0,                    // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    0f64,                 // No objective.
                    0f64,                 // Slack must be >= 0.
                    GRB_INFINITY, // Will add upper bound later on (depends on other variables).
                    'C' as i8,    // GRB_CONTINUOUS,
                    CString::new(format!("follower_slack_{}", follower_sequence))
                        .unwrap()
                        .as_ptr(),
                );
                assert_eq!(err, 0);
            }
        }
        // Infoset values (followers).
        for follower_infoset in 0..self.problem.game.treeplex(Player::Player2).num_infosets() {
            unsafe {
                let err = GRBaddvar(
                    self.model,
                    0,                    // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    0f64,                 // No objective.
                    -GRB_INFINITY,        // Values can range from -INF to INF
                    GRB_INFINITY,         // Values can range from -INF to INF
                    'C' as i8,            // GRB_CONTINUOUS,
                    CString::new(format!("follower_infoset_value_{}", follower_infoset))
                        .unwrap()
                        .as_ptr(),
                );
                assert_eq!(err, 0);
            }
        }
        // Sequence form representation (leader).
        for leader_sequence in 0..self.problem.game.treeplex(Player::Player1).num_sequences() {
            unsafe {
                let err = 
                GRBaddvar(
                    self.model,
                    0,                    // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    0f64,                 // No objective.
                    0f64,                 // Sequence probability must be >= 0
                    1f64, // Sequence probability may not be more than 1 (not really required in theory).
                    'C' as i8, // GRB_CONTINUOUS,
                    CString::new(format!("leader_sequence_form_{}", leader_sequence))
                        .unwrap()
                        .as_ptr(),
                );
                assert_eq!(err, 0);
            }
        }
        // Sequence form representation (follower).
        for follower_sequence in 0..self.problem.game.treeplex(Player::Player2).num_sequences() {
            unsafe {
                let err = GRBaddvar(
                    self.model,
                    0,                    // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    0f64,                 // No objective.
                    0f64,                 // Sequence probability must be >= 0
                    1f64, // Sequence probability may not be more than 1 (not really required in theory).
                    'B' as i8, // Binary variable.
                    CString::new(format!("follower_sequence_form_{}", follower_sequence))
                        .unwrap()
                        .as_ptr(),
                );
                assert_eq!(err, 0);
            }
        }
    }

    fn set_model_sense(&self) {
        unsafe {
            GRBsetintattr(
                self.model,
                CString::new("ModelSense").unwrap().as_ptr(),
                -1, // Maximization problem.
            );
        }
    }

    fn set_time_limit(env: *mut GRBenv, time_limit: f64) {
        unsafe {
            let err = GRBsetdblparam(env, CString::new("TimeLimit").unwrap().as_ptr(), time_limit);
        }
    }

    fn set_feas_tol(env: *mut GRBenv, tol: f64) {
        unsafe {
            let err = GRBsetdblparam(env, CString::new("FeasibilityTol").unwrap().as_ptr(), tol);
            let err = GRBsetdblparam(env, CString::new("IntFeasTol").unwrap().as_ptr(), tol);
        }
    }

    pub fn set_feasible_strategies(&self, 
                                feasible_leader: &Vec<f64>,
                                feasible_follower: &Vec<f64>) {
        // Add follower and leader sequence form strategies.
        let treeplex = self.problem.game.treeplex(Player::Player1);
        for seq_id in 0..treeplex.num_sequences() {
            let variable_index = self.get_seq_form_index_pl1(seq_id);
            unsafe {
                let err = GRBsetdblattrelement(self.model, 
                                     CString::new("Start").unwrap().as_ptr(), 
                                     variable_index as i32, 
                                     feasible_leader[seq_id]);       
                assert_eq!(err, 0);
            }
        }

        let treeplex = self.problem.game.treeplex(Player::Player2);
        for seq_id in 0..treeplex.num_sequences() {
            let variable_index = self.get_seq_form_index_pl2(seq_id);
            unsafe {
                let err = GRBsetdblattrelement(self.model, 
                                     CString::new("Start").unwrap().as_ptr(), 
                                     variable_index as i32, 
                                     feasible_follower[seq_id]);       
                assert_eq!(err, 0);
            }
        }

    }

    pub fn set_feasible_blueprint(&self, blueprint: &BlueprintBr) {
        // Probability of reaching leaves: [0,..., |L|) --- Technically we only need leaf variables within the (follower's) trunk.
        // Number of slack variables for follower: [|L|,...|L|+|S2|) --- Technically we do not need one for the empty sequence, but w/e.
        // Value of information sets for follower: [|L|+|S2|,...|L|+|S2|+|I2|)
        // Sequence form representation of leader: [|L|+|S2|+|I2|,...|L|+|S1|+|S2|+|I2|)
        // Sequence form representation of the follower: [|L|+|S1|+|S2|+|I2|,...|L|+|S1|+2|S2|+|I2|)
        
        for leaf_index in 0..self.problem.game.payoff_matrix().entries.len() {
            let leaf = self.problem.game.payoff_matrix().entries[leaf_index];
            let variable_index = self.get_leaf_prob_index(leaf_index);
            let prob = blueprint.leader_blueprint().inner()[leaf.seq_pl1] * blueprint.follower_sequence().inner()[leaf.seq_pl2];
            unsafe {
                GRBsetdblattrelement(self.model, 
                                     CString::new("Start").unwrap().as_ptr(), 
                                     variable_index as i32, 
                                     prob);       
            }
        }

        // Compute follower values upwards.
        let gradient_follower = self.problem.game.gradient(Player::Player2, blueprint.leader_blueprint());
        let treeplex = self.problem.game.treeplex(Player::Player2);
        let mut sequence_values = gradient_follower.clone();
        let mut infoset_values = std::vec::from_elem::<f64>(-std::f64::INFINITY, treeplex.num_infosets());
        for infoset_id in 0..treeplex.num_infosets() {
            let infoset = self.problem.game.treeplex(Player::Player2).infosets()[infoset_id];
            for seq_id in infoset.start_sequence..=infoset.end_sequence {
                infoset_values[infoset_id] = infoset_values[infoset_id].max(sequence_values[seq_id]);
            }
            sequence_values[infoset.parent_sequence] += infoset_values[infoset_id];

            // Add value of slack variable.
            for seq_id in infoset.start_sequence..=infoset.end_sequence {
                let slack_value = infoset_values[infoset_id] - sequence_values[seq_id];
                let variable_index = self.get_value_slack_index(seq_id);
                unsafe {
                    GRBsetdblattrelement(self.model, 
                                        CString::new("Start").unwrap().as_ptr(), 
                                        variable_index as i32, 
                                        slack_value);       
                }
            }
        }

        // Add follower infoset values.
        for infoset_id in 0..treeplex.num_infosets() {
            let variable_index = self.get_value_infoset_index(infoset_id);
            unsafe {
                GRBsetdblattrelement(self.model, 
                                     CString::new("Start").unwrap().as_ptr(), 
                                     variable_index as i32, 
                                     infoset_values[infoset_id]);       
            }
        }

        // Add follower and leader sequence form strategies.
        let treeplex = self.problem.game.treeplex(Player::Player1);
        for seq_id in 0..treeplex.num_sequences() {
            let variable_index = self.get_seq_form_index_pl1(seq_id);
            unsafe {
                GRBsetdblattrelement(self.model, 
                                     CString::new("Start").unwrap().as_ptr(), 
                                     variable_index as i32, 
                                     blueprint.leader_blueprint().inner()[seq_id]);       
            }
        }

        let treeplex = self.problem.game.treeplex(Player::Player2);
        for seq_id in 0..treeplex.num_sequences() {
            let variable_index = self.get_seq_form_index_pl2(seq_id);
            unsafe {
                GRBsetdblattrelement(self.model, 
                                     CString::new("Start").unwrap().as_ptr(), 
                                     variable_index as i32, 
                                     blueprint.follower_sequence().inner()[seq_id]);       
            }
        }
    }
}


impl<'a> Solver<'a> for GurobiSolver<'a> {
    fn new(problem: &'a BoundedProblem, solver_config: &SolverConfig) -> GurobiSolver<'a> {
        unsafe {
            let mut envP: *mut GRBenv = std::ptr::null_mut();
            println!("Setting time limit");
            println!("{:?}", solver_config.time_limit);
            GRBloadenv(&mut envP, CString::new("TestLog").unwrap().as_ptr());
            Self::set_time_limit(envP, solver_config.time_limit);
            Self::set_feas_tol(envP, 1e-8f64);

            let mut modelP: *mut GRBmodel = std::ptr::null_mut();
            GRBnewmodel(
                envP,
                &mut modelP,
                CString::new("Skinny-sse-model").unwrap().as_ptr(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );

            let gurobi_solver = GurobiSolver {
                problem,
                env: envP,
                model: modelP,
            };

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
            gurobi_solver.make_variables();

            println!("Making slack constraints");
            gurobi_solver.make_slack_constraints();

            println!("Making sequence form constraints");
            gurobi_solver.make_sequence_form_constraints(Player::Player1);
            gurobi_solver.make_sequence_form_constraints(Player::Player2);

            println!("Making on-off constraints");
            gurobi_solver.make_on_off_constraints();

            println!("Making leaf max prob constraints");
            gurobi_solver.make_leaf_max_prob_constraints();

            println!("Making leaf sum prob constraints");
            gurobi_solver.make_leaf_sum_prob_constraints();

            println!("Making bound constraints");
            gurobi_solver.make_bounds_constraints();

            println!("Setting model sense");
            gurobi_solver.set_model_sense();

            gurobi_solver
        }
    }

    fn solve(&self) {
        unsafe {
            let err = GRBoptimize(self.model);
            assert_eq!(err, 0);
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

impl <'a> Drop for GurobiSolver<'a> {
    // Free model and environment. If omitted, Gurobi will complain with
    // error 10009 (License not found).
    fn drop(&mut self) {
        unsafe {
            GRBfreemodel(self.model);
            GRBfreeenv(self.env);
        }
    }

}