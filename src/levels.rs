// use bevy::prelude::*;
use shapeshifter_level_maker::util::SpawnLevel;

pub struct GameLevels {
    pub simplicity: Vec<SpawnLevel>,
    pub convexity: Vec<SpawnLevel>,
    pub perplexity: Vec<SpawnLevel>,
    pub complexity: Vec<SpawnLevel>,
}

// 004_simplicity_square_cut

impl Default for GameLevels {
    fn default() -> Self {
        let simplicity = vec![
            //
            //
            SpawnLevel::new2("002_simplicity_square", "002_simplicity_square"),
            //
            //
            SpawnLevel::new2("003_simplicity_square_oblique", "004_simplicity_rectangle"),
            //
            //
            SpawnLevel::new2("004_simplicity_square_cut", "005_simplicity_cactus"),
        ];
        let convexity = Vec::new();
        let perplexity = Vec::new();
        let complexity = Vec::new();

        GameLevels {
            simplicity,
            convexity,
            perplexity,
            complexity,
        }
    }
}
