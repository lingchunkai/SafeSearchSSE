/// Synthetic Markov game

extern crate efg_lite;
extern crate env_logger;
extern crate structopt;
extern crate libgt;

use efg_lite::game::{Player, ExtensiveFormGame};
use efg_lite::strategy::{BehavioralStrategy, SequenceFormStrategy};
use log::debug;

use libgt::{ChanceOrPlayer, ExtensiveFormGameBuilder, GameTreeVertex, Leaf, VertexOrLeaf};
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

mod solve_games;
use solve_games::{MatGameSolution, GurobiSolver};

use libgt::GameAnnotations;

// use std::str::FromStr;

use rand::{Rng, SeedableRng};
use rand::rngs::{StdRng};

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Config {
    pub num_subgames: usize,
    pub specify_subgames: bool, // Set to False if you don't want subgames to be specified.

    pub main_game_size: (usize, usize),
    pub subgame_size: (usize, usize),

    pub influence_of_main_action: f64,
    
    pub main_game_payoff_range: (f64, f64),
    pub subgame_payoff_range: (f64, f64),
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Synth {
    pub num_subgames: usize,
    pub specify_subgames: bool, 

    pub main_game_size: (usize, usize),
    pub subgame_size: (usize, usize),

    pub main_game_payoffs_pl1: Vec::<Vec::<f64>>,
    pub main_game_payoffs_pl2: Vec::<Vec::<f64>>,

    pub subgame_payoffs_pl1: Vec::<Vec::<Vec::<f64>>>,
    pub subgame_payoffs_pl2: Vec::<Vec::<Vec::<f64>>>,

    // transition[leader action][subgame id] contains the probability
    // transition[{.}] sums to 1.
    pub transition: Vec::<Vec::<f64>>,
}

impl Config{
    pub fn new(num_subgames: usize,
                main_game_size: (usize, usize),
                subgame_size: (usize, usize),
                influence_of_main_action: f64,
                main_game_payoff_range: (f64, f64),
                subgame_payoff_range: (f64, f64),
                specify_subgames: bool) -> 
                Config { 
                    Config {
                    specify_subgames,
                    num_subgames,
                    main_game_size,
                    subgame_size,
                    influence_of_main_action,
                    main_game_payoff_range,
                    subgame_payoff_range,
                }
            }
}

impl Synth {
    pub fn new(config: &Config, random_seed: usize) -> Synth {
        let mut rng : StdRng = SeedableRng::seed_from_u64(random_seed as u64);

        // Main game payoffs
        let mut main_game_payoffs_pl1 = 
            Vec::<Vec::<f64>>::new();
        for _i in 0..config.main_game_size.0 {
            main_game_payoffs_pl1.push(Vec::<f64>::new());
            for _j in 0..config.main_game_size.1 {
                let rand_payoff : f64 = rng.gen();
                main_game_payoffs_pl1.last_mut().unwrap().push(rand_payoff);

            }
        }

        let mut main_game_payoffs_pl2 = 
            Vec::<Vec::<f64>>::new();
        for _i in 0..config.main_game_size.0 {
            main_game_payoffs_pl2.push(Vec::<f64>::new());
            for _j in 0..config.main_game_size.1 {
                let rand_payoff : f64 = rng.gen();
                main_game_payoffs_pl2.last_mut().unwrap().push(rand_payoff);
            }
        }

        // Subgame payoffs.
        let mut subgame_payoffs_pl1 = 
            Vec::<Vec::<Vec::<f64>>>::new();
        for _game_index in 0..config.num_subgames {
            subgame_payoffs_pl1.push(Vec::<Vec::<f64>>::new());
            let subgame = subgame_payoffs_pl1.last_mut().unwrap();
            for _i in 0..config.subgame_size.0 {
                subgame.push(Vec::<f64>::new());
                for _j in 0..config.subgame_size.1 {
                    let rand_payoff : f64 = rng.gen();
                    subgame.last_mut().unwrap().push(rand_payoff);
                }
            }
        }

        let mut subgame_payoffs_pl2 = 
            Vec::<Vec::<Vec::<f64>>>::new();
        for _game_index in 0..config.num_subgames {
            subgame_payoffs_pl2.push(Vec::<Vec::<f64>>::new());
            let subgame = subgame_payoffs_pl2.last_mut().unwrap();
            for _i in 0..config.subgame_size.0 {
                subgame.push(Vec::<f64>::new());
                for _j in 0..config.subgame_size.1 {
                    let rand_payoff : f64 = rng.gen();
                    subgame.last_mut().unwrap().push(rand_payoff);
                }
            }
        }

        let mut transition = 
            Vec::<Vec<f64>>::new();
        for leader_action in 0..config.main_game_size.0 {
            transition.push(Vec::<f64>::new());
            for i in 0..config.num_subgames {
                transition.last_mut().unwrap().push(rng.gen());
            }
            let accum : f64 = transition.last().unwrap().iter().sum();
            for i in 0..config.num_subgames {
                let p: f64 = transition.last_mut().unwrap()[i] / accum;
                transition.last_mut().unwrap()[i] = 
                    p * config.influence_of_main_action + 
                    1f64 / config.num_subgames as f64 * (1f64 - config.influence_of_main_action);
            }
        }

        Synth {
            specify_subgames: config.specify_subgames,
            num_subgames: config.num_subgames,
            main_game_size: config.main_game_size,
            subgame_size: config.subgame_size,
            
            main_game_payoffs_pl1,
            main_game_payoffs_pl2,
            
            subgame_payoffs_pl1,
            subgame_payoffs_pl2,
            
            transition,
        }
    }
}

type SubgameIndex = usize;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct State<'a> {
    synth: &'a Synth,

    p1_m1: usize,
    p2_m1: usize,
    p1_m2: usize,
    p2_m2: usize,

    subgame_index: Option<SubgameIndex>,
    player_to_move: ChanceOrPlayer,

}

#[derive(Debug, Clone, Eq, Ord, PartialOrd, PartialEq)]
pub struct Subgame {
    p1_m1: usize,
    p2_m1: usize,
    subgame_index: SubgameIndex,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Action {
    NextSubgame(SubgameIndex),
    ActionIndex(usize)
}

/// Everything about the game is known the player, except the other's player current action.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlayerInfo {
    p1_m1_o: Option<usize>,
    p2_m1_o: Option<usize>,
    subgame: Option<SubgameIndex>,

    // Technically we will never use this...?
    // p1_m2_o: Option<u32>,
    // p2_m2_o: Option<u32>,
}

impl<'a> State<'a> {
    pub fn initial_state(synth: &'a Synth) -> State<'a> {
        State {
            synth, 
            p1_m1: 0,
            p1_m2: 0,
            p2_m1: 0,
            p2_m2: 0,
            subgame_index: Option::<SubgameIndex>::None,
            player_to_move: ChanceOrPlayer::Player(Player::Player1),
        }
    }

    fn get_payoffs(&self) -> (f64, f64) {
        match self.subgame_index {
            Option::<SubgameIndex>::None => panic!(),
            Option::<SubgameIndex>::Some(subgame) => {
                let payoffs_pl1 = self.synth.main_game_payoffs_pl1[self.p1_m1][self.p2_m1] + 
                                self.synth.subgame_payoffs_pl1[subgame][self.p1_m2][self.p2_m2];

                let payoffs_pl2 = self.synth.main_game_payoffs_pl2[self.p1_m1][self.p2_m1] + 
                                self.synth.subgame_payoffs_pl2[subgame][self.p1_m2][self.p2_m2];
                (payoffs_pl1, payoffs_pl2)
            }
        }
    }

    fn dummy_subgame(&self) -> Option::<Subgame> {
        let dummy = Subgame {
            p1_m1: 0,
            p2_m1: 0,
            subgame_index: 0,
        };
        Option::<Subgame>::Some(dummy)
    }
}

impl<'a> GameTreeVertex for State<'a> {

    type Action = Action;
    type PlayerInfo = PlayerInfo;
    type Subgame = Subgame;

    fn next_player(&self) -> ChanceOrPlayer{
        return self.player_to_move.clone()
    }

    fn player_information(&self) -> Self::PlayerInfo {
        match self.subgame_index {
            None => {
                // We are still in the main game.
                PlayerInfo {
                    p1_m1_o: Option::<usize>::None,
                    p2_m1_o: Option::<usize>::None,
                    subgame: Option::<SubgameIndex>::None,
                }
            },
            Some(subgame_index) => {
                // We are in some subgame. Whether or not we see 
                // the actions that the previous player did is 
                // a design decision, if we do, then the game will be
                // much larger.
                PlayerInfo {
                    p1_m1_o: Option::<usize>::Some(self.p1_m1),
                    p2_m1_o: Option::<usize>::Some(self.p2_m1),
                    subgame: Option::<SubgameIndex>::Some(subgame_index),
                }
            }
        }
    }

    fn available_actions(&self) -> Box<[(Self::Action, f64)]> {
        match self.player_to_move {
            ChanceOrPlayer::Chance => {
                // TODO: put correct chance, transition probability based on influence
                // Probability of moving 
                // let p = self.synth.transition[self.p1_m1].iter().map(|x| ())
                let z = &self.synth.transition[self.p1_m1];
                let r: Vec<(Self::Action, f64)> 
                   = (0..self.synth.num_subgames).map(|x| (Action::NextSubgame(x), z[x])).collect();
                r.into_boxed_slice()

                // let r: Vec<(Self::Action, f64)> 
                //    = (0..self.synth.num_subgames).map(|x| (Action::NextSubgame(x), p)).collect();
                // r.into_boxed_slice()
            },
            ChanceOrPlayer::Player(Player::Player1) => {
                let n: usize = {
                    match self.subgame_index {
                        None => self.synth.main_game_size.0,
                        Some(_) => self.synth.subgame_size.0,
                    }
                };
                let r: Vec<(Self::Action, f64)> = (0..n).map(|x| (Action::ActionIndex(x), 0f64)).collect();
                r.into_boxed_slice()
            },
            ChanceOrPlayer::Player(Player::Player2) => {
                let n: usize = {
                    match self.subgame_index {
                        None => self.synth.main_game_size.1,
                        Some(_) => self.synth.subgame_size.1,
                    }
                };
                let r: Vec<(Self::Action, f64)> = (0..n).map(|x| (Action::ActionIndex(x), 0f64)).collect();
                r.into_boxed_slice()
            }
        }
    }

    fn next_state(&self, action: &Self::Action) -> VertexOrLeaf<Self>{
        
        match self.subgame_index {
            // Still in main game (or waiting to transit)
            None => {
                match self.next_player() {
                    ChanceOrPlayer::Chance => {
                        match action {
                            Action::ActionIndex(_) => panic!(),
                            Action::NextSubgame(next_subgame_index) => {
                                let new_state = Self {
                                    synth: self.synth,
                                    p1_m1: self.p1_m1,
                                    p1_m2: 0,
                                    p2_m1: self.p2_m1,
                                    p2_m2: 0,
                                    subgame_index: Option::<SubgameIndex>::Some(*next_subgame_index),
                                    player_to_move: ChanceOrPlayer::Player(Player::Player1),
                                };
                                VertexOrLeaf::Vertex(new_state)
                            }
                        }
                    },
                    ChanceOrPlayer::Player(Player::Player1) => {
                        match action {
                            Action::ActionIndex(action_index) => {
                                let new_state = Self {
                                    synth: self.synth, 
                                    p1_m1: *action_index,
                                    p1_m2: 0,
                                    p2_m1: 0,
                                    p2_m2: 0,
                                    subgame_index: Option::<SubgameIndex>::None,
                                    player_to_move: ChanceOrPlayer::Player(Player::Player2),
                                };
                                VertexOrLeaf::Vertex(new_state) 
                            },
                            Action::NextSubgame(_) => panic!()
                        }
                    },
                    ChanceOrPlayer::Player(Player::Player2) => {
                        match action {
                            Action::ActionIndex(action_index) => {
                                let new_state = Self {
                                    synth: self.synth, 
                                    p1_m1: self.p1_m1,
                                    p1_m2: 0,
                                    p2_m1: *action_index,
                                    p2_m2: 0,
                                    subgame_index: Option::<SubgameIndex>::None,
                                    player_to_move: ChanceOrPlayer::Chance,
                                };
                                VertexOrLeaf::Vertex(new_state) 
                            },
                            Action::NextSubgame(_) => panic!()
                        }
                        
                    }
                }
            },
            // In subgame.
            Some(_subgame_index) => {
                match self.player_to_move {
                    ChanceOrPlayer::Chance => panic!(),
                    ChanceOrPlayer::Player(Player::Player1) => {
                        match action {
                            Action::ActionIndex(action_index) => {
                                let new_state = Self {
                                    synth: self.synth, 
                                    p1_m1: self.p1_m1,
                                    p1_m2: *action_index,
                                    p2_m1: self.p2_m1,
                                    p2_m2: 0,
                                    subgame_index: self.subgame_index,
                                    player_to_move: ChanceOrPlayer::Player(Player::Player2),
                                };
                                VertexOrLeaf::Vertex(new_state)
                            },
                            Action::NextSubgame(_) => panic!()
                        }
                    },
                    ChanceOrPlayer::Player(Player::Player2) => {
                        match action {
                            Action::ActionIndex(action_index) => {
                                let leaf_state = Self {
                                    synth: self.synth, 
                                    p1_m1: self.p1_m1,
                                    p1_m2: self.p1_m2,
                                    p2_m1: self.p2_m1,
                                    p2_m2: *action_index,
                                    subgame_index: self.subgame_index,
                                    player_to_move: ChanceOrPlayer::Player(Player::Player2),
                                };
                                let (payoff_pl1, payoff_pl2) = leaf_state.get_payoffs();
                                let leaf = Leaf {
                                    payoff_pl1,
                                    payoff_pl2,
                                };
                                VertexOrLeaf::Leaf(leaf)
                            },
                            Action::NextSubgame(_) => panic!()
                        }
                    }
                }
            }
        }

        /*
        let leaf: Leaf = Leaf {
            payoff_pl1: 1f64,
            payoff_pl2: 1f64,
        };
        VertexOrLeaf::Leaf(leaf)
        */
    }

    fn subgame(&self) -> Option<Self::Subgame> {
        match self.synth.specify_subgames {
            true => {
                match self.subgame_index {
                    None => Option::<Self::Subgame>::None,
                    Some(index) => {
                        Option::<Self::Subgame>::Some(
                            Self::Subgame {
                                p1_m1: self.p1_m1,
                                p2_m1: self.p2_m1,
                                subgame_index: index,
                            }
                        )
                    },
                }
            },
            false => self.dummy_subgame(),
        }
    }

}

#[derive(StructOpt, Debug)]
#[structopt(name = "rps")]
struct Opt {
    #[structopt(short = "o", long = "output_game_file")]
    output_game_file: PathBuf,

    #[structopt(short = "b", long = "output_blueprint_file")]
    output_blueprint_file: PathBuf,

    #[structopt(short = "M", long = "num_subgames")]
    num_subgames: usize,

    #[structopt(short = "n", long = "main_game_size")]
    main_game_size: Vec<usize>,

    #[structopt(short = "m", long = "subgame_size")]
    subgame_size: Vec<usize>,

    #[structopt(short = "p", long = "influence")]
    influence_of_main_action: f64,

    #[structopt(short = "u", long = "main_game_payoffs")]
    main_game_payoff_range: Vec<f64>,

    #[structopt(short = "v", long = "subgame_payoffs")]
    subgame_payoff_range: Vec<f64>,

    #[structopt(short = "s", long = "specify_subgames")]
    specify_subgames: bool,

    #[structopt(short = "r", long = "random_seed")]
    random_seed: usize,
}



fn blueprint<'a>(synth: &Synth,
                 efg: &'a ExtensiveFormGame, 
                 annotations: &GameAnnotations<State>,
                 ) -> SequenceFormStrategy<'a> {

    // println!("{:?}", annotations);
    
    // Solve normal form representation of the main game.
    let solver = GurobiSolver::new(&synth.main_game_payoffs_pl1,
                               &synth.main_game_payoffs_pl2);
    let sol = solver.solve();

    let mut beh = BehavioralStrategy::from_uniform_strategy(efg.treeplex(Player::Player1));
    let mut embedded = beh.inner().clone();

    // Inject solution into sequence form matrix.
    let ann = &annotations.treeplex_annotations_pl1.sequence_annotations;
    for (sequence_id, a_) in ann.iter().enumerate() {
        match a_ {
            None=>continue, // Empty sequence
            Some(a)=> {
                // If there are no subgames to be specified, continue,
                // if !synth.specify_subgames {
                //     continue;
                // }
                // If we are in a subgame, we leave it as uniform.
                // Note: The following doesn't work in the case where synth.specify_subgames = false
                // since even if in the second stage, the subgame will still be `None`.
                // match a.0.subgame {
                //     None => (),
                //     Some(_) => continue,
                // };

                // Hence, we have to check manually to make sure we are at the 
                // second stage.
                match a.0.p1_m1_o {
                    None => (),
                    Some(_) => continue,
                };

                // If not in subgame, 
                match &a.1 {
                    Action::NextSubgame(_)=>panic!(),
                    Action::ActionIndex(action_index)=> {
                        embedded[sequence_id] = sol.leader_strategy[*action_index];
                    }
                }
                 
            }
        };
    }

    let blueprint_beh = BehavioralStrategy::from_treeplex_vector(embedded);
    SequenceFormStrategy::from_behavioral_strategy(blueprint_beh)
    
    // println!("{:?}", annotations);
    // println!("{:?}", sol);
    // println!("{:?}", annotations.treeplex_annotations_pl1.sequence_annotations);
    
    

}

fn main() {
    env_logger::init();

    let opt = Opt::from_args();

    println!("Subgames: {:?}", opt.specify_subgames);

    let config = Config::new(
        opt.num_subgames,
        (opt.main_game_size[0], opt.main_game_size[1]), 
        (opt.subgame_size[0], opt.subgame_size[1]),
        opt.influence_of_main_action, 
        (opt.main_game_payoff_range[0], opt.main_game_payoff_range[1]),
        (opt.subgame_payoff_range[0], opt.subgame_payoff_range[1]),
        opt.specify_subgames,
    );

    println!("Making Synth");    
    let synth = Synth::new(&config, opt.random_seed);

    println!("Making builder");
    let mut builder = ExtensiveFormGameBuilder::<State>::new();

    let initial_state = State::initial_state(&synth);
    let (efg, _annotation) = builder.make_game_and_annotations(&initial_state, true);
    let annotations = _annotation.unwrap();

    println!("num sequences {:?}", efg.treeplex(Player::Player1).num_sequences());
    println!("num infosets {:?}", efg.treeplex(Player::Player1).num_infosets());

    println!("Compute blueprints for main game and embed it into the extensive form game");
    let blueprint = blueprint(&synth, &efg, &annotations);

    let mut game_file_writer = File::create(&opt.output_game_file).unwrap();       
    efg.persist(&mut game_file_writer).unwrap();

    let mut blueprint_file_writer = File::create(&opt.output_blueprint_file).unwrap();
    blueprint.inner().persist(&mut blueprint_file_writer).unwrap();

    panic!();

    // Debug
    debug!("Player 1 sequence annotations");
    for (i, s) in annotations
        .treeplex_annotations_pl1
        .sequence_annotations
        .into_iter()
        .enumerate()
    {
        if i == 15703 {
            // println!("15703: {:?}", s.clone());
        }
        match s {
            Some(s) => debug!("{:?}: {:?}", i, s),
            None => debug!("{:?}: {:?}", i, "Empty"),
        }

    }

    debug!("Player 2 sequence annotations");
    for (i, s) in annotations
        .treeplex_annotations_pl2
        .sequence_annotations
        .into_iter()
        .enumerate()
    {
        match s {
            Some(s) => debug!("{:?}: {:?}", i, s),
            None => debug!("{:?}: {:?}", i, "Empty"),
        }
    }

    debug!("Player 1 infoset annotations");
    for (i, a) in annotations
        .treeplex_annotations_pl1
        .infoset_annotations
        .into_iter()
        .enumerate()
    {
        debug!("{:?}: {:?}", i, a.unwrap());
    }
    debug!("Player 2 infoset annotations");
    for (i, a) in annotations
        .treeplex_annotations_pl2
        .infoset_annotations
        .into_iter()
        .enumerate()
    {
        debug!("{:?}: {:?}", i, a.unwrap());
    }

    debug!("Player 1 action annotations");
    for (i, a) in annotations
        .treeplex_annotations_pl1
        .action_annotations
        .into_iter()
        .enumerate()
    {
        debug!("{:?}: {:?}", i, a.unwrap());
    }

    debug!("Player 2 action annotations");
    for (i, a) in annotations
        .treeplex_annotations_pl2
        .action_annotations
        .into_iter()
        .enumerate()
    {
        debug!("{:?}: {:?}", i, a.unwrap());
    }

    debug!("Player 1 infosets");
    for (i, infoset) in efg.treeplex(Player::Player1).infosets().iter().enumerate() {
        debug!("{:?}, {:?}", i, infoset);
    }

    debug!("Player 2 infosets");
    for (i, infoset) in efg.treeplex(Player::Player2).infosets().iter().enumerate() {
        debug!("{:?}, {:?}", i, infoset);
    }

    debug!("Payoff Matrix");
    for i in efg.payoff_matrix().entries.iter() {
        debug!("{:?}", i);
    }


}
