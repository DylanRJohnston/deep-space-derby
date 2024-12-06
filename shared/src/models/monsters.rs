use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Monster {
    pub name: &'static str,
    pub blueprint_name: &'static str,
    pub uuid: Uuid,
    // How fast they jump
    pub dexterity: i32,
    // How far they jump
    pub strength: i32,

    pub starting_position: f32,
}

impl Monster {
    const DEFAULT: Monster = Monster {
        name: "Unnamed",
        blueprint_name: "no_blueprint_set",
        uuid: Uuid::from_u128(0),
        dexterity: 5,
        strength: 5,
        starting_position: 0.0,
    };
}

impl Default for Monster {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub const MONSTERS: [Monster; 9] = [
    Monster {
        name: "Cactoro",
        uuid: Uuid::from_u128(0xb19768d8fce94b66a2d7ea84799c0101u128),
        blueprint_name: "library/Monster_Cactoro.glb",
        dexterity: 5,
        strength: 4,
        ..Monster::DEFAULT
    },
    Monster {
        name: "Purglehorn",
        uuid: Uuid::from_u128(0x99a7c5d8c06744eeb856df9d6b04c4e8u128),
        blueprint_name: "library/Monster_Alien.glb",
        dexterity: 5,
        strength: 5,
        ..Monster::DEFAULT
    },
    Monster {
        name: "Mawshroom",
        uuid: Uuid::from_u128(0xf8a2f4560fa44e89b915f0b0de101a1au128),
        blueprint_name: "library/Monster_Mushnub.glb",
        dexterity: 1,
        strength: 9,
        ..Monster::DEFAULT
    },
    Monster {
        name: "Mechapanda",
        uuid: Uuid::from_u128(0x0ef5f3373cea4c9ca6655bd3e7bc4c63u128),
        blueprint_name: "library/Monster_Mech.glb",
        dexterity: 4,
        strength: 7,
        ..Monster::DEFAULT
    },
    Monster {
        name: "Finflare",
        uuid: Uuid::from_u128(0x6cb10197a7234cf980f7fb957f7eb9f1u128),
        blueprint_name: "library/Monster_Fish.glb",
        dexterity: 7,
        strength: 3,
        ..Monster::DEFAULT
    },
    Monster {
        name: "Green Spiky Thing",
        uuid: Uuid::from_u128(0xcbde634a2d3648f383b3c7e45cc864b7u128),
        blueprint_name: "library/Monster_Green_Spiky.glb",
        dexterity: 3,
        strength: 7,
        ..Monster::DEFAULT
    },
    Monster {
        name: "Gallus Cranium",
        uuid: Uuid::from_u128(0x73c68289e1334859a0f4e45883076e10u128),
        blueprint_name: "library/Monster_Pink_Slime.glb",
        dexterity: 10,
        strength: 0,
        ..Monster::DEFAULT
    },
    Monster {
        name: "Cluckerhead",
        uuid: Uuid::from_u128(0x9f987f8ff320446e8930740aca46954fu128),
        blueprint_name: "library/Monster_Chicken.glb",
        dexterity: 7,
        strength: 2,
        ..Monster::DEFAULT
    },
    Monster {
        name: "Fangmaw",
        uuid: Uuid::from_u128(0xb4775b5b2e1f42debe985d3d7890db0du128),
        blueprint_name: "library/Monster_Yeti.glb",
        dexterity: 8,
        strength: 1,
        ..Monster::DEFAULT
    },
];
