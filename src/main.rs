use std::time::Duration;

use avian2d::prelude::*;
use bevy::{prelude::*, window::WindowResolution};
use bevy_asset_loader::prelude::*;
use rand::{Rng, rngs::ThreadRng};
fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Flappy Bird"),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                resolution: WindowResolution::new(280, 500),
                resizable:false,
                ..default()
            }),
            ..default()
        }),
        PhysicsPlugins::default(),
    ));
    app.init_state::<GameState>();
    app.add_loading_state(
        LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::Start)
            .load_collection::<AudioAssets>()
            .load_collection::<ImageAssets>(),
    );
    app.add_systems(OnEnter(GameState::Start), setup_start_screen);
    app.add_systems(Update, start_game.run_if(in_state(GameState::Start)));
    app.add_systems(Update, restart_game.run_if(in_state(GameState::GameOver)));
    app.add_systems(OnEnter(GameState::Playing), setup);
    app.add_systems(OnEnter(GameState::Playing), setup_scoreboard);
    app.add_systems(Update, spawn_pipes.run_if(in_state(GameState::Playing)));
    app.add_systems(
        Update,
        (move_bird, change_sprite)
            .chain()
            .run_if(in_state(GameState::Playing)),
    );
    app.add_systems(
        Update,
        detect_collisions_with_pipes.run_if(in_state(GameState::Playing)),
    );
    app.add_systems(
        Update,
        (detect_collisions_with_passage, update_scoreboard)
            .chain()
            .run_if(in_state(GameState::Playing)),
    );
    app.run();
}
const PIPE_HEIGHT: f32 = 320.0;

const HALF_PIPE_HEIGHT: f32 = PIPE_HEIGHT / 2.0;

const STARTING_POSITION: f32 = 500.0;
const PIPE_SPEED_X: f32 = -100.0;
const SCOREBOARD_FONT_SIZE: f32 = 33.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);
fn setup(mut commands: Commands, audio_assets: Res<AudioAssets>, image_assets: Res<ImageAssets>) {
    commands.insert_resource(CollisionSound(audio_assets.hit.clone()));
    commands.insert_resource(PassSound(audio_assets.pass.clone()));
    commands.insert_resource(PipeSpawnerTimer(Timer::new(
        Duration::from_millis(1500),
        TimerMode::Repeating,
    )));

    commands.insert_resource(PipeSpriteHandle(image_assets.pipe.clone()));
    commands.insert_resource(BirdSpriteHandle {
        up_flap: image_assets.upflap_bird.clone(),
        down_flap: image_assets.downflap_bird.clone(),
        mid_flap: image_assets.midflap_bird.clone(),
    });

    commands.spawn((
        Ground,
        Sprite {
            image: image_assets.base.clone(),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(280.0, 110.0),
        Transform::from_xyz(0.0, -270.0, 10.0),
    ));
    commands.spawn((
        Bird,
        Sprite {
            image: image_assets.downflap_bird.clone(),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::circle(16.0),
        CollisionEventsEnabled,
        Transform::from_xyz(-100.0, 0.0, 0.0),
        GravityScale(20.5),
        DespawnOnExit(GameState::GameOver),
    ));

    let rng = rand::rng();
    spawn_pipe_set(commands,image_assets.pipe.clone(),rng,300.0);
}

fn setup_scoreboard(mut commands: Commands) {

    commands.spawn((
        Text::new("Score: "),
        TextFont {
            font_size: SCOREBOARD_FONT_SIZE,
            ..default()
        },
        TextColor(TEXT_COLOR),
        ScoreboardUi,
        Node {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        },
        children![(
            TextSpan::new("0"),
            TextFont {
                font_size: SCOREBOARD_FONT_SIZE,
                ..default()
            },
            TextColor(SCORE_COLOR),
        )],
        DespawnOnExit(GameState::GameOver),

    ));
}

fn setup_start_screen(mut commands: Commands, image_assets: Res<ImageAssets>) {
    commands.spawn(Camera2d);
    commands.insert_resource(PlayerScore(0));
    commands.spawn((
        Sprite {
            image: image_assets.background_day.clone(),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0), // Put behind everything
    ));
    commands.spawn((
        DespawnOnExit(GameState::Start),
        Sprite {
            image: image_assets.start_messsage.clone(),
            ..default()
        },
    ));
}
fn start_game(
    buttons: Res<ButtonInput<MouseButton>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if buttons.pressed(MouseButton::Left) {
        game_state.set(GameState::Playing);
    }
}

fn restart_game(
    buttons: Res<ButtonInput<MouseButton>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut player_score: ResMut<PlayerScore>,
) {
    if buttons.pressed(MouseButton::Left) {
        game_state.set(GameState::Playing);
        player_score.0 = 0;
    }
}
fn spawn_pipe_set(
    mut commands: Commands,
    pipe_asset:Handle<Image>,
    mut rng:ThreadRng,
    starting_position:f32
){
        let position_of_passage: f32 = rng.random_range(-100.0..=100.0);
        let height_of_passage = rng.random_range(80.0..=150.0);
        let half_passage_height = height_of_passage / 2.0;
        let position_of_first_pipe = position_of_passage - half_passage_height - HALF_PIPE_HEIGHT;
        let position_of_second_pipe = position_of_passage + half_passage_height + HALF_PIPE_HEIGHT;

        commands.spawn((
            Pipe,
            Sprite {
                image: pipe_asset.clone(),
                ..default()
            },
            RigidBody::Kinematic,
            Collider::rectangle(48.0, 320.0),
            Transform::from_xyz(starting_position, position_of_first_pipe, 0.0),
            LinearVelocity(Vec2 {
                x: PIPE_SPEED_X,
                y: 0.0,
            }),
            DespawnOnExit(GameState::GameOver),
        ));
        commands.spawn((
            Pipe,
            Sprite {
                image: pipe_asset,
                flip_y: true,
                ..default()
            },
            RigidBody::Kinematic,
            Collider::rectangle(48.0, 320.0),
            Transform::from_xyz(starting_position, position_of_second_pipe, 0.0),
            LinearVelocity(Vec2 {
                x: PIPE_SPEED_X,
                y: 0.0,
            }),
            DespawnOnExit(GameState::GameOver),
        ));
        commands.spawn((
            Passage,
            RigidBody::Kinematic,
            Collider::rectangle(48.0, height_of_passage),
            Transform::from_xyz(starting_position, position_of_passage, 0.0),
            Sensor,
            LinearVelocity(Vec2 {
                x: PIPE_SPEED_X,
                y: 0.0,
            }),
            DespawnOnExit(GameState::GameOver),
        ));
    
}

fn spawn_pipes(
    time: Res<Time>,
    commands: Commands,
    green_pipe: Res<PipeSpriteHandle>,
    mut pipe_spawner_timer: ResMut<PipeSpawnerTimer>,
) {
    pipe_spawner_timer.tick(time.delta());
    if pipe_spawner_timer.is_finished() {
        let rng = rand::rng();
        spawn_pipe_set(commands,green_pipe.clone(),rng,STARTING_POSITION);
    }
}

fn change_sprite(
    bird_sprite_handle: Res<BirdSpriteHandle>,
    bird: Single<(&LinearVelocity, &mut Sprite), With<Bird>>,
) {
    let (linear_velocity, mut sprite) = bird.into_inner();
    if linear_velocity.y > 0.0 {
        sprite.image = bird_sprite_handle.up_flap.clone();
    } else if linear_velocity.y == 0.0 {
        sprite.image = bird_sprite_handle.mid_flap.clone();
    } else {
        sprite.image = bird_sprite_handle.down_flap.clone();
    }
}

fn move_bird(
    buttons: Res<ButtonInput<MouseButton>>,
    mut bird: Single<&mut LinearVelocity, With<Bird>>,
) {
    if buttons.pressed(MouseButton::Left) {
        bird.y = 70.0;
    }
}

fn detect_collisions_with_pipes(
    mut collision_reader: MessageReader<CollisionStart>,
    bird: Single<Entity, With<Bird>>,
    pipes_query: Query<(), With<Pipe>>,
    mut commands: Commands,
    sound: Res<CollisionSound>,
    image_handles: Res<ImageAssets>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for event in collision_reader.read() {
        let collider1 = event.collider1;
        let collider2 = event.collider2;
        if bird.entity() == collider1 {
            if pipes_query.get(collider2).is_ok() {
                game_state.set(GameState::GameOver);
                commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
                commands.spawn((
                    DespawnOnExit(GameState::GameOver),
                    Sprite {
                        image: image_handles.gameover.clone(),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 20.0),
                ));
            }
        } else if pipes_query.get(collider1).is_ok() {
            game_state.set(GameState::GameOver);

            commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
            commands.spawn((
                DespawnOnExit(GameState::GameOver),
                Sprite {
                    image: image_handles.gameover.clone(),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 20.0),
            ));
        }
    }
}
fn update_scoreboard(
    bird: Single<Entity, With<Bird>>,
    mut collision_reader: MessageReader<CollisionEnd>,
    passage_query: Query<(), With<Passage>>,
    score_root: Single<Entity, (With<ScoreboardUi>, With<Text>)>,
    mut writer: TextUiWriter,
    player_score: Res<PlayerScore>,
) {
    for event in collision_reader.read() {
        let collider1 = event.collider1;
        let collider2 = event.collider2;
        if bird.entity() == collider1 {
            if passage_query.get(collider2).is_ok() {
                *writer.text(*score_root, 1) = player_score.0.to_string();
            }
        } else if passage_query.get(collider1).is_ok() {
            *writer.text(*score_root, 1) = player_score.0.to_string();
        }
    }
}
fn detect_collisions_with_passage(
    mut collision_reader: MessageReader<CollisionEnd>,
    bird: Single<Entity, With<Bird>>,
    passage_query: Query<(), With<Passage>>,
    mut commands: Commands,
    sound: Res<PassSound>,
    mut player_score: ResMut<PlayerScore>,
) {
    for event in collision_reader.read() {
        let collider1 = event.collider1;
        let collider2 = event.collider2;
        if bird.entity() == collider1 {
            if passage_query.get(collider2).is_ok() {
                commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
                player_score.0 += 1;
            }
        } else if passage_query.get(collider1).is_ok() {
            commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
            player_score.0 += 1;
        }
    }
}

#[derive(Component)]
#[require(RigidBody, Collider, CollisionEventsEnabled, Sprite, Transform)]
struct Bird;

#[derive(Component)]
struct ScoreboardUi;

#[derive(Component)]
#[require(RigidBody, Collider, Sprite, LinearVelocity, Transform)]
struct Pipe;

#[derive(Component)]
#[require(RigidBody, Collider, Sprite, Transform)]
struct Ground;

#[derive(Component)]
#[require(RigidBody, Collider, Transform, Sensor)]
struct Passage;

#[derive(Resource, Deref)]
struct PipeSpriteHandle(Handle<Image>);

#[derive(Resource)]
struct BirdSpriteHandle {
    up_flap: Handle<Image>,
    mid_flap: Handle<Image>,
    down_flap: Handle<Image>,
}

#[derive(Resource, Deref, DerefMut)]
struct PipeSpawnerTimer(Timer);

#[derive(Resource, Deref)]
struct CollisionSound(Handle<AudioSource>);

#[derive(Resource, Deref)]
struct PassSound(Handle<AudioSource>);

#[derive(Resource)]
struct PlayerScore(usize);

#[derive(AssetCollection, Resource)]
struct AudioAssets {
    #[asset(path = "sounds/hit.ogg")]
    hit: Handle<AudioSource>,
    #[asset(path = "sounds/point.ogg")]
    pass: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "sprites/bluebird-upflap.png")]
    pub upflap_bird: Handle<Image>,
    #[asset(path = "sprites/bluebird-midflap.png")]
    pub midflap_bird: Handle<Image>,
    #[asset(path = "sprites/bluebird-downflap.png")]
    pub downflap_bird: Handle<Image>,

    #[asset(path = "sprites/pipe-green.png")]
    pub pipe: Handle<Image>,

    #[asset(path = "sprites/background-day.png")]
    pub background_day: Handle<Image>,

    #[asset(path = "sprites/base.png")]
    pub base: Handle<Image>,

    #[asset(path = "sprites/gameover.png")]
    pub gameover: Handle<Image>,

    #[asset(path = "sprites/message.png")]
    pub start_messsage: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Start,
    Playing,
    GameOver,
}
