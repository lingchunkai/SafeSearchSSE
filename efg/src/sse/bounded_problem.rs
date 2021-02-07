use crate::game::{EFGTools, ExtensiveFormGame, SubgameOrFree};

use crate::game::Player;
use crate::sse::BlueprintBr;

use crate::sse::ValueBound;
use crate::treeplex::{SequenceId, Treeplex, TreeplexTools};

/// Creates intermediate specification an MILP problem
/// from the treeplex repressentation. We do all the required calculation
/// of parent-child relationships here.
#[derive(Debug)]
pub struct BoundedProblem {
    /// Modified game.
    pub game: ExtensiveFormGame,

    /// Game treeplex tools.
    pub game_tools: EFGTools,

    /// Follower treeplex tools.
    pub treeplex_follower_tools: TreeplexTools,

    /// Probability mass entering subgame assuming *both* leader and
    /// follower followed the BP strategy. This typically does not sum
    /// to 1.0.
    pub input_mass: f64,

    /// Bounds (possibly none) for each of the follower information sets.
    pub bounds: Vec<(usize, ValueBound)>,

    /// Leaf indices which are important for the purposes of calculation
    /// of the modified objective.
    pub leaves_within_trunk: Vec<usize>,
}

impl BoundedProblem {
    pub fn new(
        game: ExtensiveFormGame,
        input_mass: f64,
        bounds: Vec<(usize, ValueBound)>,
        leaves_within_trunk: Vec<usize>,
    ) -> BoundedProblem {

        let treeplex_follower_tools = TreeplexTools::new(game.treeplex(Player::Player2));
        let game_tools = EFGTools::new(&game);

        BoundedProblem {
            game,
            game_tools,
            treeplex_follower_tools,
            input_mass,
            bounds,
            leaves_within_trunk,
        }
    }
}