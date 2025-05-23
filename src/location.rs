use rand::Rng;
use std::fmt;

#[derive(Debug,Copy, Clone, PartialEq)]
pub enum Species {
    Human,
    Orc,
    Elf,
    Cat,
    Rat,
    Bee,
    Bear,
    Ghost,
}
impl fmt::Display for Species {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Species::Human => write!(f, "Human"),
            Species::Orc => write!(f, "Orc"),
            Species::Elf => write!(f, "Elf"),
            Species::Cat => write!(f, "Cat"),
            Species::Rat => write!(f, "Rat"),
            Species::Bee => write!(f, "Bee"),
            Species::Bear => write!(f, "Bear"),
            Species::Ghost => write!(f, "Ghost"),
        }
    }
}
#[derive(Debug,Copy,  Clone, PartialEq)]
pub enum Governance {
    Monarchy,
    Democracy,
    Theocracy,
    Anarchy,
    Hivemind,
    Council,
}

#[derive(Debug,Copy, Clone, PartialEq)]
pub enum LocationState {
    Thriving,
    Struggling,
    Abandoned,
    Ruins,
    Cursed,
    Sacred,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Industry {
    Farming,
    Mining,
    Lumber,
    Fishing,
    Trading,
    Crafting,
    Foraging,
    Hunting,
    Research,
    
}
impl fmt::Display for Governance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Governance::Monarchy => write!(f, "Monarchic"),
            Governance::Democracy => write!(f, "Democratic"),
            Governance::Theocracy => write!(f, "Theocratic"),
            Governance::Anarchy => write!(f, "Anarchic"),
            Governance::Hivemind => write!(f, "Hivemind"),
            Governance::Council => write!(f, "Council"),
        }
    }
}

impl Industry {
    pub fn get_description(&self) -> &str {
        match self {
            Industry::Farming => "vast fields of crops surround the settlement",
            Industry::Mining => "the sound of pickaxes echoes from deep mines",
            Industry::Lumber => "massive lumber mills process ancient trees",
            Industry::Fishing => "fishing boats dot the nearby waters",
            Industry::Trading => "merchants haggle in busy marketplaces",
            Industry::Crafting => "skilled artisans work in numerous workshops",
            Industry::Foraging => "foragers gather rare herbs and plants",
            Industry::Hunting => "hunters prepare for their next expedition",
            Industry::Research => "scholars debate in marble halls",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Location {
    pub name: String,
    pub species: Species,
    pub governance: Governance,
    pub state: LocationState,
    pub size: usize,
    pub industry: Industry,  // Add this field

}

impl Location {
    pub fn generate_description(&self) -> String {
        format!(
            "A {} {} settlement of {}s under {} rule, where {}.",
            self.state.to_string().to_lowercase(),
            self.size_description(),
            self.species,
            self.governance,
            self.industry.get_description()
        )
    }

    fn size_description(&self) -> &str {
        match self.size {
            0..=50 => "tiny",
            51..=200 => "small",
            201..=500 => "medium",
            _ => "large",
        }
    }
}