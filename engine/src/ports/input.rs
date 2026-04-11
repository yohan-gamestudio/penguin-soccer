use crate::domain::action::Action;
use crate::domain::types::PlayerId;

pub trait InputPort {
    fn submit_action(&mut self, player_id: PlayerId, action: Action) -> Result<(), String>;
}
