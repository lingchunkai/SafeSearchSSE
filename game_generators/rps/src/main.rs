// RPS for debugging purposes.

extern crate efg_lite;
extern crate env_logger;
extern crate structopt;


use efg_lite::game::Player;
use libgt::{ChanceOrPlayer, GameTreeVertex, ExtensiveFormGameBuilder, Leaf, VertexOrLeaf};
use log::{debug, info};
use structopt::StructOpt;

use std::fs::File;
use std::path::PathBuf;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct State {
    action_pl1: Option<Action>,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Action {
    Rock,
    Paper,
    Scissors,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct DummyInfo {}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct DummySubgame {}

impl State {
    pub fn initial_state() -> State {
        State {
            action_pl1: Option::None,
        }
    }
}

impl GameTreeVertex for State {
    type Action = Action;

    // There is only one information set per player.
    type PlayerInfo = DummyInfo;
    
    type Subgame = DummySubgame;

    fn next_player(&self) -> ChanceOrPlayer {
        match self.action_pl1.clone() {
            None => ChanceOrPlayer::Player(Player::Player1),
            Some(_) => ChanceOrPlayer::Player(Player::Player2),
        }
    }

    fn player_information(&self) -> Self::PlayerInfo {
        Self::PlayerInfo {}
    }

    fn available_actions(&self) -> Box<[(Self::Action, f64)]> {
        vec![
            (Action::Rock, 1.0f64),
            (Action::Scissors, 1.0f64),
            (Action::Paper, 1.0f64),
        ]
        .into_boxed_slice()
    }

    fn next_state(&self, action: &Self::Action) -> VertexOrLeaf<Self> {
        match self.next_player() {
            ChanceOrPlayer::Chance => panic!("Unexpected chance vertex"),
            ChanceOrPlayer::Player(player) => match player {
                Player::Player1 => VertexOrLeaf::Vertex(State {
                    action_pl1: Option::Some(action.clone()),
                }),
                Player::Player2 => {
                    let payoff_pl1 = match self.action_pl1.clone().unwrap() {
                        Action::Rock => match action {
                            Action::Scissors => 1.0,
                            Action::Rock => 0.0,
                            Action::Paper => -1.0,
                        },
                        Action::Paper => match action {
                            Action::Scissors => -1.0,
                            Action::Rock => 1.0,
                            Action::Paper => 0.0,
                        },
                        Action::Scissors => match action {
                            Action::Scissors => 0.0,
                            Action::Rock => -1.0,
                            Action::Paper => 1.0,
                        },
                    };
                    let leaf: Leaf = Leaf {
                        payoff_pl1: payoff_pl1,
                        payoff_pl2: -payoff_pl1,
                    };
                    VertexOrLeaf::Leaf(leaf)
                }
            },
        }
    }
}


#[derive(StructOpt, Debug)]
#[structopt(name = "rps")]
struct Opt {
    #[structopt(short = "o", long = "output_file")]
    output_file: PathBuf,
}

fn main() {
    env_logger::init();

    let opt = Opt::from_args();

    let mut builder = ExtensiveFormGameBuilder::<State>::new();

    let initial_state = State::initial_state();
    let (efg, _annotation) = builder.make_game_and_annotations(&initial_state, true);
    let annotations = _annotation.unwrap();

    let mut file_writer = File::create(&opt.output_file).unwrap();       
    efg.persist(&mut file_writer).unwrap();

}
