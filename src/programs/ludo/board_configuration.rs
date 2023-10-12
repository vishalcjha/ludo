#[derive(Debug, Clone)]
pub struct BoardConfiguration {
    pub vertices: Vec<f32>,
    pub indices: Vec<u16>,
    pub colors: Vec<f32>,
    pub start_index_dice: i32,
    pub end_index_dice: i32,
}
