// Leduc Poker.
/// We will make the assumption that there are n cards and exactly 2 suits.
/// We will typically keep n=3.
/// We will assume that there are 2 rounds, and the board card is revealed at the 
/// beginning of the second round.
/// In each round, we will allow a maximum of t raises. Each raise is adding a 
/// *predefined* quantity to the pot.
/// 

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

use noisy_float::prelude::*;
use std::str::FromStr;

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Ord)]
pub struct Config {
    pub num_cards: usize,

    pub raise_amount_r1: R64,
    pub raise_amount_r2: R64,

    pub pot_contribution_per_player: R64,

    pub max_raises_per_round: usize,
    pub rake_percentage: R64,

    pub subgame_setting: SubgameSetting,
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq, Ord)]
pub struct Card {
    suit: usize,
    value: usize,
}

impl Config {
    pub fn new(num_cards: usize, 
                raise_amounts: &Vec<R64>,
                pot_contribution_per_player: R64,
                max_raises_per_round: usize,
                rake_percentage: R64,
                subgame_setting: SubgameSetting) 
    -> Config {
        Config {
            num_cards,
            raise_amount_r1: raise_amounts[0],
            raise_amount_r2: raise_amounts[1],
            pot_contribution_per_player: pot_contribution_per_player,
            rake_percentage,
            max_raises_per_round,
            subgame_setting,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq, Ord)]
pub enum SubgameSetting {
    None, 
    SecondRound,
    NthAction(usize),
}

impl FromStr for SubgameSetting {
    type Err = String;
    fn from_str(setting: &str) -> Result<Self, Self::Err> {
        match setting {
            "none" => Ok(SubgameSetting::None),
            "second_round" => Ok(SubgameSetting::SecondRound),
            "3th_action"=> Ok(SubgameSetting::NthAction(3)),
            "2th_action"=> Ok(SubgameSetting::NthAction(2)),
            _ => Err("Cannot parse initial setting".to_string()),
        }
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Action {
    Fold,
    Call,
    Raise,
    Deal(Card, Card, Card),
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Phase {
    Deal,
    Round(usize),
    End,
}

#[derive(Debug, Copy, Clone)]
pub enum Outcome {
    Tie,
    Win(Player)
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct State<'a> {
    config: &'a Config,

    /// Private card information for Player 1.
    card_pl1: Card,

    /// Private card information for Player 2.
    card_pl2: Card,

    /// Public card. 
    public_card: Card,

    /// Pot contributed by Player 1.
    pot_pl1: R64,

    /// Pot contributed by Player 2.
    pot_pl2: R64,

    /// Number of raises, including (potentially) the raise in the first action of the round.
    num_raises_r1: usize,

    /// First action of round 1, which may be either a raise or a call (techncially a fold is dominated, 
    /// but for SSE this still be played...? Probably still has prob. 0). None if this round has not 
    /// started yet.
    first_action_r1: Option<Action>,

    /// Final action of round 2, played when the round ends. This can be fold or call (not raise).
    /// None if the game has not ended yet.
    last_action_r1: Option<Action>,

    num_raises_r2: usize,
    first_action_r2: Option<Action>,
    last_action_r2: Option<Action>,
}

impl<'a> State<'a> {
    pub fn make_initial(config: &'a Config) -> State {
        State {
            config,

            card_pl1: Card{ suit: 2, value: config.num_cards},
            card_pl2: Card{ suit: 2, value: config.num_cards},
            public_card: Card{ suit: 2, value: config.num_cards},

            pot_pl1: config.pot_contribution_per_player, 
            pot_pl2: config.pot_contribution_per_player, 

            num_raises_r1: 0,
            first_action_r1: None,
            last_action_r1: None,

            num_raises_r2: 0, 
            first_action_r2: None,
            last_action_r2: None,
        }
    }

    /// Check if there are any abnormal things going on. 
    /// Note that this will return FALSE for the initial state, which
    /// is filled wtith dummy cards. 
    fn validate_non_initial(&self) -> bool {
        self.card_pl1.suit < 2 &&
        self.card_pl2.suit < 2 && 
        self.public_card.suit < 2 &&
        self.card_pl1.value < self.config.num_cards &&
        self.card_pl2.value < self.config.num_cards && 
        self.public_card.value < self.config.num_cards && 
        (self.first_action_r1.is_some() || self.last_action_r1.is_none()) && // Avoid case where first aciton is none and last action is some.
        (self.first_action_r2.is_some() || self.last_action_r2.is_none()) && // Avoid case where first aciton is none and last action is some.
        (self.num_raises_r1 >= 1 || self.first_action_r1 != Some(Action::Raise)) &&
        (self.num_raises_r2 >= 1 || self.first_action_r2 != Some(Action::Raise)) &&
        self.num_raises_r1 <= self.config.max_raises_per_round && 
        self.num_raises_r2 <= self.config.max_raises_per_round
    }

    pub fn is_game_over(&self) -> bool {
        self.current_phase() == Phase::End
    }

    fn current_phase(&self) -> Phase {
        if !self.have_cards_been_dealt() {
            // Cards have not been dealt yet.
            Phase::Deal
        } else if self.last_action_r1.is_none() {
            // Round 1 has not ended yet, but the cards have already been dealt.
            Phase::Round(0)
        } else if self.last_action_r1 == Some(Action::Fold) {
            // If first round ended up with a fold, then the game has ended.
            Phase::End
        } else if self.last_action_r2.is_none() {
            // We have not folded in r1, but r2's last action has not been fixed yet.
            Phase::Round(1)
        } else {
            // Round 2 has ended either in a call (showdown) or a fold
            Phase::End
        }
    }

    fn have_cards_been_dealt(&self) -> bool {
        // Cards are dealt when player does not have some dummy card.
        self.card_pl1.suit != 2 
    }

    pub fn pot(&self, player: Player) -> R64 {
        match player {
            Player::Player1 => self.pot_pl1,
            Player::Player2 => self.pot_pl2,
        }
    }

    fn showdown_winner(&self) -> Outcome {
        // Check if anyone shares the same card as the public one. 
        if self.card_pl1.value == self.public_card.value {
            return Outcome::Win(Player::Player1);
        }
        if self.card_pl2.value == self.public_card.value {
            return Outcome::Win(Player::Player2);
        }

        // No one matches the public card, now we check who has the higher card, if it
        // is a tie, then it is a draw.
        if self.card_pl1.value == self.card_pl2.value {
            return Outcome::Tie;
        } else if self.card_pl1.value > self.card_pl2.value {
            return Outcome::Win(Player::Player1);
        } else if self.card_pl2.value > self.card_pl1.value {
            return Outcome::Win(Player::Player2);
        }

        panic!();
    }

    /// Number of actions taken in a completed round, i.e., first and last actions aer 
    fn num_actions_in_completed_round(first_action: Action, 
                            last_action: Action, 
                            num_raises: usize) -> usize {
        assert!(last_action == Action::Fold || last_action == Action::Call);
        if first_action == Action::Raise {
            // Number of raises (including the first action) plus the last.
            num_raises + 1
        } else {
            // Number of raises, plus 1 for the first and last moves each.
            num_raises + 2
        }
    }

    fn last_player_in_round(first_action: Option<Action>, 
                            last_action: Option<Action>, 
                            num_raises: usize) -> Player {
        let num_actions = Self::num_actions_in_round(first_action, last_action, num_raises);
        match num_actions % 2 {
            0 => Player::Player2, 
            1 => Player::Player1,
            _ => { panic!(); }
        }
    }

    fn num_actions_in_round(first_action: Option<Action>, 
                            last_action: Option<Action>,
                            num_raises: usize) -> usize {
        assert!(first_action != Some(Action::Fold));

        if first_action.is_none() {
            return 0;
        } 

        if last_action.is_some() {
            return Self::num_actions_in_completed_round(first_action.unwrap(), last_action.unwrap(), num_raises);
        }
        
        if first_action == Some(Action::Raise) {
            return num_raises;
        } else if first_action == Some(Action::Call) {
            return num_raises + 1;
        }

        panic!();
    }

    fn payoff(&self, outcome: Outcome) -> Leaf {
        let total_pot = (self.pot_pl1 + self.pot_pl2).raw();
        let raked_value = ((self.config.rake_percentage) * 2f64 * self.pot_pl1.min(self.pot_pl2)).raw();
        let winner_revenue = total_pot - raked_value;
        let tie_revenue = (total_pot - raked_value) / 2f64;
        match outcome {
            Outcome::Tie => Leaf {
                payoff_pl1: tie_revenue - self.pot_pl1.raw(),
                payoff_pl2: tie_revenue - self.pot_pl2.raw(),
            },
            Outcome::Win(Player::Player1) => Leaf { 
                payoff_pl1: winner_revenue - self.pot_pl1.raw(), 
                payoff_pl2: -self.pot_pl2.raw()
            },
            Outcome::Win(Player::Player2) => Leaf {
                payoff_pl1: -self.pot_pl1.raw(), 
                payoff_pl2: winner_revenue - self.pot_pl2.raw()
            }
        }
    }

    /// This computes the payoffs at terminal states (leaves) of the game.
    pub fn get_leaf(&self) -> Leaf {
        assert!(self.is_game_over());
        if self.last_action_r1 == Some(Action::Fold) || self.last_action_r2 == Some(Action::Fold) {
            let last_player = Self::last_player_in_round(self.first_action_r1, 
                                                         self.last_action_r1,
                                                         self.num_raises_r1);
            self.payoff(Outcome::Win(-last_player))
        } else {
            self.payoff(self.showdown_winner())
        }
    }

    pub fn get_player_info(&self, player: Player) -> PlayerInfo {
        PlayerInfo {
            // Public card is `None` if there we are in the first betting round, and 
            // is revealed in the second round.
            public_card: match self.current_phase() {
                Phase::Deal => { panic!(); },
                Phase::End => { panic!(); },
                Phase::Round(round) => match round {
                    0 => None,
                    1 => Some(self.public_card),
                    _ => { panic!(); },
                }
            },

            // Player's private cards are always revealed.
            private_card: match player {
                Player::Player1 => self.card_pl1,
                Player::Player2 => self.card_pl2,
            },

            // Public information of player actions.
            num_raises_r1: self.num_raises_r1,
            first_action_r1: self.first_action_r1,
            last_action_r1: self.last_action_r1,

            num_raises_r2: self.num_raises_r2,
            first_action_r2: self.first_action_r2,
            last_action_r2: self.last_action_r2,
        }
    }

    fn dummy_subgame(&self) -> Subgame {
        Subgame {
            public_card: Some (Card {suit: 2, value: self.config.num_cards }), // Impossible suit
            num_raises_r1: 0,
            first_action_r1: Action::Fold,
            last_action_r1: Some(Action::Fold),
        }
    }
}

/// Everything about the game is known the player, save for the opponent's private card.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlayerInfo {
    // Public information
    public_card: Option<Card>,

    num_raises_r1: usize,
    first_action_r1: Option<Action>,
    last_action_r1: Option<Action>,

    num_raises_r2: usize,
    first_action_r2: Option<Action>,
    last_action_r2: Option<Action>,

    // Private information
    private_card: Card,
}

/*
/// Subgame information are the cards played in the first round and the (revealed) floor card.
/// TODO (maybe change?)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subgame {
    public_card: Card,

    num_raises_r1: usize,
    first_action_r1: Action,
    last_action_r1: Action,
}
*/

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subgame {
    public_card: Option<Card>,

    num_raises_r1: usize,
    first_action_r1: Action,
    last_action_r1: Option<Action>,
}


impl<'a> GameTreeVertex for State<'a> {
    type Action = Action;

    type PlayerInfo = PlayerInfo;

    type Subgame = Subgame;

    fn next_player(&self) -> ChanceOrPlayer {
        assert!(self.is_game_over() == false);

        match self.current_phase() {
            Phase::Deal => {
                ChanceOrPlayer::Chance 
            },
            Phase::End => {
                panic!();
            },
            Phase::Round(round) => {
                match round {
                    0 => ChanceOrPlayer::Player(
                        -Self::last_player_in_round(
                            self.first_action_r1,
                            self.last_action_r1, 
                            self.num_raises_r1)), 
                    1 => ChanceOrPlayer::Player(
                        -Self::last_player_in_round(
                            self.first_action_r2,
                            self.last_action_r2, 
                            self.num_raises_r2)), 
                    _ => { panic!(); }
                }
            }
        }
    }

    fn player_information(&self) -> Self::PlayerInfo {
        match self.next_player() {
            ChanceOrPlayer::Chance => panic!(),
            ChanceOrPlayer::Player(player) => {
                self.get_player_info(player)
            }
        }
    }

    fn available_actions(&self) -> Box<[(Self::Action, f64)]> {
        assert!(!self.is_game_over());
        match self.current_phase() {
            Phase::Deal => {
                // Deal cards, choose 3 out of all of the possible cards.
                let mut card_list = Vec::<Card>::new();
                for card_value in 0..self.config.num_cards {
                    card_list.push(Card { value: card_value, suit: 0 });
                    card_list.push(Card { value: card_value, suit: 1 });
                }

                let num_cards = card_list.len();
                let num_permutations = num_cards * (num_cards - 1) * (num_cards - 2);
                let prob = 1f64 / (num_permutations as f64);

                let cards_dealt_list = card_list.iter().combinations(3);
                
                let mut cards_dealt_permuted = vec![];
                for x in cards_dealt_list {
                    cards_dealt_permuted.push((Action::Deal(*x[0], *x[1], *x[2]), prob));
                    cards_dealt_permuted.push((Action::Deal(*x[0], *x[2], *x[1]), prob));
                    cards_dealt_permuted.push((Action::Deal(*x[1], *x[0], *x[2]), prob));
                    cards_dealt_permuted.push((Action::Deal(*x[1], *x[2], *x[0]), prob));
                    cards_dealt_permuted.push((Action::Deal(*x[2], *x[0], *x[1]), prob));
                    cards_dealt_permuted.push((Action::Deal(*x[2], *x[1], *x[0]), prob));
                }
                cards_dealt_permuted.into_boxed_slice()
            },

            Phase::End => {
                panic!();
            },

            Phase::Round(round) => {
                let (num_actions_this_round, num_raises_this_round) = match round {
                    0 => (Self::num_actions_in_round(
                                                self.first_action_r1, 
                                                self.last_action_r1,
                                                self.num_raises_r1),
                          self.num_raises_r1),
                    1 => (Self::num_actions_in_round(
                                                self.first_action_r2, 
                                                self.last_action_r2,
                                                self.num_raises_r2),
                          self.num_raises_r2),
                    _ => { panic!(); }
                };
                
                let mut move_list = Vec::<(Self::Action, f64)>::new();

                // One can always raise as long as we have not hit the maximum.
                if num_raises_this_round < self.config.max_raises_per_round {
                    move_list.push(
                        (Action::Raise, 1f64)
                    );
                }
                
                // One can only fold when at least 1 raise occured for this round.
                if num_raises_this_round > 0 {
                    move_list.push(
                        (Action::Fold, 1f64)
                    );
                }

                // Players can always call.
                move_list.push(
                    (Action::Call, 1f64)
                );

                move_list.into_boxed_slice()
            }
        }
    }

    /// Computes the next state from an action (which could be from chance).
    fn next_state(&self, action: &Self::Action) -> VertexOrLeaf<Self> {
        let new_state = match action {
            Action::Deal(card_pl1, card_pl2, public_card) => {
                assert_eq!(self.next_player(), ChanceOrPlayer::Chance);
                State {
                    config: self.config,
                    card_pl1: *card_pl1,
                    card_pl2: *card_pl2,
                    public_card: *public_card,
                    pot_pl1: self.pot_pl1, 
                    pot_pl2: self.pot_pl2,
                    num_raises_r1: self.num_raises_r1,
                    num_raises_r2: self.num_raises_r2,
                    last_action_r1: self.last_action_r1,
                    last_action_r2: self.last_action_r2,
                    first_action_r1: self.first_action_r1,
                    first_action_r2: self.first_action_r2,
                }
            }
            Action::Raise => {
                match self.current_phase() {
                    Phase::Round(0) => { 
                        let cur_player = -Self::last_player_in_round(
                                            self.first_action_r1,
                                            self.last_action_r1, 
                                            self.num_raises_r1);
                        State {
                            config: self.config,
                            card_pl1: self.card_pl1,
                            card_pl2: self.card_pl2,
                            public_card: self.public_card,
                            pot_pl1: match cur_player {
                                Player::Player1 => self.pot_pl2 + self.config.raise_amount_r1,
                                Player::Player2 => self.pot_pl1,
                            },
                            pot_pl2: match cur_player {
                                Player::Player1 => self.pot_pl2,
                                Player::Player2 => self.pot_pl1 + self.config.raise_amount_r1,
                            },
                            num_raises_r1: self.num_raises_r1 + 1,
                            num_raises_r2: self.num_raises_r2,
                            last_action_r1: self.last_action_r1,
                            last_action_r2: self.last_action_r2,
                            first_action_r1: match self.first_action_r1{
                                None => Some(Action::Raise),
                                Some(_) => self.first_action_r1,
                            },
                            first_action_r2: None,
                        }
                    },
                    Phase::Round(1) => { 
                        let cur_player = -Self::last_player_in_round(
                                            self.first_action_r2,
                                            self.last_action_r2, 
                                            self.num_raises_r2);
                        State {
                            config: self.config,
                            card_pl1: self.card_pl1,
                            card_pl2: self.card_pl2,
                            public_card: self.public_card,
                            pot_pl1: match cur_player {
                                Player::Player1 => self.pot_pl2 + self.config.raise_amount_r2,
                                Player::Player2 => self.pot_pl1,
                            },
                            pot_pl2: match cur_player {
                                Player::Player1 => self.pot_pl2,
                                Player::Player2 => self.pot_pl1 + self.config.raise_amount_r2,
                            },
                            num_raises_r1: self.num_raises_r1,
                            num_raises_r2: self.num_raises_r2 + 1,
                            last_action_r1: self.last_action_r1,
                            last_action_r2: self.last_action_r2,
                            first_action_r1: self.first_action_r1,
                            first_action_r2: match self.first_action_r2 {
                                None => Some(Action::Raise),
                                Some(_) => self.first_action_r2,
                            } 
                        }
                    },
                    _ => { panic!(); }
                }
            },
            Action::Fold => {
                match self.current_phase() {
                    Phase::Round(0) => { 
                        assert!(self.first_action_r1.is_some());
                        State {
                            config: self.config,
                            card_pl1: self.card_pl1,
                            card_pl2: self.card_pl2,
                            public_card: self.public_card,
                            pot_pl1: self.pot_pl1,
                            pot_pl2: self.pot_pl2,
                            num_raises_r1: self.num_raises_r1,
                            num_raises_r2: self.num_raises_r2,
                            last_action_r1: Some(Action::Fold),
                            last_action_r2: self.last_action_r2,
                            first_action_r1: self.first_action_r1,
                            first_action_r2: self.first_action_r2,
                        }
                    },
                    Phase::Round(1) => { 
                        assert!(self.first_action_r1.is_some());
                        State {
                            config: self.config,
                            card_pl1: self.card_pl1,
                            card_pl2: self.card_pl2,
                            public_card: self.public_card,
                            pot_pl1: self.pot_pl1,
                            pot_pl2: self.pot_pl2,
                            num_raises_r1: self.num_raises_r1,
                            num_raises_r2: self.num_raises_r2,
                            last_action_r1: self.last_action_r1,
                            last_action_r2: Some(Action::Fold),
                            first_action_r1: self.first_action_r1,
                            first_action_r2: self.first_action_r2,
                        }
                    },
                    _ => { panic!(); }
                }
            }
            Action::Call => {
                match self.current_phase() {
                    Phase::Round(0) => {
                        let cur_player = -Self::last_player_in_round(
                                            self.first_action_r1,
                                            self.last_action_r1, 
                                            self.num_raises_r1);
                        State {
                            config: self.config,
                            card_pl1: self.card_pl1,
                            card_pl2: self.card_pl2,
                            public_card: self.public_card,
                            pot_pl1: match cur_player {
                                Player::Player1 => self.pot_pl2,
                                Player::Player2 => self.pot_pl1,
                            },
                            pot_pl2: match cur_player {
                                Player::Player1 => self.pot_pl2,
                                Player::Player2 => self.pot_pl1,
                            },
                            num_raises_r1: self.num_raises_r1,
                            num_raises_r2: self.num_raises_r2,
                            last_action_r1: match self.first_action_r1 {
                                // If *first* action is still None, then we do *not*
                                // fill in the last action yet! For call only!
                                None => None,
                                Some(_) => Some(Action::Call),
                            },
                            last_action_r2: self.last_action_r2,
                            first_action_r1: match self.first_action_r1{
                                None => Some(Action::Call),
                                Some(_) => self.first_action_r1,
                            },
                            first_action_r2: self.first_action_r2,
                        }
                    },
                    Phase::Round(1) => { 
                        let cur_player = -Self::last_player_in_round(
                                            self.first_action_r2,
                                            self.last_action_r2, 
                                            self.num_raises_r2);
                        State {
                            config: self.config,
                            card_pl1: self.card_pl1,
                            card_pl2: self.card_pl2,
                            public_card: self.public_card,
                            pot_pl1: match cur_player {
                                Player::Player1 => self.pot_pl2,
                                Player::Player2 => self.pot_pl1,
                            },
                            pot_pl2: match cur_player {
                                Player::Player1 => self.pot_pl2,
                                Player::Player2 => self.pot_pl1,
                            },
                            num_raises_r1: self.num_raises_r1,
                            num_raises_r2: self.num_raises_r2,
                            last_action_r1: self.last_action_r1,
                            last_action_r2: match self.first_action_r2 {
                                // If *first* action is still None, then we do *not*
                                // fill in the last action yet! For call only!
                                None => None,
                                Some(_) => Some(Action::Call),
                            },
                            first_action_r1: self.first_action_r1,
                            first_action_r2: match self.first_action_r2 {
                                None => Some(Action::Call),
                                Some(_) => self.first_action_r2,
                            } 
                        }
                    },
                    _ => { panic!(); }
                }
            }
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
        match self.config.subgame_setting {
            SubgameSetting::SecondRound => 
                match self.current_phase() {
                    Phase::Round(1) => 
                        { 
                        Some(Subgame {
                            public_card: Some(self.public_card),
                            num_raises_r1: self.num_raises_r1,
                            first_action_r1: self.first_action_r1.unwrap(),
                            last_action_r1: Some(self.last_action_r1.unwrap()),
                        })
                        },
                    _ => None,
                },
            SubgameSetting::NthAction(n) => {
                // Figure out the first n actions.
                let num_actions_r1 = Self::num_actions_in_round(self.first_action_r1, 
                                                                self.last_action_r1, 
                                                                self.num_raises_r1);
                if num_actions_r1 >= n {
                    // Get the subgame representation of the first n-th actions.
                    Some(Subgame {
                        public_card: None,
                        num_raises_r1: {
                            let num_calls = {
                                let mut x = 0;
                                if self.first_action_r1.unwrap() == Action::Call {
                                    x += 1;
                                }
                                if self.last_action_r1.is_some() && 
                                   self.last_action_r1.unwrap() == Action::Call && 
                                   num_actions_r1 == n {
                                    x += 1;
                                }
                                x
                            };

                            n - num_calls
                        },
                        first_action_r1: self.first_action_r1.unwrap(),
                        last_action_r1: {
                            if num_actions_r1 == n && 
                               self.last_action_r1.is_some() && 
                               self.last_action_r1.unwrap() == Action::Call {
                                Some(Action::Call)
                            } else {
                                None
                            }
                        }
                    })
                } else {
                    match self.current_phase() {
                        Phase::Round(1) => 
                            // If the second round is reached, then we have the subgame at round 2.
                            Some(Subgame {
                                public_card: Some(self.public_card),
                                num_raises_r1: self.num_raises_r1,
                                first_action_r1: self.first_action_r1.unwrap(),
                                last_action_r1: self.last_action_r1,
                            }),
                        Phase::Round(0) => {
                            // If we are still in the first subgame but not yet encountered n or more actions, then
                            // we are not in a subgame.
                            None 
                        },
                        _ => { None }
                    }
                }
            }
            SubgameSetting::None => Some(self.dummy_subgame()),
        }
    }

}

#[derive(StructOpt, Debug)]
#[structopt(name = "leduc")]
struct Opt {
    // Number of cards *per suit*.
    #[structopt(short = "n", long = "num_cards_per_suit")]
    num_cards: usize,

    // Raise quantities at each round.
    #[structopt(short = "b", long = "bet_sizes", raw(use_delimiter = "true"), default_value="2,4")]
    raise_amounts: Vec<f64>,

    // Initial money *each* player contributes to the pot.
    #[structopt(short = "p", long = "pot_contribution_per_player")]
    pot_contribution_per_player: f64,

    // Maximum number of raises per round. 
    #[structopt(short = "m", long = "max_raises_per_round")]
    max_raises_per_round: usize,

    // Percentage of the pot which is taken by the house.
    #[structopt(short = "r", long = "rake_percentage")]
    rake_percentage: f64,

    // Subgame settings.
    #[structopt(short = "s", long = "subgame_setting")]
    subgame_setting: SubgameSetting,

    // Output file
    #[structopt(short = "o", long = "output_file")]
    output_file: PathBuf,
}

fn main() {

    env_logger::init();

    let opt = Opt::from_args();
    let config = Config::new(opt.num_cards, 
                             &opt.raise_amounts.iter().map(|&x| R64::from_f64(x)).collect::<Vec::<R64>>(), 
                             R64::from_f64(opt.pot_contribution_per_player), 
                             opt.max_raises_per_round, 
                             R64::from_f64(opt.rake_percentage),
                             opt.subgame_setting.clone());

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
        if i == 576 {
            // println!("576: {:?}", s.clone());
        }
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

    let mut file_writer = File::create(&opt.output_file).unwrap();
    efg.persist(&mut file_writer).unwrap();
}