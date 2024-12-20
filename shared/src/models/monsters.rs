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

    pub idle_animation: &'static str,
    pub jump_animation: &'static str,
    pub dance_animation: &'static str,
    pub death_animation: &'static str,

    pub jump_delay: f32,
    pub jump_end: f32,

    pub scale: f32,
}

impl Monster {
    const DEFAULT: Monster = Monster {
        name: "Unnamed",
        blueprint_name: "no_blueprint_set",
        uuid: Uuid::from_u128(0),
        dexterity: 5,
        strength: 5,
        starting_position: 0.0,
        scale: 1.0,
        jump_delay: 0.3,
        jump_end: 0.1,
        idle_animation: "CharacterArmature|Idle",
        jump_animation: "CharacterArmature|Jump",
        dance_animation: "CharacterArmature|Dance",
        death_animation: "CharacterArmature|Death",
    };
}

impl Default for Monster {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub const MONSTERS: [Monster; 11] = [
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
        idle_animation: "RobotArmature|Idle",
        jump_animation: "RobotArmature|Jump",
        dance_animation: "RobotArmature|Dance",
        death_animation: "RobotArmature|Death",
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
        name: "Cranius",
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
        dexterity: 6,
        strength: 3,
        ..Monster::DEFAULT
    },
    Monster {
        name: "Fangmaw",
        uuid: Uuid::from_u128(0xb4775b5b2e1f42debe985d3d7890db0du128),
        blueprint_name: "library/Monster_Yeti.glb",
        dexterity: 5,
        strength: 5,
        ..Monster::DEFAULT
    },
    Monster {
        name: "Dave",
        uuid: Uuid::from_u128(0x331539f0d6934d09b6de219bd3767ede),
        blueprint_name: "library/Monster_Dave.glb",
        dexterity: 0,
        strength: 10,
        jump_animation: "CharacterArmature|Roll",
        dance_animation: "CharacterArmature|Wave",
        scale: 2.5,
        jump_delay: 0.001,
        jump_end: 0.001,
        ..Monster::DEFAULT
    },
    Monster {
        name: "Zombone",
        uuid: Uuid::from_u128(0x766848319d864928b6c4ac3bff3b9897),
        blueprint_name: "library/Monster_Zombie.glb",
        dexterity: 5,
        strength: 5,
        jump_animation: "CharacterArmature|Crawl",
        dance_animation: "CharacterArmature|HitReact",
        scale: 2.5,
        jump_delay: 0.05,
        jump_end: 0.0001,
        ..Monster::DEFAULT
    },
];
