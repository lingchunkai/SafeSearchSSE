mod cbc_solver;
mod gurobi_solver;
mod mip_solution;
mod solver;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

use std::io::BufReader;

use efg_lite::game::{ExtensiveFormGame, Player};
use efg_lite::schema::game_capnp;
use efg_lite::schema::vector_capnp;
use efg_lite::sse::{BlueprintBr, GameBuilder};
use efg_lite::strategy::{BehavioralStrategy, SequenceFormStrategy};

use efg_lite::treeplex::TreeplexTools;
use efg_lite::vector::TreeplexVector;
use log::{debug, error, info, warn};

use solver::SolverConfig;

use crate::cbc_solver::CbcSolver;
use crate::gurobi_solver::GurobiSolver;
use crate::solver::Solver;

use std::str::FromStr;

#[macro_use]
extern crate approx;

#[derive(StructOpt, Debug)]
#[structopt(name = "SubgameSolver")]
struct Opt {
    // Input game file
    #[structopt(short = "g", long = "game_file")]
    game_file: PathBuf,

    // Input blueprint file, none if not provided.
    #[structopt(short = "b", long = "blueprint_file")]
    blueprint_file: BlueprintType,

    // Subgame index
    // #[structopt(short = "s", long = "subgame_index")]
    // subgame_index: usize,

    // Splitting ratio
    #[structopt(short = "r", long = "splitting_ratio", default_value = "0.5")]
    splitting_ratio: f64,

    // Gift factor
    #[structopt(short = "f", long = "gift_factor", default_value = "1.0")]
    gift_factor: f64,

    // Time limit *per subgame*
    #[structopt(short = "t", long = "time_limit_per_subgame")]
    time_limit: f64,
}

#[derive(Debug, Clone)]
enum BlueprintType {
    File(PathBuf),
    Uniform,
}

impl FromStr for BlueprintType {
    type Err = String;
    fn from_str(bp_type: &str) -> Result<Self, Self::Err> {
        match bp_type {
            "u" | "U" | "uniform" | "Uniform" => Ok(BlueprintType::Uniform),
            f => {
                let pathbuf = PathBuf::from(f);
                Ok(BlueprintType::File(pathbuf))
            }
        }
    }
}

fn solve_all_subgames<'a, 'b>(
    game: &'a ExtensiveFormGame,
    leader_blueprint: &'a SequenceFormStrategy<'a>,
    opt: &'b Opt,
) -> SequenceFormStrategy<'a> {
    let blueprint_br = BlueprintBr::new(game, &leader_blueprint);

    let follower_treeplex_tools = TreeplexTools::new(game.treeplex(Player::Player2));
    let leader_treeplex_tools = TreeplexTools::new(game.treeplex(Player::Player1));

    let game_builder = GameBuilder::new(
        game,
        &follower_treeplex_tools,
        &leader_treeplex_tools,
        &blueprint_br,
        opt.splitting_ratio,
        opt.gift_factor,
    );

    let solver_config = SolverConfig {
        time_limit: opt.time_limit,
    };

    let mut leader_full_strategy = leader_blueprint.inner().clone();

    let mut written_to = std::vec::from_elem(false, game.treeplex(Player::Player1).num_sequences());

    debug!("Preprocessed with blueprint");

    let mut objective_values = vec![];

    info!("Number of subgames{:?}", game.num_subgames());

    for subgame_id in 0..game.num_subgames() {
        debug!("Solving subgame {:?}", subgame_id);
        let (bounded_problem, game_mapper, feasible_leader, feasible_follower) =
            game_builder.bounded_problem(subgame_id);

        // debug!("Mapper --- {:?}", game_mapper);
        // debug!("Game --- {:?}", bounded_problem.game);
        debug!("Solving");
        // let solver = CbcSolver::new(&bounded_problem, &solver_config);
        let solver = GurobiSolver::new(&bounded_problem, &solver_config);

        // Map feasible solution for BP to skinny treeplex.


        // solver.set_feasible_blueprint(&blueprint_br);
        solver.set_feasible_strategies(&feasible_leader, &feasible_follower);

        debug!("Feasible leader: {:?}", feasible_leader);
        debug!("Feasible follower: {:?}", feasible_follower);

        solver.solve();

        debug!("Solved!");
        let sol = solver.get_solution();
        let strategy_pl1 = sol.leader_strategy.clone();
        let strategy_pl2 = sol.follower_strategy.clone();
        let leaf_probs = sol.leaf_probabilities.clone();
        let slack = sol.follower_slack.clone();
        let follower_value = sol.follower_value.clone();

        debug!("Leader skinny strategy {:?}", strategy_pl1);

        // Map strategy into full game
        let behavior_strategy_pl1 = BehavioralStrategy::from_sequence_form_strategy(strategy_pl1);

        debug!(
            "Behavioral strategy from sequence form {:?}",
            behavior_strategy_pl1
        );
        let skinny_treeplex_leader = bounded_problem.game.treeplex(Player::Player1);

        for skinny_infoset_id in (0..skinny_treeplex_leader.num_infosets()).rev() {
            let skinny_infoset = skinny_treeplex_leader.infosets()[skinny_infoset_id];
            let original_infoset_id = game_mapper
                .mapper_leader
                .skinny_infoset_to_infoset(skinny_infoset_id);
            let original_parent_sequence =
                game.treeplex(Player::Player1).infosets()[original_infoset_id].parent_sequence;
            let parent_prob = leader_full_strategy[original_parent_sequence];

            for skinny_sequence_id in skinny_infoset.start_sequence..=skinny_infoset.end_sequence {
                let original_child_sequence = game_mapper
                    .mapper_leader
                    .skinny_seq_to_seq(skinny_sequence_id);
                assert!(written_to[original_child_sequence] == false);
                leader_full_strategy[original_child_sequence] =
                    behavior_strategy_pl1.inner()[skinny_sequence_id] * parent_prob;
                written_to[original_child_sequence] = true;
            }
        }

        objective_values.push(sol.objective_value);
    }
    println!(
        "Payoff BP leader --- {:?}",
        game.evaluate_payoffs(
            &leader_blueprint,
            blueprint_br.follower_sequence(),
            Player::Player1
        )
    );
    println!(
        "Payoff BP follower --- {:?}",
        game.evaluate_payoffs(
            &leader_blueprint,
            blueprint_br.follower_sequence(),
            Player::Player2
        )
    );

    debug!("Leader full strategy --- {:?}", leader_full_strategy);

    debug!(
        "Objective value list for each subgame --- {:?}",
        objective_values
    );

    println!("Saving follower blueprint best response");
    let mut file_writer = File::create("bp-follower-strategy.vec").unwrap();
    blueprint_br
        .follower_sequence()
        .inner()
        .persist(&mut file_writer)
        .unwrap();

    println!("Saving follower blueprint value");
    let mut file_writer = File::create("bp-follower-value.vec").unwrap();
    blueprint_br
        .follower_seq_values()
        .persist(&mut file_writer)
        .unwrap();

    SequenceFormStrategy::from_treeplex_vector(leader_full_strategy)

}

/*
fn solve_subgame(game_builder: &GameBuilder, subgame_id: usize) {
    debug!("subgame id: {:?}", subgame_id);
    let (bounded_problem, game_mapper) = game_builder.bounded_problem(subgame_id);

    debug!("Game mapper {:?}", game_mapper);

    let solver = cbc_solver::CbcSolver::new(&bounded_problem);

    debug!("{:?}", bounded_problem);

    solver.solve();

    let sol = solver.get_solution();
    let strategy_pl1 = sol.leader_strategy.clone();
    let strategy_pl2 = sol.follower_strategy.clone();
    let leaf_probs = sol.leaf_probabilities.clone();
    let slack = sol.follower_slack.clone();
    let follower_value = sol.follower_value.clone();
    debug!("Strategy Pl1----{:?}", strategy_pl1);
    debug!("Strategy Pl2----{:?}", strategy_pl2);
    debug!("Leaf probabilities----{:?}", leaf_probs);
    debug!(
        "Utilities {:?}",
        bounded_problem.game.payoff_matrix().entries
    );
    debug!("Follower sequence slacks {:?}", slack);
    debug!("Follower value {:?}", follower_value);
}
*/

fn leader_blueprint<'a>(
    blueprint_type: &BlueprintType,
    game: &'a ExtensiveFormGame,
) -> SequenceFormStrategy<'a> {
    match blueprint_type {
        BlueprintType::Uniform => {
            // Assume leader's blueprint strategy is uniform by default.
            warn!("No blueprint file specified. Using uniform strategy as blueprint.");
            SequenceFormStrategy::from_uniform_strategy(game.treeplex(Player::Player1))
        }
        BlueprintType::File(path) => {
            let blueprint_file = File::open(path).unwrap();
            let mut blueprint_file_reader = BufReader::new(blueprint_file);
            let message_reader = capnp::serialize::read_message(
                &mut blueprint_file_reader,
                capnp::message::ReaderOptions {
                    traversal_limit_in_words: 8 * 1024 * 1024 * 1024,
                    nesting_limit: 64,
                },
            )
            .unwrap();

            let blueprint_reader = message_reader
                .get_root::<vector_capnp::vector::Reader>()
                .unwrap();
            let vector =
                TreeplexVector::deserialize(&blueprint_reader, game.treeplex(Player::Player1))
                    .unwrap();
            SequenceFormStrategy::from_treeplex_vector(vector)
        }
    }
}

fn main() {
    env_logger::init();

    let opt = Opt::from_args();

    let game_file = File::open(&opt.game_file).unwrap();
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
            let leader_blueprint = leader_blueprint(&opt.blueprint_file, &game);
            let leader_strategy = solve_all_subgames(&game, &leader_blueprint, &opt);
            info!("Subgames all solved");

            let br = BlueprintBr::new(game, &leader_strategy);

            debug!("Leader strategy --- {:?}", leader_strategy);
            debug!("Follower strategy --- {:?}", br.follower_sequence());

            println!(
                "Payoff SS leader --- {:?}",
                game.evaluate_payoffs(&leader_strategy, br.follower_sequence(), Player::Player1)
            );
            println!(
                "Payoff SS follower --- {:?}",
                game.evaluate_payoffs(&leader_strategy, br.follower_sequence(), Player::Player2)
            );

            println!("Saving refined leader strategy");
            let mut file_writer = File::create("full-leader-strategy.vec").unwrap();
            leader_strategy.inner().persist(&mut file_writer).unwrap();

            println!("Saving refined follower strategy");
            let mut file_writer = File::create("full-follower-strategy.vec").unwrap();
            br.follower_sequence()
                .inner()
                .persist(&mut file_writer)
                .unwrap();

            println!("Saving refined leader br-values");
            let mut file_writer = File::create("full-leader-br-values.vec").unwrap();
            br.leader_seq_values().persist(&mut file_writer).unwrap();

            println!("Saving refined follower br-values");
            let mut file_writer = File::create("full-follower-br-values.vec").unwrap();
            br.follower_seq_values().persist(&mut file_writer).unwrap();

        }
        Err(err) => {
            println!("{}", err);
        }
    }
}
