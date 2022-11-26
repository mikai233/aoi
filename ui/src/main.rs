use std::time::Duration;

use bevy::app::App;
use bevy::DefaultPlugins;
use bevy::input::Input;
use bevy::log::{error, info};
use bevy::math::Vec3;
use bevy::prelude::{Camera2dBundle, Color, Commands, Component, KeyCode, Query, Res, ResMut, SpriteBundle, Timer, Transform, With, Without};
use bevy::sprite::Sprite;
use bevy::time::Time;
use bevy::utils::default;
use futures::SinkExt;
use futures::StreamExt;
use protobuf::{EnumOrUnknown, MessageDyn, MessageField, MessageFull};
use rand::{random, thread_rng};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio_util::codec::Framed;

use protocol::codec::ProtoCodec;
use protocol::mapper::kcp_config;
use protocol::test::{LoginReq, LoginResp, PlayerMoveNotify, SCOtherPlayersStateNotify, SCPlayerEnterNotify, SCPlayerMoveNotify};

type ProtoMessage = Box<dyn MessageDyn>;

#[derive(Component)]
struct FuseTime {
    timer: Timer,
}

#[derive(Component, Debug)]
struct Player(i32);

#[derive(Component)]
struct SelfFlag;

#[derive(Component, Default, Debug)]
struct Location {
    pub x: f64,
    pub y: f64,
    pub state: State,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap())
        .add_startup_system(setup)
        .add_startup_system(start_networking)
        .add_system(handle_keyboard_cmd)
        .add_system(handle_server_msg)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands.insert_resource(FuseTime {
        timer: Timer::new(Duration::from_millis(100), true),
    });
    // commands.spawn_bundle(SpriteBundle {
    //     transform: Transform { scale: Vec3::splat(30.), ..default() },
    //     sprite: Sprite { color: Color::rgb(50., 50., 50.), ..default() },
    //     ..default()
    // }).insert(Player(player_id));
}

fn handle_server_msg(mut commands: Commands, mut from_server: ResMut<mpsc::Receiver<ProtoMessage>>, self_player: Query<&Player, With<SelfFlag>>, mut others: Query<(&mut Transform, &Player), Without<SelfFlag>>) {
    let mut player_id = 0;
    while let Ok(message) = from_server.try_recv() {
        let desc = message.descriptor_dyn();
        let msg_name = desc.name();
        if msg_name == LoginResp::descriptor().name() {
            let msg = message.downcast_box::<LoginResp>().unwrap();
            player_id = msg.player_id;
            spawn_player(&mut commands, player_id, true, msg.color.unwrap(), None);
        } else if msg_name == SCPlayerEnterNotify::descriptor().name() {
            // let self_player_id = if query.is_empty() { player_id } else { query.single().0 };
            let msg = message.downcast_box::<SCPlayerEnterNotify>().unwrap();
            // if self_player_id != msg.player_id {
            spawn_player(&mut commands, msg.player_id, false, msg.color.unwrap(), None);
            // commands.spawn_bundle(SpriteBundle {
            //     transform: Transform { scale: Vec3::splat(30.), ..default() },
            //     sprite: Sprite { color: Color::rgb(50., 50., 50.), ..default() },
            //     ..default()
            // }).insert(Player(msg.player_id));
            // }
        } else if msg_name == SCOtherPlayersStateNotify::descriptor().name() {
            let msg = message.downcast_box::<SCOtherPlayersStateNotify>().unwrap();
            for each_player in msg.players {
                let location = Location {
                    x: each_player.location.x,
                    y: each_player.location.y,
                    state: each_player.state.unwrap(),
                };
                spawn_player(&mut commands, each_player.player_id, false, each_player.color.unwrap(), Some(location));
            }
        } else if msg_name == SCPlayerMoveNotify::descriptor().name() {
            let msg = message.downcast_box::<SCPlayerMoveNotify>().unwrap();
            others.for_each_mut(|(mut t, p)| {
                if p.0 == msg.player_id {
                    t.translation.x = msg.location.x as f32;
                    t.translation.y = msg.location.y as f32;
                }
            });
        }
    }
}

fn spawn_player(commands: &mut Commands, player_id: i32, is_self: bool, color: protocol::test::Color, location: Option<Location>) {
    let v3 = if let Some(l) = location {
        Vec3::new(l.x as f32, l.y as f32, 0.)
    } else {
        Vec3::default()
    };
    let mut thread_rng = thread_rng();
    let r = color.r as f32;
    let g = color.g as f32;
    let b = color.b as f32;
    let mut c = commands.spawn_bundle(SpriteBundle {
        transform: Transform { scale: Vec3::splat(20.), translation: v3, ..default() },
        sprite: Sprite { color: Color::rgb(r, g, b), ..default() },
        ..default()
    });
    c.insert(Player(player_id));
    if is_self {
        c.insert(SelfFlag);
    }
}

fn handle_keyboard_cmd(keyboard_input: Res<Input<KeyCode>>, time: Res<Time>, mut timer: ResMut<FuseTime>, send_to_server: Res<mpsc::Sender<ProtoMessage>>, mut query: Query<(&mut Transform, &Player), (With<Player>, With<SelfFlag>)>) {
    timer.timer.tick(time.delta());
    if query.is_empty() {
        return;
    }
    let (mut paddle_transform, player) = query.single_mut();
    let mut notify = PlayerMoveNotify::new();
    notify.player_id = player.0;
    let mut current_x = paddle_transform.translation.x;
    let mut current_y = paddle_transform.translation.y;
    notify.state = EnumOrUnknown::new(State::Idle);
    let delta = 5.;
    if keyboard_input.pressed(KeyCode::Left) {
        current_x -= delta;
        notify.state = EnumOrUnknown::new(State::MoveLeft);
    }

    if keyboard_input.pressed(KeyCode::Right) {
        current_x += delta;
        notify.state = EnumOrUnknown::new(State::MoveRight);
    }

    if keyboard_input.pressed(KeyCode::Up) {
        current_y += delta;
        notify.state = EnumOrUnknown::new(State::MoveUp);
    }

    if keyboard_input.pressed(KeyCode::Down) {
        current_y -= delta;
        notify.state = EnumOrUnknown::new(State::MoveDown);
    }

    paddle_transform.translation.x = current_x;
    paddle_transform.translation.y = current_y;
    if timer.timer.just_finished() {
        let mut v = Vector2::new();
        v.x = current_x as f64;
        v.y = current_y as f64;
        notify.location = MessageField::some(v);
        match send_to_server.try_send(Box::new(notify)) {
            Ok(_) => {}
            Err(err) => {
                error!("{}",err);
            }
        };
    }

    // info!("x:{} y:{}",current_x,current_y);
}

fn start_networking(mut commands: Commands, runtime: Res<Runtime>) {
    let (from_server_sender, from_server_receiver) = mpsc::channel::<ProtoMessage>(2000);
    let (to_server_sender, mut to_server_receiver) = mpsc::channel::<ProtoMessage>(2000);

    runtime.spawn(async move {
        // let addr = "172.20.198.152:4895";
        let addr = "127.0.0.1:4895";
        let cfg = kcp_config();
        let stream = tokio_kcp::KcpStream::connect(&cfg, addr.parse().unwrap()).await.unwrap();
        // let stream = TcpStream::connect(addr).await.unwrap();
        let codec = ProtoCodec::new(false);
        let mut framed = Framed::new(stream, codec);
        let player_id: i32 = random();
        let mut login = LoginReq::new();
        login.player_id = player_id;
        framed.send(Box::new(login)).await.unwrap();
        loop {
            tokio::select! {
                rsp = framed.next() => {
                    match rsp {
                        None => {
                            break;
                            info!("socket closed");
                        }
                        Some(Ok(rsp)) => {
                            from_server_sender.send(rsp).await;
                        }
                        Some(Err(err)) => {
                            error!("socket read message err:{}",err);
                        }
                    }
                }
                req = to_server_receiver.recv() => {
                    match req {
                        None => {
                            break;
                        }
                        Some(req) => {
                            match framed.send(req).await {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("socket send message err:{}",err);
                                    break;
                                }
                            };
                        }
                    }
                }
            }
        }
    });
    commands.insert_resource(to_server_sender);
    commands.insert_resource(from_server_receiver);
}