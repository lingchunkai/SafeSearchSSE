mod gurobi_solver;
mod solver;
mod zero_sum_solution;

use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

use std::io::BufReader;

use efg_lite::game::{ExtensiveFormGame, Player};
use efg_lite::schema::game_capnp;
use efg_lite::strategy::{BehavioralStrategy, SequenceFormStrategy};

use log::{debug, error, info, warn};

use solver::SolverConfig;

// use crate::cbc_solver::CbcSolver; // TODO (chunkail)
use crate::gurobi_solver::GurobiSolver;
use crate::solver::Solver;

#[derive(StructOpt, Debug)]
#[structopt(name = "ZeroSumSolver")]
struct Opt {
    // Input game file
    #[structopt(short = "g", long = "input_game_file")]
    input_file: PathBuf,

    // Time limit
    #[structopt(short = "t", long = "time_limit")]
    time_limit: f64,
}

fn main() {
    env_logger::init();

    let opt = Opt::from_args();

    let game_file = File::open(&opt.input_file).unwrap();
    let mut game_file_reader = BufReader::new(game_file);
    let message_reader = capnp::serialize::read_message(
        &mut game_file_reader,
        capnp::message::ReaderOptions {
            traversal_limit_in_words: 8 * 1024 * 1024 * 1024,
            nesting_limit: 64,
        },
    )
    .unwrap();

    let game_reader = message_reader
        .get_root::<game_capnp::game::Reader>()
        .unwrap();

    match ExtensiveFormGame::deserialize(&game_reader) {
        Ok(ref game) => {

            if !game.is_zero_sum() {
                warn!("Game is not zero-sum. Will convert using Player 1 as template");
            }

            let game = game.zero_sum(Player::Player1);
            println!("Num infosets: {:?}, {:?}", game.treeplex(Player::Player1).num_infosets(), game.treeplex(Player::Player2).num_infosets());
            println!("Num sequences: {:?}, {:?}", game.treeplex(Player::Player1).num_sequences(), game.treeplex(Player::Player2).num_sequences());
            // println!("{:?}", game.treeplex(Player::Player1).infosets()[5999]);

            let solver_config = SolverConfig {
                time_limit: opt.time_limit,
            };

            let solver = GurobiSolver::new(&game, &solver_config);
            solver.solve();

            let sol = solver.get_solution();
            let sol_strategy_p1 = sol.strategy_pl1;
            let sol_game_value = sol.game_value;

            // println!("pl1 strategy: {:?}", sol_strategy_p1);
            println!("game value: {:?}", sol_game_value);

            println!("Saving zero-sum pl1 strategy");
            let mut file_writer = File::create("pl1-zero-sum-strategy.vec").unwrap();
            sol_strategy_p1.inner().persist(&mut file_writer).unwrap();
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}

