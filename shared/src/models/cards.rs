use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub enum TargetKind {
    Player,
    Monster,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum Target {
    Player(Uuid),
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
}

impl Card {
    pub fn target_kind(&self) -> TargetKind {
        match self {
            Card::Poison => TargetKind::Monster,
            Card::TasteTester => TargetKind::Monster,
            Card::ExtraRations => TargetKind::Monster,
            Card::PsyBlast => TargetKind::Monster,
            Card::TinfoilHat => TargetKind::Monster,
            Card::Meditation => TargetKind::Monster,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Card::Poison => "Poison",
            Card::TasteTester => "Taste Tester",
            Card::ExtraRations => "Extra Rations",
            Card::PsyBlast => "Psy Blast",
            Card::TinfoilHat => "Tinfoil Hat",
            Card::Meditation => "Meditation",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Card::Poison => "/pkg/icons/poison.svg",
            Card::TasteTester => "/pkg/icons/taste-tester.svg",
            Card::ExtraRations => "/pkg/icons/ramen.svg",
            Card::PsyBlast => "/pkg/icons/psyblast.svg",
            Card::TinfoilHat => "/pkg/icons/helmet.svg",
            Card::Meditation => "/pkg/icons/meditation.svg",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Card::Poison => "-3 Strength",
            Card::TasteTester => "Blocks Poison & Extra Rations",
            Card::ExtraRations => "+2 Strength",
            Card::PsyBlast => "-3 Speed",
            Card::TinfoilHat => "Blocks Psy Blast & Meditation",
            Card::Meditation => "+2 Speed",
        }
    }
}
