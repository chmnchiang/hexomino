use std::cell::Cell;

use crate::game::state::Player;

struct Me(Cell<Option<Player>>);

trait PlayerExt {
    fn is_me() -> bool;
}

//impl VPlayer {
    //pub fn new(player: Player, me: Player) -> Self {
        //if player == me { VPlayer::Me } else { VPlayer::They }
    //}

    //pub fn is_me(player: Player, me: Pl
        //if player == me { VPlayer::Me } else { VPlayer::They }
//}
