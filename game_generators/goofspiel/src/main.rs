// Re-implementation of Goofspiel.
// TODO (chunkail) refactor into different files.

extern crate efg_lite;
extern crate env_logger;
extern crate structopt;

use efg_lite::game::Player;
use log::debug;

use itertools::Itertools;
use libgt::{ChanceOrPlayer, ExtensiveFormGameBuilder, GameTreeVertex, Leaf, VertexOrLeaf};
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Config {
    pub num_cards: usize,
    // pub limited_feedback: bool, // TODO (chunkail) : implement.
    pub is_prize_shuffled: bool,

    // Whether game is zero sum.
    pub is_zero_sum: bool,

    // Number of cards remaining to be won before subgames are generated.
    // 0 means that there are no subgames generated.
    pub subgame_depth: usize,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct State<'a> {
    config: &'a Config,

    cards_played_pl1: Vec<usize>,
    cards_played_pl2: Vec<usize>,
    prizes_revealed: Vec<usize>,
}

impl<'a> State<'a> {
    /// Creates a state coressponding to the root of the game tree
    /// (which in the case of goofspeil, belongs to chance).
    pub fn make_initial(config: &'a Config) -> State {
        State {
            config,
            cards_played_pl1: vec![],
            cards_played_pl2: vec![],
            prizes_revealed: vec![],
        }
    }

    /// Checks if this is a  valid goofspiel state. Panics if the state
    /// is an invalid one. Note that these checks are for sanity and may not cover
    /// every possible case of invalid states!
    pub fn validate(&self) {
        // Obvious sanity checks.
        assert!(self.prizes_revealed.len() <= self.config.num_cards);
        assert!(self.cards_played_pl1.len() <= self.config.num_cards);
        assert!(self.cards_played_pl2.len() <= self.config.num_cards);
        assert!(self.prizes_revealed.len() >= self.cards_played_pl1.len());
        assert!(self.prizes_revealed.len() >= self.cards_played_pl2.len());
        assert!(self.prizes_revealed.len() <= self.cards_played_pl1.len() + 1);
        assert!(self.prizes_revealed.len() <= self.cards_played_pl2.len() + 1);
        assert!(self.cards_played_pl1.len() >= self.cards_played_pl2.len());
        assert!(self.cards_played_pl1.len() <= self.cards_played_pl2.len() + 1);
        assert_eq!(
            self.cards_played_pl1.iter().unique().count(),
            self.cards_played_pl1.len()
        );
        assert_eq!(
            self.cards_played_pl2.iter().unique().count(),
            self.cards_played_pl2.len()
        );
        assert_eq!(
            self.prizes_revealed.iter().unique().count(),
            self.prizes_revealed.len()
        );
    }

    pub fn is_game_over(&self) -> bool {
        self.validate();
        // Game is over when player 2 has played the number of cards equal
        // to the number of prizes available.
        self.cards_played_pl2.len() == self.config.num_cards
    }

    /// This computes the payoffs at terminal states (leaves) of the game.
    /// In the basic version of goofspiel, we assume that no player wins
    /// the prize if players bid the same amount.
    pub fn get_leaf(&self) -> Leaf {
        assert!(self.is_game_over());
        let mut payoff_pl1: f64 = 0.0f64;
        let mut payoff_pl2: f64 = 0.0f64;
        for (i, prize_value) in self.prizes_revealed.iter().enumerate() {
            let bid_pl1 = self.cards_played_pl1[i];
            let bid_pl2 = self.cards_played_pl2[i];

            match self.config.is_zero_sum {
                false => {
                    if bid_pl1 > bid_pl2 {
                        payoff_pl1 += *prize_value as f64;
                    }
                    if bid_pl2 > bid_pl1 {
                        payoff_pl2 += *prize_value as f64;
                    }
                }
                true => {
                    if bid_pl1 > bid_pl2 {
                        payoff_pl1 += (*prize_value as f64)/2f64;
                        payoff_pl2 -= (*prize_value as f64)/2f64;
                    }
                    if bid_pl2 > bid_pl1 {
                        payoff_pl2 += (*prize_value as f64)/2f64;
                        payoff_pl1 -= (*prize_value as f64)/2f64;
                    }
                }
            }
        }
        Leaf {
            payoff_pl1,
            payoff_pl2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlayerInfo<T> {
    prizes_revealed: Box<[usize]>,
    cards_played_pl1: Box<[T]>,
    cards_played_pl2: Box<[T]>,
}

/// Goofspiel is a Markovian game, so all actions are private save
/// for those played in the current round. So, the description of a game
/// state is very similar to the description of the game vertex itself,
/// except that private moves in that very round are omitted.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subgame {
    cards_played_pl1: Vec<usize>,
    cards_played_pl2: Vec<usize>,
    prizes_revealed: Vec<usize>,
}


impl<T> PlayerInfo<T> {
    pub fn new(
        prizes_revealed: Box<[usize]>,
        cards_played_pl1: Box<[T]>,
        cards_played_pl2: Box<[T]>,
    ) -> PlayerInfo<T> {
        PlayerInfo {
            prizes_revealed,
            cards_played_pl1,
            cards_played_pl2,
        }
    }
}

impl<'a> GameTreeVertex for State<'a> {
    type Action = usize;

    /// In the no-chance version of the game, the history of the players' actions
    /// and past prizes revealed suffices.
    type PlayerInfo = PlayerInfo<usize>;

    type Subgame = Subgame;

    fn next_player(&self) -> ChanceOrPlayer {
        assert!(self.is_game_over() == false);

        if self.cards_played_pl2.len() == self.prizes_revealed.len() {
            ChanceOrPlayer::Chance
        } else if self.cards_played_pl2.len() < self.cards_played_pl1.len() {
            ChanceOrPlayer::Player(Player::Player2)
        } else {
            ChanceOrPlayer::Player(Player::Player1)
        }
    }

    fn player_information(&self) -> Self::PlayerInfo {
        // For the game with full information (of opponent's cards played),
        // the player information includes the cards which have been played by each player,
        // with the exception that player 2 does not know the last card player 1 played.
        // This is independent of which player is about to move.
        //
        // In addition, we will need the history of the cards which we are bidding.
        match self.next_player() {
            ChanceOrPlayer::Chance => panic!(),
            ChanceOrPlayer::Player(player) => {
                match player {
                    Player::Player1 => Self::PlayerInfo::new(
                        self.prizes_revealed.clone().into_boxed_slice(),
                        self.cards_played_pl1.clone().into_boxed_slice(),
                        self.cards_played_pl2.clone().into_boxed_slice(),
                    ),
                    Player::Player2 => {
                        // Make sure we only allow p2 to see all but the last card p1 has played.
                        // TOOD (chunkail): reorganize this...
                        let rounds_completed = self.cards_played_pl2.len();
                        Self::PlayerInfo::new(
                            self.prizes_revealed.clone().into_boxed_slice(),
                            self.cards_played_pl1
                                .iter()
                                .take(rounds_completed)
                                .map(|&x| x)
                                .collect::<Vec<_>>()
                                .into_boxed_slice(),
                            self.cards_played_pl2.clone().into_boxed_slice(),
                        )
                    }
                }
            }
        }
    }

    fn available_actions(&self) -> Box<[(Self::Action, f64)]> {
        match self.next_player() {
            ChanceOrPlayer::Chance => {
                // Check if we are playing with shuffled prizes or otherwise.
                match self.config.is_prize_shuffled {
                    true => {
                        // Compute the (uniform) probability for each prize that is remaining.
                        let num_prizes_remaining =
                            self.config.num_cards - self.prizes_revealed.len();
                        let card_probability: f64 = 1.0 / (num_prizes_remaining as f64);
                        // Filter out prizes which have not been encountered before and
                        // add them into boxed slice.
                        (0..self.config.num_cards)
                            .into_iter()
                            .filter(|&x| !self.prizes_revealed.contains(&x))
                            .map(|x| (x, card_probability))
                            .collect::<Vec<(Self::Action, f64)>>()
                            .into_boxed_slice()
                    }
                    false => vec![(self.prizes_revealed.len(), 1.0f64)].into_boxed_slice(),
                }
            }
            // If the state is not chance, then the player takes an action, we
            // filter out cards which have not been played before and box them up.
            ChanceOrPlayer::Player(player) => match player {
                Player::Player1 => (0..self.config.num_cards)
                    .into_iter()
                    .filter(|&x| !self.cards_played_pl1.contains(&x))
                    .map(|x| (x, 1.0f64))
                    .collect::<Vec<(Self::Action, f64)>>()
                    .into_boxed_slice(),
                Player::Player2 => (0..self.config.num_cards)
                    .into_iter()
                    .filter(|&x| !self.cards_played_pl2.contains(&x))
                    .map(|x| (x, 1.0f64))
                    .collect::<Vec<(Self::Action, f64)>>()
                    .into_boxed_slice(),
            },
        }
    }

    /// Computes the next state from an action (which could be from chance).
    fn next_state(&self, action: &Self::Action) -> VertexOrLeaf<Self> {
        let new_state = match self.next_player() {
            ChanceOrPlayer::Chance => {
                // If the current state is a chance node, then we should
                let mut new_prizes_revealed = self.prizes_revealed.clone();
                new_prizes_revealed.push(action.clone());
                State {
                    config: self.config,
                    prizes_revealed: new_prizes_revealed,
                    cards_played_pl1: self.cards_played_pl1.clone(),
                    cards_played_pl2: self.cards_played_pl2.clone(),
                }
            }
            ChanceOrPlayer::Player(player) => match player {
                Player::Player1 => {
                    let mut new_cards_played_pl1 = self.cards_played_pl1.clone();
                    new_cards_played_pl1.push(action.clone());
                    State {
                        config: self.config,
                        prizes_revealed: self.prizes_revealed.clone(),
                        cards_played_pl1: new_cards_played_pl1,
                        cards_played_pl2: self.cards_played_pl2.clone(),
                    }
                }
                Player::Player2 => {
                    let mut new_cards_played_pl2 = self.cards_played_pl2.clone();
                    new_cards_played_pl2.push(action.clone());
                    State {
                        config: self.config,
                        prizes_revealed: self.prizes_revealed.clone(),
                        cards_played_pl1: self.cards_played_pl1.clone(),
                        cards_played_pl2: new_cards_played_pl2,
                    }
                }
            },
        };

        // Now, check if the game is over. We return `VertexOrLeaf::Vertex`
        // if the game is not over, and `VertexOrLeaf::Leaf` if the game is over.
        match new_state.is_game_over() {
            true => {
                let leaf = new_state.get_leaf();
                VertexOrLeaf::Leaf(leaf)
            }
            false => VertexOrLeaf::Vertex(new_state),
        }
    }

    fn subgame(&self) -> Option<Self::Subgame> {
        // At this point, we will assume that the state is indeed valid.
        let num_rounds_resolved = self.cards_played_pl2.len();
        assert!(self.config.num_cards >= num_rounds_resolved);
        let num_rounds_remaining = self.config.num_cards - num_rounds_resolved; // Includes the current round.
        if num_rounds_remaining > self.config.subgame_depth {
            None
        } else {
            let subgame_history_len = self.config.num_cards - self.config.subgame_depth;
            Some(Subgame {
                cards_played_pl1: (&self.cards_played_pl1[0..subgame_history_len]).to_vec(),
                cards_played_pl2: (&self.cards_played_pl2[0..subgame_history_len]).to_vec(),
                prizes_revealed: (&self.prizes_revealed[0..subgame_history_len]).to_vec(),
            })
        }
    }
}

impl Config {
    pub fn new(
        num_cards: usize,
        is_prize_shuffled: bool,
        is_zero_sum: bool,
        subgame_depth: usize,
    ) -> Config {
        Config {
            num_cards,
            is_prize_shuffled,
            is_zero_sum,
            subgame_depth,
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "goofspiel")]
struct Opt {
    // Number of cards/prizes
    #[structopt(short = "n", long = "num_cards")]
    num_cards: usize,

    // Are prizes shuffled
    #[structopt(short = "s", long = "is_prize_shuffled")]
    is_prize_shuffled: bool,

    // Is game zero sum
    #[structopt(short = "z", long = "is_zero_sum")]
    is_zero_sum: bool,

    // Subgame depth
    #[structopt(short = "d", long = "subgame_depth")]
    subgame_depth: usize,

    // Output file
    #[structopt(short = "o", long = "ouput_file")]
    output_file: PathBuf,
}

fn main() {

    env_logger::init();

    let opt = Opt::from_args();
    let config = Config::new(
        opt.num_cards,
        opt.is_prize_shuffled,
        opt.is_zero_sum,
        opt.subgame_depth,
    );

    debug!("Configuration selected: {:?}", config);

    let initial_state = State::make_initial(&config);

    debug!("Initial state {:?}", initial_state);

    let mut builder = ExtensiveFormGameBuilder::<State>::new();

    let (efg, _annotation) = builder.make_game_and_annotations(&initial_state, true);
    let annotations = _annotation.unwrap();

    debug!("Player 1 sequence annotations");
    for (i, s) in annotations
        .treeplex_annotations_pl1
        .sequence_annotations
        .into_iter()
        .enumerate()
    {
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
    for (i, a) in annotations
        .treeplex_annotations_pl2
        .action_annotations
        .into_iter()
        .enumerate()
    {
        debug!("{:?}: {:?}", i, a.unwrap());
    }

    let mut file_writer = File::create(&opt.output_file).unwrap();
    efg.persist(&mut file_writer).unwrap();
}