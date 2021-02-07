// Here we try to solve for individual main/subgames in order to piece together blueprints.
// Note these are all normal form games.

// We use the standard multiple LP method.

// Follower is the column player, 
// Their actions are indexed by the SECOND, i.e., size of the inner array.


#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/gurobi_bindings.rs"));

use std::ffi::{CString, CStr};
use noisy_float::prelude::*;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct MatGameSolution {
    pub leader_payoff: f64,
    pub follower_payoff: f64,

    pub leader_strategy: Vec::<f64>,
    pub follower_br: usize,
}

pub struct GurobiSolver<'a> {
    payoffs_pl1: &'a Vec::<Vec<f64>>,
    payoffs_pl2: &'a Vec::<Vec<f64>>,
}

pub struct SingleLPSolver<'a> {
    env: *mut GRBenv,
    model: *mut GRBmodel,

    best_follower_pure_strategy: usize,

    payoffs_pl1: &'a Vec::<Vec<f64>>,
    payoffs_pl2: &'a Vec::<Vec<f64>>,
}

impl<'a> GurobiSolver<'a> {
    fn payoff_size_check(&self) {
        assert!(self.payoffs_pl1.len() > 0);
        assert!(self.payoffs_pl2.len() > 0);
        assert_eq!(self.payoffs_pl1.len(), self.payoffs_pl2.len());
        assert_eq!(self.payoffs_pl1[0].len(), self.payoffs_pl2[0].len());
        
        // Ensure payoffs are indeed a retangular matrix.
        assert_eq!(self.payoffs_pl1.iter().all(|x| x.len() == self.payoffs_pl1[0].len()), true);
        assert_eq!(self.payoffs_pl2.iter().all(|x| x.len() == self.payoffs_pl2[0].len()), true);
    }

    pub fn new(payoffs_pl1: &'a Vec::<Vec<f64>>, 
           payoffs_pl2: &'a Vec::<Vec<f64>>) -> GurobiSolver<'a> {
        let solver = GurobiSolver {
            payoffs_pl1,
            payoffs_pl2,
        };
        solver.payoff_size_check();
        solver
    }

    pub fn solve(&self) -> MatGameSolution {
        let num_follower_actions = self.payoffs_pl1.last().unwrap().len();
        let mut solutions = Vec::<MatGameSolution>::new();
        
        for best_follower_pure_strategy in 0..num_follower_actions {
            let single_LP_solver = SingleLPSolver::new(
                self.payoffs_pl1,
                self.payoffs_pl2,
                best_follower_pure_strategy,
            );
            solutions.push(single_LP_solver.solve().clone());
        }

        // println!("{:?}", solutions);


        // Now iterate over all best actions and pick the best.
        let sol = (*solutions.iter().max_by_key(|x| r64(x.leader_payoff)).unwrap()).clone();

        println!("Payoffs pl1 {:?}", self.payoffs_pl1);
        println!("Payoffs pl2 {:?}", self.payoffs_pl2);

        // TODO: test if truely SSE
        let bleh0 = 
            get_player_payoffs(self.payoffs_pl1, self.payoffs_pl2, 
                               0, 
                               &sol.leader_strategy);
        let bleh1 = 
            get_player_payoffs(self.payoffs_pl1, self.payoffs_pl2, 
                               1, 
                               &sol.leader_strategy);

        println!("0 {:?}", bleh0);
        println!("1 {:?}", bleh1);
        
        sol
    }
}

fn get_player_payoffs(
                      payoffs_pl1: &Vec::<Vec::<f64>>,
                      payoffs_pl2: &Vec::<Vec::<f64>>,
                      follower_pure_strategy: usize, 
                      leader_strategy: &Vec::<f64>) -> (f64, f64) {
    
    let num_follower_actions = payoffs_pl1.last().unwrap().len();
    let num_leader_actions = payoffs_pl1.len();

    // TODO: make idiomatic using rust iterators.
    let mut accum_payoff_pl1 = 0f64;
    let mut accum_payoff_pl2 = 0f64;

    for leader_action_id in 0..num_leader_actions {
        let prob = leader_strategy[leader_action_id];
        accum_payoff_pl1 += prob * payoffs_pl1[leader_action_id][follower_pure_strategy];
        accum_payoff_pl2 += prob * payoffs_pl2[leader_action_id][follower_pure_strategy];
    }

    (accum_payoff_pl1, accum_payoff_pl2)
}

impl<'a> SingleLPSolver<'a> {

    pub fn new(
            payoffs_pl1: &'a Vec::<Vec<f64>>,
            payoffs_pl2: &'a std::vec::Vec<std::vec::Vec<f64>>, 
            best_follower_pure_strategy: usize) -> SingleLPSolver<'a> {
        
        unsafe {
            let mut envP: *mut GRBenv = std::ptr::null_mut();
            GRBloadenv(&mut envP, CString::new("zero-sum-log").unwrap().as_ptr());

            let mut modelP: *mut GRBmodel = std::ptr::null_mut();
            GRBnewmodel(
                envP,
                &mut modelP,
                CString::new(format!("Follower BR: {}", best_follower_pure_strategy)).unwrap().as_ptr(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );

            GRBsetintattr(
                modelP,
                CString::new("ModelSense").unwrap().as_ptr(),
                -1, // Maximization problem.
            );

            println!("Making variables");
            Self::make_variables(modelP, payoffs_pl1, payoffs_pl2, best_follower_pure_strategy);
            
            println!("Making constraints");
            Self::make_constraints(modelP, payoffs_pl1, payoffs_pl2, best_follower_pure_strategy);

            SingleLPSolver {
                env: envP,
                model: modelP,
                best_follower_pure_strategy: best_follower_pure_strategy,
                payoffs_pl1,
                payoffs_pl2,
            }
        }
        
    }

    fn make_variables(model: *mut GRBmodel, 
                    payoffs_pl1: &Vec::<Vec::<f64>>,
                    payoffs_pl2: &Vec::<Vec::<f64>>,
                    best_follower_pure_strategy: usize) {
        let num_follower_actions = payoffs_pl1.last().unwrap().len();
        let num_leader_actions = payoffs_pl1.len();

        let mut status:i32 = 0;
        
        for leader_strategy in 0..num_leader_actions {
            unsafe {
                status = GRBaddvar(
                    model,
                    0,                    // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    std::ptr::null_mut(), // Will add constraints later on.
                    payoffs_pl1[leader_strategy][best_follower_pure_strategy], // Leader payoffs for their action.
                    0f64,                 // Sequence probability must be >= 0
                    1f64, // Sequence probability may not be more than 1 (not really required in theory).
                    'C' as i8, // GRB_CONTINUOUS,
                    CString::new(format!("leader strategy {}", leader_strategy))
                        .unwrap()
                        .as_ptr(),
                );
            }
            assert_eq!(status, 0);
        }
    }

    fn make_constraints(model: *mut GRBmodel, 
                    payoffs_pl1: &Vec::<Vec::<f64>>,
                    payoffs_pl2: &Vec::<Vec::<f64>>,
                    best_follower_pure_strategy: usize) {
        let num_follower_actions = payoffs_pl1.last().unwrap().len();
        let num_leader_actions = payoffs_pl1.len();

        let mut status : i32 = 0;
        
        // Best follower strategy has to be `best_follower_pure_strategy`
        for follower_strategy in 0..num_follower_actions {
            if follower_strategy == best_follower_pure_strategy {
                continue;
            }


            // Note that `col_indices` refers to variable index, not column in the game matrix
            let mut col_indices = Vec::<i32>::new();         
            let mut col_coeffs = Vec::<f64>::new();
            for leader_action_id in 0..num_leader_actions {
                let col_coeff = payoffs_pl2[leader_action_id][best_follower_pure_strategy] - 
                                payoffs_pl2[leader_action_id][follower_strategy];
                col_coeffs.push(col_coeff);
                col_indices.push(leader_action_id as i32);
            }

            unsafe{
                status = GRBaddconstr(
                    model,
                    col_indices.len() as i32,
                    col_indices.as_mut_ptr(),
                    col_coeffs.as_mut_ptr(),
                    GRB_GREATER_EQUAL as i8,
                    0f64,
                    CString::new(format!("BR constr, best follower strategy: {:?}, follower strategy {:?}", best_follower_pure_strategy, follower_strategy))
                        .unwrap()
                        .as_ptr(),
                );
                assert_eq!(status , 0);
            }
        }

        // Add sum-to-one constraints. TODO: make idiomatic.
        let mut col_indices = Vec::<i32>::new();
        let mut col_coeffs = Vec::<f64>::new();
        for leader_action_id in 0..num_leader_actions {
            col_coeffs.push(1f64);
            col_indices.push(leader_action_id as i32);
        }
        unsafe {
            status = GRBaddconstr(
                model,
                col_indices.len() as i32,
                col_indices.as_mut_ptr(),
                col_coeffs.as_mut_ptr(),
                GRB_EQUAL as i8,
                1f64,
                CString::new(format!("Sum-to-one constraint"))
                    .unwrap()
                    .as_ptr(),
            );
            assert_eq!(status , 0);
        }
    }

    pub fn solve(&self) -> MatGameSolution{
        let num_leader_actions = self.payoffs_pl1.len();

        unsafe {
            GRBoptimize(self.model);
        }

        // Extract solution, objective
        let mut dst = Vec::<f64>::new();
        dst.resize(num_leader_actions as usize, 0f64);
        unsafe {
            GRBgetdblattrarray(
                self.model,
                CString::new("X").unwrap().as_ptr(),
                0 as i32, // Order of variables starts from 0.
                num_leader_actions as i32,
                dst.as_mut_ptr(),
            );
        }
    
        let (leader_payoff, follower_payoff) = 
            get_player_payoffs(&self.payoffs_pl1, 
                               &self.payoffs_pl2, 
                               self.best_follower_pure_strategy,
                               &dst);
            

        MatGameSolution {
            leader_payoff: leader_payoff,
            follower_payoff: follower_payoff,
        
            leader_strategy: dst,
            follower_br: self.best_follower_pure_strategy,
        }       
    }
}

