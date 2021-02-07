use std::ops::Neg;

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd)]
pub enum Player {
    Player1,
    Player2,
}
impl Neg for Player {
    type Output = Player;
    fn neg(self) -> Self::Output {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }
}