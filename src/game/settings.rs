#[derive(Debug)]
pub struct Settings {
    pub player_count: usize,
}
impl Settings {
    pub fn new(player_count: usize) -> Self {
        Self { player_count }
    }
}
impl Default for Settings {
    fn default() -> Self {
        Self { player_count: 1 }
    }
}
