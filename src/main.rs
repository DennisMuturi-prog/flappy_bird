use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;
fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        PhysicsPlugins::default(),
        PhysicsDebugPlugin,
    ));
    app.add_systems(Startup, setup);
    app.add_systems(Startup, setup_scoreboard);
    app.add_systems(Update, spawn_pipes);
    app.add_systems(Update, move_bird);
    app.add_systems(Update, detect_collisions_with_pipes);
    app.add_systems(Update, (detect_collisions_with_passage,update_scoreboard).chain());
    app.run();
}
const PASSAGE_HEIGHT: f32 = 150.0;
const PIPE_HEIGHT: f32 = 320.0;

const HALF_PASSAGE_HEIGHT: f32 = PASSAGE_HEIGHT / 2.0;
const HALF_PIPE_HEIGHT: f32 = PIPE_HEIGHT / 2.0;

const STARTING_POSITION: f32 = 500.0;
const PIPE_SPEED_X: f32 = -150.0;
const SCOREBOARD_FONT_SIZE: f32 = 33.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    let bird_collision_sound = asset_server.load("sounds/hit.ogg");
    let bird_pass_sound = asset_server.load("sounds/point.ogg");
    commands.insert_resource(CollisionSound(bird_collision_sound));
    commands.insert_resource(PassSound(bird_pass_sound));
    commands.insert_resource(PipeSpawnerTimer(Timer::new(
        Duration::from_millis(1500),
        TimerMode::Repeating,
    )));

    let green_pipe: Handle<Image> = asset_server.load("sprites/pipe-green.png");
    commands.insert_resource(PipeSpriteHandle(green_pipe.clone()));
    let background_day: Handle<Image> = asset_server.load("sprites/background-day.png");
    let bluebird_upflap: Handle<Image> = asset_server.load("sprites/bluebird-upflap.png");
    let bluebird_midflap: Handle<Image> = asset_server.load("sprites/bluebird-midflap.png");
    let bluebird_downflap: Handle<Image> = asset_server.load("sprites/bluebird-upflap.png");
    commands.insert_resource(BirdSpriteHandle {
        up_flap: bluebird_upflap,
        down_flap: bluebird_downflap.clone(),
        mid_flap: bluebird_midflap,
    });
    commands.spawn(Sprite {
        image: background_day,
        custom_size: Some(Vec2 {
            x: 1000.0,
            y: 600.0,
        }),
        ..default()
    });
    commands.spawn((
        Bird,
        Sprite {
            image: bluebird_downflap,
            ..default()
        },
        RigidBody::Dynamic,
        Collider::circle(16.0),
        CollisionEventsEnabled,
        Transform::from_xyz(0.0, 0.0, 0.0),
        GravityScale(1.5),
    ));
}

fn setup_scoreboard(mut commands: Commands) {
    commands.insert_resource(PlayerScore(0));

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
    ));
}

fn spawn_pipes(
    time: Res<Time>,
    mut commands: Commands,
    green_pipe: Res<PipeSpriteHandle>,
    mut pipe_spawner_timer: ResMut<PipeSpawnerTimer>,
) {
    pipe_spawner_timer.tick(time.delta());
    if pipe_spawner_timer.is_finished() {
        let mut rng = rand::rng();
        let position_of_passage: f32 = rng.random_range(-100.0..=100.0);
        let position_of_first_pipe = position_of_passage - HALF_PASSAGE_HEIGHT - HALF_PIPE_HEIGHT;
        let position_of_second_pipe = position_of_passage + HALF_PASSAGE_HEIGHT + HALF_PIPE_HEIGHT;

        commands.spawn((
            Pipe,
            Sprite {
                image: green_pipe.clone(),
                ..default()
            },
            RigidBody::Kinematic,
            Collider::rectangle(48.0, 320.0),
            Transform::from_xyz(STARTING_POSITION, position_of_first_pipe, 0.0),
            LinearVelocity(Vec2 {
                x: PIPE_SPEED_X,
                y: 0.0,
            }),
        ));
        commands.spawn((
            Pipe,
            Sprite {
                image: green_pipe.clone(),
                flip_y: true,
                ..default()
            },
            RigidBody::Kinematic,
            Collider::rectangle(48.0, 320.0),
            Transform::from_xyz(STARTING_POSITION, position_of_second_pipe, 0.0),
            LinearVelocity(Vec2 {
                x: PIPE_SPEED_X,
                y: 0.0,
            }),
        ));
        commands.spawn((
            Passage,
            RigidBody::Kinematic,
            Collider::rectangle(48.0, PASSAGE_HEIGHT),
            Transform::from_xyz(STARTING_POSITION, position_of_passage, 0.0),
            Sensor,
            LinearVelocity(Vec2 {
                x: PIPE_SPEED_X,
                y: 0.0,
            }),
        ));
    }
}

fn move_bird(
    buttons: Res<ButtonInput<MouseButton>>,
    mut bird: Single<&mut Transform, With<Bird>>,
    time: Res<Time>,
) {
    if buttons.pressed(MouseButton::Left) {
        bird.translation.y += 500.0 * (time.delta_secs());
    }
}

fn detect_collisions_with_pipes(
    mut collision_reader: MessageReader<CollisionStart>,
    bird: Single<Entity, With<Bird>>,
    pipes_query: Query<(), With<Pipe>>,
    mut commands: Commands,
    sound: Res<CollisionSound>,
) {
    for event in collision_reader.read() {
        let collider1 = event.collider1;
        let collider2 = event.collider2;
        if bird.entity() == collider1 {
            if pipes_query.get(collider2).is_ok() {
                commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
            }
        } else if pipes_query.get(collider1).is_ok() {
            commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
        }
    }
}
fn update_scoreboard(
    bird: Single<Entity, With<Bird>>,
    mut collision_reader: MessageReader<CollisionEnd>,
    passage_query: Query<(), With<Passage>>,
    score_root: Single<Entity, (With<ScoreboardUi>, With<Text>)>,
    mut writer: TextUiWriter,
    player_score: Res<PlayerScore>
){
    for event in collision_reader.read() {
        let collider1 = event.collider1;
        let collider2 = event.collider2;
        if bird.entity() == collider1 {
            if passage_query.get(collider2).is_ok() {
                *writer.text(*score_root,1 ) = player_score.0.to_string();

            }
        } else if passage_query.get(collider1).is_ok() {
            *writer.text(*score_root,1 ) = player_score.0.to_string();
            
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
