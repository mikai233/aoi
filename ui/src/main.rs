use bevy::app::App;
use bevy::DefaultPlugins;
use bevy::input::Input;
use bevy::log::info;
use bevy::math::Vec3;
use bevy::prelude::{Camera2dBundle, Color, Commands, Component, KeyCode, Query, Res, SpriteBundle, Transform, With};
use bevy::render::camera::CameraPlugin;
use bevy::sprite::Sprite;
use bevy::utils::default;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

#[derive(Component, Debug)]
struct Player(i32);

#[derive(Component, Default, Debug)]
struct Location {
    x: i32,
    y: i32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap())
        .add_startup_system(setup)
        .add_system(greet_people)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
    // commands.spawn().insert(Player(1)).insert(Location::default());
    commands.spawn_bundle(SpriteBundle {
        transform: Transform { scale: Vec3::splat(30.), ..default() },
        sprite: Sprite { color: Color::rgb(50., 50., 50.), ..default() },
        ..default()
    }).insert(Player(1));
}

fn greet_people(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Transform, With<Player>>) {
    let mut paddle_transform = query.single_mut();
    let mut current_x = paddle_transform.translation.x;
    let mut current_y = paddle_transform.translation.y;

    if keyboard_input.pressed(KeyCode::Left) {
        current_x -= 10.0;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        current_x += 10.0;
    }

    if keyboard_input.pressed(KeyCode::Up) {
        current_y += 10.0;
    }

    if keyboard_input.pressed(KeyCode::Down) {
        current_y -= 10.0;
    }

    paddle_transform.translation.x = current_x;
    paddle_transform.translation.y = current_y;
    info!("x:{} y:{}",current_x,current_y);
}

fn start_networking(mut commands: Commands, runtime: Res<Runtime>) {
    let (from_server_sender, from_server_receiver) = mpsc::channel::<NetworkMessage>(20);
    let (to_server_sender, mut to_server_receiver) = mpsc::channel::<NetworkMessage>(20);

    runtime.spawn(async move {

    });
}