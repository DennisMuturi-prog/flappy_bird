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
    app.add_systems(Update, spawn_pipes);
    app.add_systems(Update, move_bird);
    app.run();
}
const PASSAGE_HEIGHT: f32 = 150.0;
const PIPE_HEIGHT: f32 = 320.0;

const HALF_PASSAGE_HEIGHT: f32 = PASSAGE_HEIGHT / 2.0;
const HALF_PIPE_HEIGHT: f32 = PIPE_HEIGHT / 2.0;

const STARTING_POSITION: f32 = 500.0;
const PIPE_SPEED_X: f32 = -150.0;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
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
        GravityScale(1.3),
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
fn move_bird_1(
    buttons: Res<ButtonInput<MouseButton>>,
    mut bird_forces: Single<Forces, With<Bird>>,
) {
    if buttons.pressed(MouseButton::Left) {
        println!("i was pressed");

        bird_forces.apply_force(Vec2::new(0.0, 100500.0));
    }
}

fn move_bird(
    buttons: Res<ButtonInput<MouseButton>>,
    mut bird: Single<(&mut Transform, &mut Sprite,&mut LinearVelocity), With<Bird>>,
    bird_sprite_handle: Res<BirdSpriteHandle>,
    time: Res<Time>,
) {
    if buttons.pressed(MouseButton::Left) {
        println!("i was pressed {} {}",bird.2.x,bird.2.y);
        println!("i was calculating {}",time.delta_secs()*450.0);
        bird.0.translation.y += 450.0*(time.delta_secs());
        bird.1.image = bird_sprite_handle.mid_flap.clone();
    }
}

fn move_bird_2(
    buttons: Res<ButtonInput<MouseButton>>,
    mut bird_velocity: Single<&mut LinearVelocity, With<Bird>>,
    time: Res<Time>,
) {
    if buttons.pressed(MouseButton::Left) {
        println!("i was pressed {} {}",bird_velocity.x,bird_velocity.y);

        bird_velocity.y += 2.0;
        
    }
}

#[derive(Component)]
#[require(RigidBody, Collider, CollisionEventsEnabled, Sprite, Transform)]
struct Bird;

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
