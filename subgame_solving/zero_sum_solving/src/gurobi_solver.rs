
use crate::solver::{Solver, SolverConfig};
use crate::zero_sum_solution::ZeroSumSolution;
use efg_lite::game::{EFGTools, ExtensiveFormGame, Player};
use efg_lite::strategy::SequenceFormStrategy;
use efg_lite::treeplex::{SequenceId, TreeplexTools};
use efg_lite::vector::TreeplexVector;

use std::ffi::{CString, CStr};

use optimizers::gurobi_bindings::bindings::*;

/// ===============================================================================================
/// We use the dualized formulation in
/// http://www.cs.cmu.edu/~ggordon/poker/source/zsumextformlp.m
///
/// Specifically,
///
///  Ax = a,  x >= 0, A, a are sequence form constraints for Player 1 and empty sequence identifier.
///  By = b,  y >= 0, B, b are sequence form constraints for Player 2 and empty sequence identifier.
///  x = arg max y' * R * x,
///  y = arg min y' * R * x,
///  where R is the reward matrix for Player 1 (who chooses x)
///
/// Or,
/// min_y max_x x' R y
/// Ax = a, x >= 0
/// By = b, y >= 0
///
/// which after taking the dual in the middle maximum and performing some sign changes, gives
/// min_{y,z} a' z
/// A'z - Ry >= 0
/// By = b, y >= 0
///
/// Or, if we swap min/max's and play around with signs, we get what is given by Geoff's formulation
/// min_{x,z} b'z
/// Rx + B'z >= 0  ---------------(A)
/// Ax = a, ----------------------(B)
/// x >= 0,
///
/// We will use Geoff's formulation for simplicity.
/// ===============================================================================================
/// The number of variables (cols) are
///
/// (|I2| + 1) + |S1|, where
/// |S1| = num_sequences (pl2) and
/// |I2| = num_infosets (pl1).
///
/// The number of constraints (rows) are
/// |S2| + (|I1| + 1)
///
/// where
/// |S1| = num_sequences (Pl1),
/// |S2| = num_sequences (Pl2),
/// |I1| = num_infosets (Pl1),
/// |I2| = num_infosets (Pl2).
///
/// The additional 1 variable in the constraints involving |I| are due tothe (primal) constraint associated
/// with the empy sequence.
/// ===============================================================================================
/// The numbering we will use for variables is
///
/// Sequence form representation of pl1 (x): [0,..., |S1|)
/// Dual variables for infosets of pl2 (z): [|S1|,... |S1|+|I2|)
/// Dual variables for empty sequence of pl2 (z): |S1|+|I2|
/// ================================================================================================
pub struct GurobiSolver<'a> {
    game: &'a ExtensiveFormGame,
    env: *mut GRBenv, // TODO: make static, so we don't have to reconstruct each time.
    model: *mut GRBmodel, // Gurobi model.
}

impl<'a> GurobiSolver<'a> {
    fn get_seq_form_index_pl1(&self, sequence_id: SequenceId) -> usize {
        sequence_id
    }

    fn get_infoset_index_pl2(&self, infoset_id: usize) -> usize {
        infoset_id + self.game.treeplex(Player::Player1).num_sequences()
    }

    fn get_empty_seq_constr_index_pl2(&self) -> usize {
        self.game.treeplex(Player::Player1).num_sequences()
            + self.game.treeplex(Player::Player2).num_infosets()
    }

    fn get_solution_strategy_pl1(&self) -> SequenceFormStrategy<'a> {
        let treeplex = self.game.treeplex(Player::Player1);

        let mut dst = Vec::<f64>::new();
        unsafe {
            dst.resize(treeplex.num_sequences(), 0f64);
            GRBgetdblattrarray(
                self.model,
                CString::new("X").unwrap().as_ptr(),
                self.get_seq_form_index_pl1(0) as i32,
                treeplex.num_sequences() as i32,
                dst.as_mut_ptr(),
            );
        }

        let treeplex_vector = TreeplexVector::from_vec(treeplex, dst);
        SequenceFormStrategy::from_treeplex_vector(treeplex_vector)
    }

    fn get_solution_value(&self) -> f64 {
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

    /// Constraints (B)
    fn make_sequence_form_constraints_pl1(&self) {
        let treeplex = self.game.treeplex(Player::Player1);
        let empty_sequence_id = treeplex.empty_sequence_id();

        // Get cols and coeffs for empty sequence constraint.
        let mut col_indices: Vec<i32> = vec![self.get_seq_form_index_pl1(empty_sequence_id) as i32];
        let mut col_coeffs: Vec<f64> = vec![1.0];
        unsafe {
            GRBaddconstr(
                self.model,
                1,
                col_indices.as_mut_ptr(),
                col_coeffs.as_mut_ptr(),
                GRB_EQUAL as i8,
                1f64,
                CString::new(format!("seq_form_constraints_empty_seq_pl1"))
                    .unwrap()
                    .as_ptr(),
            );
        }

        for (infoset_id, infoset) in treeplex.infosets().iter().enumerate() {
            // Get cols and coeffs for non-empty sequence constraint.
            let mut col_indices: Vec<i32> =
                vec![self.get_seq_form_index_pl1(infoset.parent_sequence) as i32];
            let mut col_coeffs: Vec<f64> = vec![1.0];

            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                col_indices.push(self.get_seq_form_index_pl1(sequence_id) as i32);
                col_coeffs.push(-1.0);
            }

            let nz = col_indices.len() as i32;

            assert_eq!(col_indices.len(), col_coeffs.len());
            assert_eq!(col_indices.len(), nz as usize);
            unsafe {
                GRBaddconstr(
                    self.model,
                    nz,
                    col_indices.as_mut_ptr(),
                    col_coeffs.as_mut_ptr(),
                    GRB_EQUAL as i8,
                    0f64,
                    CString::new(format!("seq_form_constraints_{}_pl1", infoset_id))
                        .unwrap()
                        .as_ptr(),
                );
            }
        }
    }

    /// Constraints (A)
    fn make_constraints_per_sequence(&self) {
        let mut status : i32 = 0;

        let efg_tools = EFGTools::new(self.game);

        let treeplex = self.game.treeplex(Player::Player2);
        let treeplex_tools = TreeplexTools::new(treeplex);

        for seq_id in 0..treeplex.num_sequences() {
            let mut col_indices = Vec::<i32>::new();
            let mut col_coeffs = Vec::<f64>::new();

            // Handle the part of the constraint dealing with Rx.
            let leaf_indices = efg_tools.leaf_indices_at_sequence(Player::Player2, seq_id);
            for leaf_index in leaf_indices {
                let leaf = self.game.payoff_matrix().entries[leaf_index];
                let seq_pl1 = leaf.seq_pl1;
                let seq_pl1_var_index = self.get_seq_form_index_pl1(seq_pl1);
                let payoff = leaf.payoff_pl1 * leaf.chance_factor;

                col_indices.push(seq_pl1_var_index as i32);
                // println!("{:?}", col_indices.last());
                col_coeffs.push(payoff);
            }

            // Handle the part of the constraint dealing with B'z.
            // We will need to get (I) the parent infoset which contained this
            // sequence and (II) all direct children infosets from this sequence.

            // (I)
            // If the parent infoset of the sequence does not exist (i.e., seq_id is the empty sequence),
            // then the variable that we use
            // If the parent infoset exists, then we us it.
            match seq_id == treeplex.empty_sequence_id() {
                true => {
                    col_indices.push(self.get_empty_seq_constr_index_pl2() as i32);
                    col_coeffs.push(1.0);
                }
                false => {
                    let parent_infoset = treeplex_tools.parent_infoset_of_seq(seq_id).unwrap();
                    let parent_infoset_var_index = self.get_infoset_index_pl2(parent_infoset);
                    col_indices.push(parent_infoset_var_index as i32);
                    col_coeffs.push(1.0);
                }
            }

            // (II)
            for child_infoset in treeplex_tools.seq_to_infoset_range(seq_id) {
                let child_infoset_var_index = self.get_infoset_index_pl2(child_infoset);
                col_indices.push(child_infoset_var_index as i32);
                col_coeffs.push(-1.0);
            }

            unsafe {
                status = GRBaddconstr(
                    self.model,
                    col_indices.len() as i32,
                    col_indices.as_mut_ptr(),
                    col_coeffs.as_mut_ptr(),
                    GRB_GREATER_EQUAL as i8,
                    0f64,
                    CString::new(format!("best_response_constraints_player2_{}", seq_id))
                        .unwrap()
                        .as_ptr(),
                );
                /*
                if status != 0 {
                    println!("{:?}", seq_id);
                    println!("{:?}", col_indices);
                    println!("{:?}", col_coeffs);
                    let mut W = col_indices.clone();
                    W.sort();
                    println!("{:?}", W);
                    println!("{:?}", CStr::from_ptr(GRBgeterrormsg(self.env)));
                }
                */
                assert_eq!(status, 0);
            }
        }
    }

    fn make_variables(&self) {
        let mut status : i32 = 0;

        // Sequence form representation pl1.
        for leader_sequence in 0..self.game.treeplex(Player::Player1).num_sequences() {
            unsafe {
                status = GRBaddvar(
                    self.model,
                    0,                    // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    0f64,                 // No objective.
                    0f64,                 // Sequence probability must be >= 0
                    1f64, // Sequence probability may not be more than 1 (not really required in theory).
                    'C' as i8, // GRB_CONTINUOUS,
                    CString::new(format!("pl1_sequence_form_{}", leader_sequence))
                        .unwrap()
                        .as_ptr(),
                );
                assert_eq!(status, 0);
            }
        }

        // Infoset values pl2 (|I2| variables)
        for follower_infoset in 0..self.game.treeplex(Player::Player2).num_infosets() {
            unsafe {
                status = GRBaddvar(
                    self.model,
                    0,                    // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    0f64,                 // No objective.
                    -GRB_INFINITY,        // Values can range from -INF to INF
                    GRB_INFINITY,         // Values can range from -INF to INF
                    'C' as i8,            // GRB_CONTINUOUS,
                    CString::new(format!("pl2_infoset_value_{}", follower_infoset))
                        .unwrap()
                        .as_ptr(),
                );
                assert_eq!(status, 0);
            }
        }

        // Extra variable coressponding to dual of empty sequence constraint for player 2.
        unsafe {
            status = GRBaddvar(
                self.model,
                0,                    // Will add constraints later on.
                std::ptr::null_mut(), // Will add constraints later on.
                std::ptr::null_mut(), // Will add constraints later on.
                1f64,                 // Objective of 1.
                -GRB_INFINITY,        // Values can range from -INF to INF
                GRB_INFINITY,         // Values can range from -INF to INF
                'C' as i8,            // GRB_CONTINUOUS,
                CString::new("pl2_infoset_empty_sequence_value")
                    .unwrap()
                    .as_ptr(),
            );
            assert_eq!(status, 0);
        }
    }

    fn set_model_sense(&self) {
        unsafe {
            GRBsetintattr(
                self.model,
                CString::new("ModelSense").unwrap().as_ptr(),
                1, // Minimization problem.
            );
        }
    }

    fn set_time_limit(env: *mut GRBenv, time_limit: f64) {
        unsafe {
            let err = GRBsetdblparam(env, CString::new("TimeLimit").unwrap().as_ptr(), time_limit);
        }
    }
}

impl<'a> Solver<'a> for GurobiSolver<'a> {

    fn new(game: &'a ExtensiveFormGame, solver_config: &'a SolverConfig) -> GurobiSolver<'a> {
        unsafe {
            let mut envP: *mut GRBenv = std::ptr::null_mut();
            println!("Setting time limit");
            println!("{:?}", solver_config.time_limit);
            GRBloadenv(&mut envP, CString::new("zero-sum-log").unwrap().as_ptr());
            Self::set_time_limit(envP, solver_config.time_limit);

            let mut modelP: *mut GRBmodel = std::ptr::null_mut();
            GRBnewmodel(
                envP,
                &mut modelP,
                CString::new("zero-sum-model").unwrap().as_ptr(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );

            let gurobi_solver = GurobiSolver {
                game,
                env: envP,
                model: modelP,
            };

            println!("Making variables");
            gurobi_solver.make_variables();

            println!("Making constraints-per-sequence (A)");
            gurobi_solver.make_constraints_per_sequence();

            println!("Make sequence form constraints (B)");
            gurobi_solver.make_sequence_form_constraints_pl1();

            println!("Setting model sense");
            gurobi_solver.set_model_sense();

            gurobi_solver
        }
    }

    fn solve(&self) {
        unsafe {
            GRBoptimize(self.model);
        }
    }

    fn get_solution(&self) -> ZeroSumSolution<'a> {
        ZeroSumSolution::new(self.get_solution_strategy_pl1(), self.get_solution_value())
    }
}