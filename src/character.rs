pub struct Character{
    pub name: String,
    pub class: String,
    pub health: i32,
    pub attack: i32,
    pub dodge: i32,
    pub luck: i32,
    pub xp: i32,
}

pub trait Player {
    fn new(
        name: String,
        class: String,
        health: i32,
        attack: i32,
        dodge: i32,
        luck: i32,
    ) -> Character;

    fn select(&self, player_name: String) -> Self;
    fn damage(&mut self, damage_amount: i32);
    fn heal(&mut self, heal_amount: i32);
    fn attack(&self, heal_amount: i32);
    fn dodge(&self, dodge_amount: i32);
    fn stats(&self) -> String;
}

impl Player for Character {
    fn new(
        name: String,
        class: String,
        health: i32,
        attack: i32,
        dodge: i32,
        luck: i32,
    ) -> Character {
        Character {
            name: name.to_string(),
            class: class.to_string(),
            health: health,
            attack: attack,
            dodge: dodge,
            luck: luck,
            xp: 0,
        }
    }

    fn select(&self, player_name: String) -> Self {
        Self::new(
            player_name,
            self.class.to_string(),
            self.health,
            self.attack,
            self.dodge,
            self.luck,
        )
    }

    fn damage(&mut self, damage_amount: i32) {
        self.health -= damage_amount;
        self.xp += 2;
    }

    fn heal(&mut self, heal_amount: i32) {
        self.health += heal_amount;
        self.xp += 1;
    }

    fn attack(&self, attack_amount: i32) {
        self.xp + self.attack + self.luck / 2;
    }

    fn dodge(&self, dodge_amount: i32) {
        self.xp + self.dodge + self.luck / 2;
    }

    fn stats(&self) -> String {
        format!(
            "{} hp: {} attack: {} dodge: {} luck: {} xp: {}",
            self.class, self.health, self.attack, self.dodge, self.luck, self.xp
        )
    }
}