use crate::prelude::*;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_header_ui)
            .add_startup_system(spawn_player_ui)
            .add_system(spawn_level_up_ui.in_schedule(OnEnter(GameState::LevelUp)))
            .add_system(despawn_level_up_ui.in_schedule(OnExit(GameState::LevelUp)))
            .add_system(button_system)
            .add_system(update_world_text)
            .add_system(player_health_ui_sync)
            .add_system(player_exp_ui_sync);
    }
}

fn update_world_text(
    mut commands: Commands,
    mut text: Query<(Entity, &mut Style, &mut WorldTextUI)>,
    main_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    render_camera: Query<&Camera, With<FinalCamera>>,
    time: Res<Time>,
) {
    //AHHH
    let (camera, transform) = main_camera.single();
    let final_camera = render_camera.single();

    for (entity, mut style, mut world_ui) in &mut text {
        world_ui.lifetime.tick(time.delta());
        if world_ui.lifetime.just_finished() {
            commands.entity(entity).despawn_recursive();
        }

        world_ui.position = world_ui.position + world_ui.velocity * time.delta_seconds();

        if let Some(coords) = camera.world_to_viewport(transform, world_ui.position.extend(0.0)) {
            let mut coords = coords / Vec2::new(RENDER_WIDTH, RENDER_HEIGHT)
                * final_camera.logical_viewport_size().unwrap();
            coords.y = final_camera.logical_viewport_size().unwrap().y - coords.y;

            style.position = UiRect {
                top: Val::Px(coords.y),
                left: Val::Px(coords.x),
                bottom: Val::Px(coords.y),
                right: Val::Px(coords.x),
            }
        }
    }
}

pub fn spawn_world_text(commands: &mut Commands, assets: &AssetServer, position: Vec2, text: &str) {
    let font = assets.load("fonts/pointfree.ttf");

    //Gross offset because text is at top left of given coords
    let position = position + Vec2::new(-0.2, 0.7);

    let parent = (
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(20.0), Val::Percent(20.0)),
                position_type: PositionType::Absolute,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            z_index: ZIndex::Global(-100),
            ..default()
        },
        WorldTextUI {
            lifetime: Timer::from_seconds(0.5, TimerMode::Once),
            velocity: Vec2::new(0.15, 1.0),
            position,
        },
    );

    let text = TextBundle::from_section(
        text,
        TextStyle {
            font,
            font_size: 32.0,
            color: Color::rgb(0.9, 0.8, 0.8),
        },
    );

    commands.spawn(parent).with_children(|commands| {
        commands.spawn(text);
    });
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &WeaponUpgrade),
        With<Button>,
    >,
    mut upgrade_event: EventWriter<UpgradeSelected>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color, weapon) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = Color::RED.into();
                next_state.set(GameState::Gameplay);
                upgrade_event.send(UpgradeSelected(weapon.clone()));
            }
            Interaction::Hovered => {
                *color = Color::GREEN.into();
            }
            Interaction::None => {
                *color = Color::DARK_GREEN.into();
            }
        }
    }
}

fn despawn_level_up_ui(mut commands: Commands, ui: Query<Entity, With<LevelUpUI>>) {
    for ui in &ui {
        commands.entity(ui).despawn_recursive();
    }
}

fn spawn_level_up_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let level_up_parent = (
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        },
        LevelUpUI,
    );

    let level_up_popup = NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(80.0), Val::Percent(70.0)),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceAround,
            ..default()
        },
        background_color: Color::DARK_GRAY.into(),
        ..default()
    };

    commands.spawn(level_up_parent).with_children(|commands| {
        commands.spawn(level_up_popup).with_children(|commands| {
            spawn_button(commands, &asset_server, &WeaponUpgrade::CloseShot);
            spawn_button(commands, &asset_server, &WeaponUpgrade::AreaShot);
            spawn_button(commands, &asset_server, &WeaponUpgrade::Whip);
        });
    });
}

fn spawn_button(
    commands: &mut ChildBuilder,
    asset_server: &AssetServer,
    weapon: &WeaponUpgrade,
) -> Entity {
    let font = asset_server.load("fonts/pointfree.ttf");
    let button = (
        ButtonBundle {
            style: Style {
                size: Size::new(Val::Percent(50.0), Val::Percent(15.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                ..default()
            },
            background_color: Color::CRIMSON.into(),
            ..default()
        },
        weapon.clone(),
    );

    let text = weapon.name();

    let button_text = TextBundle::from_section(
        text,
        TextStyle {
            font,
            font_size: 40.0,
            color: Color::rgb(0.9, 0.9, 0.9),
        },
    );
    commands
        .spawn(button)
        .with_children(|commands| {
            commands.spawn(button_text);
        })
        .id()
}

fn player_health_ui_sync(mut ui: Query<&mut Style, With<HealthUI>>, player: Query<&Player>) {
    let mut style = ui.single_mut();
    let player = player.single();

    let percent = player.health / player.max_health;
    style.size.width = Val::Percent(percent * 100.0);
}

fn player_exp_ui_sync(mut ui: Query<&mut Style, With<ExpUI>>, player: Query<&Player>) {
    let mut style = ui.single_mut();
    let player = player.single();

    let percent = player.exp as f32 / player.next_level_exp as f32;
    style.size.width = Val::Percent(percent * 100.0);
}

fn spawn_header_ui(mut commands: Commands) {
    let parent_node = (
        NodeBundle {
            style: Style {
                //XXX using Px here because UI isn't based on camera size, just window size
                size: Size::new(Val::Percent(100.0), Val::Percent(10.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Row,
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: BackgroundColor(Color::DARK_GREEN),
            ..default()
        },
        HeaderBarUI,
        Name::new("Header Bar UI"),
    );

    let exp_node = (
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(0.0), Val::Percent(100.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::BLUE),
            ..default()
        },
        ExpUI,
        Name::new("Exp UI"),
    );

    commands.spawn(parent_node).with_children(|commands| {
        commands.spawn(exp_node);
    });
}

fn spawn_player_ui(mut commands: Commands) {
    let parent_node = (
        NodeBundle {
            style: Style {
                //XXX using Px here because UI isn't based on camera size, just window size
                size: Size::new(Val::Percent(5.0), Val::Percent(2.0)),
                position: UiRect {
                    //Player is always centered
                    left: Val::Percent(47.5),
                    right: Val::Auto,
                    top: Val::Percent(55.0),
                    bottom: Val::Auto,
                },
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Row,
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: BackgroundColor(Color::BLACK),
            ..default()
        },
        PlayerUI,
        Name::new("Player UI"),
    );

    let health_node = (
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(0.0), Val::Percent(100.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::RED),
            ..default()
        },
        HealthUI,
        Name::new("Health UI"),
    );

    commands.spawn(parent_node).with_children(|commands| {
        commands.spawn(health_node);
    });
}
