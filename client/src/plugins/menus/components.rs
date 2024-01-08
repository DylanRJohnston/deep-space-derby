use bevy::prelude::*;

use crate::plugins::pallet;

pub fn full_screen_container() -> NodeBundle {
    NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Start,
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn button() -> ButtonBundle {
    ButtonBundle {
        style: Style {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        background_color: pallet::BLUE_3.into(),
        ..default()
    }
}
