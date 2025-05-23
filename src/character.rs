use crate::location::Species;
use rand::Rng;

pub struct Character {
    pub name: String,
    pub species: Species,
    pub class: String,
    pub health: i32,
    pub max_health: i32,
    pub attack: i32,
    pub dodge: i32,
    pub luck: i32,
    pub xp: i32,
}

impl Character {
    pub fn new(
        name: String,
        class: String,
        species: Species,
        health: i32,
        max_health: i32,
        attack: i32,
        dodge: i32,
        luck: i32,
    ) -> Self {
        Character {
            name: name.to_string(),
            species,
            class: class.to_string(),
            health,
            max_health,
            attack,
            dodge,
            luck,
            xp: 0,
        }
    }

    pub fn create_human(name: String) -> Self {
        Self::new(
            name,
            "Adventurer".to_string(),
            Species::Human,
            100,
            100,
            10,
            10,
            5,
        )
    }

    pub fn create_random() -> Self {
        let mut rng = rand::thread_rng();
        
        let classes = vec!["Warrior", "Mage", "Rogue", "Cleric"];
        let class = classes[rng.gen_range(0..classes.len())].to_string();
        
        let health = rng.gen_range(80..120);
        let attack = rng.gen_range(8..12);
        let dodge = rng.gen_range(8..12);
        let luck = rng.gen_range(1..10);

        Self::new(
            "Player".to_string(),
            class,
            Species::Human,
            health,
            health,
            attack,
            dodge,
            luck,
        )
    }

    pub fn stats(&self) -> String {
        format!(
            "{} hp: {} attack: {} dodge: {} luck: {} xp: {}",
            self.class, self.health, self.attack, self.dodge, self.luck, self.xp
        )
    }

    pub fn damage(&mut self, damage_amount: i32) {
        self.health -= damage_amount;
        self.xp += 2;
    }

    pub fn heal(&mut self, heal_amount: i32) {
        self.health += heal_amount;
        self.xp += 1;
    }

    pub fn attack(&self) -> i32 {
        self.attack + self.luck / 2 + self.xp / 10
    }

    pub fn dodge(&self) -> i32 {
        self.dodge + self.luck / 2 + self.xp / 10
    }
}