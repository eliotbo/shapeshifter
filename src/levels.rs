// use bevy::prelude::*;
use super::game::CurrentLevel;
use shapeshifter_level_maker::util::SpawnLevel;

pub struct GameLevels {
    pub simplicity: Vec<SpawnLevel>,
    pub convexity: Vec<SpawnLevel>,
    pub perplexity: Vec<SpawnLevel>,
    pub complexity: Vec<SpawnLevel>,
}

impl GameLevels {
    pub fn get(&self, level: &CurrentLevel) -> SpawnLevel {
        match level {
            CurrentLevel::Simplicity(idx) => self.simplicity[*idx].clone(),
            CurrentLevel::Convexity(idx) => self.convexity[*idx].clone(),
            CurrentLevel::Perplexity(idx) => self.perplexity[*idx].clone(),
            CurrentLevel::Complexity(idx) => self.complexity[*idx].clone(),
        }
    }
}

// 004_simplicity_square_cut

impl Default for GameLevels {
    fn default() -> Self {
        let simplicity = vec![
            //
            //
            SpawnLevel::new3("002_simplicity_square", "002_simplicity_square", 0),
            //
            //
            SpawnLevel::new3("002_simplicity_square", "003_simplicity_square_oblique", 3),
            //
            SpawnLevel::new4(
                "002_simplicity_square",
                "004_simplicity_square_cut",
                3,
                1.15,
            ),
            //
            SpawnLevel::new4(
                "002_simplicity_square",
                "004_simplicity_square_parallel",
                3,
                1.15,
            ),
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
