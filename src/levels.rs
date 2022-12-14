use super::game_spawn::SpawnInstruction;
use bevy::prelude::*;
use shapeshifter_level_maker::util::SpawnLevel;

pub struct UnlockedLevels {
    pub levels: Vec<Level>,
}

pub struct UnlockedCities {
    pub cities: Vec<City>,
}

pub struct CurrentLevel {
    pub level: Level,
}

#[derive(Clone, PartialEq)]
pub enum City {
    Tutorial,
    Simplicity,

    Perplexity,
    Complexity,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Level {
    Tutorial(usize),
    Simplicity(usize),

    Perplexity(usize),
    Complexity(usize),
}

impl Level {
    pub fn tutorial(&mut self, x: usize) {
        *self = Level::Tutorial(x);
    }

    pub fn simplicity(&mut self, x: usize) {
        *self = Level::Simplicity(x);
    }

    pub fn perplexity(&mut self, x: usize) {
        *self = Level::Perplexity(x);
    }
    pub fn complexity(&mut self, x: usize) {
        *self = Level::Complexity(x);
    }
}

pub struct GameLevels {
    pub tutorial: Vec<SpawnLevel>,
    pub simplicity: Vec<SpawnLevel>,

    pub perplexity: Vec<SpawnLevel>,
    pub complexity: Vec<SpawnLevel>,
}

impl GameLevels {
    pub fn get(&self, level: &Level) -> SpawnLevel {
        match level {
            Level::Tutorial(x) => self.tutorial[*x].clone(),
            Level::Simplicity(idx) => self.simplicity[*idx].clone(),

            Level::Perplexity(idx) => self.perplexity[*idx].clone(),
            Level::Complexity(idx) => self.complexity[*idx].clone(),
        }
    }

    pub fn get_total_levels(&self) -> usize {
        self.tutorial.len() + self.simplicity.len() + self.perplexity.len() + self.complexity.len()
    }

    pub fn to_int(&self, level: &Level) -> usize {
        let tut_num = self.tutorial.len();
        let sim_num = self.simplicity.len();
        let per_num = self.perplexity.len();

        // let total = sim_num + con_num + per_num + com_num;

        match level {
            Level::Tutorial(idx) => *idx,
            Level::Simplicity(idx) => *idx + tut_num,
            // Level::Convexity(idx) => sim_num + tut_num + *idx,
            Level::Perplexity(idx) => sim_num + tut_num + *idx,
            Level::Complexity(idx) => sim_num + tut_num + per_num + *idx,
        }
    }
}

pub fn send_tutorial_text(
    tutorial_level: usize,
    spawn_instruction_event_writer: &mut EventWriter<SpawnInstruction>,
) {
    let text = match tutorial_level {
        0 => "The goal is to fit the whole polygon inside the target area",
        1 => "Rotate the polygon using either the right mouse button (up and down) or the scroll wheel",
//         1 => "Rotate the polygon by holding right click, then move away for more precision
// Or scroll over it",
        2 => "Perform a cut by holding C key, and then using the mouse across the whole polygon",
        3 => "There is a \"restart level\" option in the options accessible via the option button or M key",
        4 => "The number of remaining cuts for the level is shown in the top left corner",
        5 => "You are on your own now! Good luck!",
        _ => "",
    };

    if text != "".to_owned() {
        spawn_instruction_event_writer.send(SpawnInstruction {
            text: text.to_string(),
        });
    }
}

// 004_simplicity_square_cut

// SpawnLevel::new4("002_simplicity_square", "tree1", 1, 1.25),

impl Default for GameLevels {
    fn default() -> Self {
        let tutorial = vec![
            // tutorial
            // 0
            SpawnLevel::new3("002_simplicity_square", "002_simplicity_square", 0),
            //
            // 1
            SpawnLevel::new4(
                "002_simplicity_square",
                "003_simplicity_square_oblique",
                0,
                1.05,
            ),
            //
            // 2
            // SpawnLevel::new4("002_simplicity_square", "004_simplicity_square_cut", 4, 1.2),
            SpawnLevel::new4("gege1", "004_simplicity_square_parallel", 3, 1.1),
            //
            // 3
            // SpawnLevel::new4(
            //     "002_simplicity_square",
            //     "004_simplicity_square_parallel",
            //     1,
            //     1.15,
            // ),
            //
            // 4
            // SpawnLevel::new4("seal1", "pear", 3, 1.3),
            SpawnLevel::new4("a", "heart", 1, 1.3), // simplicity
        ];
        // let  = vec![
        // SpawnLevel::new4("a", "spade", 3, 1.155),

        // SpawnLevel::new4("002_simplicity_square", "tree1", 3, 1.25),
        // ];
        let simplicity = vec![
            // SpawnLevel::new4("a", "glass", 3, 1.1),
            // SpawnLevel::new4("t", "turtle1", 2, 1.3),
            SpawnLevel::new4("crab1", "whale1", 3, 1.3),
            SpawnLevel::new4("f", "fish_charles", 1, 1.12),
            SpawnLevel::new4("squirrel1", "bird1", 3, 1.18), // convexity
        ];
        let perplexity = vec![
            // SpawnLevel::new4("giraffe1", "cat2", 3, 1.35),
            SpawnLevel::new4("spade", "p", 2, 1.15), // perplexity
            SpawnLevel::new4("gege_weird", "beaver1", 2, 1.4), // perplexity
            SpawnLevel::new4("squirrel1", "glass", 2, 1.15),
        ];
        let complexity = vec![
            SpawnLevel::new4("cat3", "otter1", 3, 1.2),
            SpawnLevel::new4("gege2", "fox1", 3, 1.1),
        ];

        GameLevels {
            tutorial,
            simplicity,
            // convexity,
            perplexity,
            complexity,
        }
    }
}
