use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub enum TargetKind {
    Player,
    MultiplePlayers(usize),
    Monster,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Target {
    Player(Uuid),
    MultiplePlayers(Vec<Uuid>),
    Monster(Uuid),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
pub enum Card {
    Poison,
    ExtraRations,
    TasteTester,
    PsyBlast,
    Meditation,
    TinfoilHat,
    Nepotism,
    Theft,
    Extortion,
    Stupify,
    Scrutiny,
    Crystals,
}

impl Card {
    pub fn target_kind(&self) -> TargetKind {
        match self {
            Card::Poison => TargetKind::Monster,
            Card::TasteTester => TargetKind::Monster,
            Card::ExtraRations => TargetKind::Monster,
            Card::PsyBlast => TargetKind::Monster,
            Card::TinfoilHat => TargetKind::Monster,
            Card::Nepotism => TargetKind::Monster,
            Card::Meditation => TargetKind::Monster,
            Card::Theft => TargetKind::Player,
            Card::Extortion => TargetKind::Player,
            Card::Stupify => TargetKind::Player,
            Card::Scrutiny => TargetKind::MultiplePlayers(2),
            Card::Crystals => TargetKind::Player,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Card::Poison => "Poison",
            Card::TasteTester => "Taste Tester",
            Card::ExtraRations => "Extra Rations",
            Card::PsyBlast => "Psy Blast",
            Card::TinfoilHat => "Tinfoil Hat",
            Card::Nepotism => "Nepotism",
            Card::Meditation => "Meditation",
            Card::Theft => "Theft",
            Card::Extortion => "Extortion",
            Card::Stupify => "Stupify",
            Card::Scrutiny => "Scrutiny",
            Card::Crystals => "Crystals",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Card::Poison => "/pkg/icons/poison.svg",
            Card::TasteTester => "/pkg/icons/taste-tester.svg",
            Card::ExtraRations => "/pkg/icons/ramen.svg",
            Card::PsyBlast => "/pkg/icons/psyblast.svg",
            Card::TinfoilHat => "/pkg/icons/helmet.svg",
            Card::Nepotism => "/pkg/icons/nepotism.svg",
            Card::Meditation => "/pkg/icons/meditation.svg",
            Card::Theft => "/pkg/icons/theft.svg",
            Card::Extortion => "/pkg/icons/extortion.svg",
            Card::Stupify => "/pkg/icons/stupify.svg",
            Card::Scrutiny => "/pkg/icons/scrutiny.svg",
            Card::Crystals => "/pkg/icons/crystals.svg",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Card::Poison => "-2 Strength",
            Card::TasteTester => "Blocks Strength Effects",
            Card::ExtraRations => "+1 Strength",
            Card::PsyBlast => "-2 Dexterity",
            Card::TinfoilHat => "Blocks Dexterity Effects",
            Card::Nepotism => "Monster starts race 1m ahead",
            Card::Meditation => "+1 Dexterity",
            Card::Theft => "Take 20% of a player's Crystals",
            Card::Extortion => "Take 2 random cards from a player",
            Card::Stupify => "(Free Action) Player must speak loudly and in single syllables",
            Card::Scrutiny => "Up to 2 players cannot play cards this turn",
            Card::Crystals => "(Free Action) Give 1 player 1000 Crystals",
        }
    }

    pub fn victim_description(&self, perpetrator: &str) -> String {
        match self {
            Card::Theft => format!("{perpetrator} stole 20% of your crystals!"),
            Card::Extortion => format!("{perpetrator} stole 2 of your cards!"),
            Card::Stupify => "You must speak loudly and in single syllables!".to_string(),
            Card::Scrutiny => format!("{perpetrator} stopped you from playing cards this turn!"),
            Card::Crystals => format!("{perpetrator} gave you 1000 Crystals!"),
            _ => "N/A".to_owned(),
        }
    }

    pub fn is_free(&self) -> bool {
        match self {
            Card::Crystals => true,
            Card::Stupify => true,
            _ => false,
        }
    }
}
