use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;
fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,PhysicsPlugins::default(),PhysicsDebugPlugin));
    app.add_systems(Startup, setup);
    app.add_systems(Update, spawn_pipes);
    app.run();
}
const PASSAGE_HEIGHT:f32=150.0;
const PIPE_HEIGHT:f32=320.0;

const HALF_PASSAGE_HEIGHT:f32=PASSAGE_HEIGHT/2.0;
const HALF_PIPE_HEIGHT:f32=PIPE_HEIGHT/2.0;

const STARTING_POSITION:f32=200.0;
const PIPE_SPEED_X:f32=-200.0;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.insert_resource(PipeSpawnerTimer(Timer::new(Duration::from_secs(2), TimerMode::Repeating)));

    let green_pipe: Handle<Image> = asset_server.load("sprites/pipe-green.png");
    commands.insert_resource(PipeSpriteHandle(green_pipe.clone()));
    let background_day: Handle<Image> = asset_server.load("sprites/background-day.png");
    let bluebird_upflap: Handle<Image> = asset_server.load("sprites/bluebird-upflap.png");
    let bluebird_midflap: Handle<Image> = asset_server.load("sprites/bluebird-midflap.png");
    let bluebird_downflap: Handle<Image> = asset_server.load("sprites/bluebird-upflap.png");
    commands.spawn(Sprite {
        image: background_day,
        custom_size:Some(Vec2{x:1000.0,y:600.0}),
        ..default()
    });
    commands.spawn((
        Bird,
        Sprite {
            image: bluebird_upflap,
            ..default()
        },
        RigidBody::Kinematic,
        Collider::circle(16.0),
        CollisionEventsEnabled,
        Transform::from_xyz(0.0, 0.0, 0.0)
    ));
    
}

fn spawn_pipes(
    time: Res<Time>,
    mut commands: Commands,
    green_pipe: Res<PipeSpriteHandle>,
    mut pipe_spawner_timer: ResMut<PipeSpawnerTimer>
){
    pipe_spawner_timer.tick(time.delta());
    if pipe_spawner_timer.is_finished(){
        let mut rng = rand::rng();
        let position_of_passage:f32=rng.random_range(-100.0..=100.0);
        println!("position of first pipe is {}",position_of_passage);
        
        let position_of_first_pipe=position_of_passage-HALF_PASSAGE_HEIGHT-HALF_PIPE_HEIGHT;
        let position_of_second_pipe=position_of_passage + HALF_PASSAGE_HEIGHT + HALF_PIPE_HEIGHT;
    
        commands.spawn((
            Pipe,
            Sprite {
                image: green_pipe.clone(),
                ..default()
            },
            RigidBody::Kinematic,
            Collider::rectangle(48.0,320.0),
            Transform::from_xyz(STARTING_POSITION, position_of_first_pipe, 0.0),
            LinearVelocity(Vec2{x:PIPE_SPEED_X,y:0.0})
        ));
        commands.spawn((
            Pipe,
            Sprite {
                image: green_pipe.clone(),
                flip_y:true,
                ..default()
            },
            RigidBody::Kinematic,
            Collider::rectangle(48.0,320.0),
            Transform::from_xyz(STARTING_POSITION, position_of_second_pipe, 0.0),
            LinearVelocity(Vec2{x:PIPE_SPEED_X,y:0.0})
    
        ));
        commands.spawn((
            Passage,
            RigidBody::Kinematic,
            Collider::rectangle(48.0,200.0),
            Transform::from_xyz(STARTING_POSITION, position_of_passage, 0.0),
            Sensor,
            LinearVelocity(Vec2{x:PIPE_SPEED_X,y:0.0})
        ));

    }

}

#[derive(Component)]
#[require(RigidBody, Collider, CollisionEventsEnabled, Sprite,Transform)]
struct Bird;

#[derive(Component)]
#[require(RigidBody, Collider, Sprite, LinearVelocity,Transform)]
struct Pipe;

#[derive(Component)]
#[require(RigidBody, Collider,Transform,Sensor)]
struct Passage;

#[derive(Resource,Deref)]
struct PipeSpriteHandle(Handle<Image>);


#[derive(Resource,Deref,DerefMut)]
struct PipeSpawnerTimer(Timer);
