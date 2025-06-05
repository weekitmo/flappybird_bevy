use bevy::prelude::*;
use bevy::{color::palettes::css::BLUE, window::PrimaryWindow};
use rand::{rngs::ThreadRng, thread_rng, Rng};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Flappybird"),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        resolution: Vec2::new(512., 512.).into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, setup_level)
        .add_systems(
            Update,
            (update_bird, update_obstacles, draw_coordinate_system),
        )
        .run();
}

fn draw_coordinate_system(
    mut gizmos: Gizmos,
    mut commands: Commands,
    game_manager: Res<GameManager>,
    query: Query<Entity, With<CoordinateLabel>>,
) {
    let window_width = game_manager.window_dimensions.x;
    let window_height = game_manager.window_dimensions.y;

    // 清除旧的标签
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // X轴 (红色)
    gizmos.line_2d(
        Vec2::new(-window_width / 2.0, 0.0),
        Vec2::new(window_width / 2.0, 0.0),
        Color::srgb(1.0, 0.0, 0.0),
    );

    // Y轴 (绿色)
    gizmos.line_2d(
        Vec2::new(0.0, -window_height / 2.0),
        Vec2::new(0.0, window_height / 2.0),
        Color::srgb(0.0, 1.0, 0.0),
    );

    // 绘制刻度和标签
    let step = 50.0;
    for x in (-window_width as i32 / 2..=window_width as i32 / 2).step_by(step as usize) {
        // 刻度线
        gizmos.line_2d(
            Vec2::new(x as f32, -5.0),
            Vec2::new(x as f32, 5.0),
            Color::srgb(1.0, 0.0, 0.0),
        );

        // X轴标签
        if x != 0 {
            commands.spawn((
                Text2d::new(x.to_string()),
                TextFont {
                    font_size: 16.0,
                    ..Default::default()
                },
                TextColor(BLUE.into()),
                Transform::from_xyz(x as f32, 10.0, 0.0),
                CoordinateLabel,
            ));
        }
    }

    for y in (-window_height as i32 / 2..=window_height as i32 / 2).step_by(step as usize) {
        // 刻度线
        gizmos.line_2d(
            Vec2::new(-5.0, y as f32),
            Vec2::new(5.0, y as f32),
            Color::srgb(0.0, 1.0, 0.0),
        );

        // Y轴标签
        if y != 0 {
            commands.spawn((
                Text2d::new(y.to_string()),
                TextFont {
                    font_size: 16.0,
                    ..Default::default()
                },
                TextColor(BLUE.into()),
                Transform::from_xyz(-10., y as f32, 0.0),
                CoordinateLabel,
            ));
        }
    }
}

#[derive(Component)]
struct CoordinateLabel;

const PIXEL_RATIO: f32 = 4.0;
const FLAP_FORCE: f32 = 500.;
const GRAVITY: f32 = 2000.;
const VELOCITY_TO_ROTATION_RATIO: f32 = 7.5;
// 障碍物数量
const OBSTACLE_AMOUNT: f32 = 5.;
// 障碍物尺寸， 原图 18x144
const OBSTACLE_WIDTH: f32 = 32.0;
const OBSTACLE_HEIGHT: f32 = 144.0;
// 垂直偏移
const OBSTACLE_VERTICAL_OFFSET: f32 = 30.0;

const OBSTACLE_GAP_SIZE: f32 = 15.;
const OBSTACLE_SPACING: f32 = 60.0;
// 障碍物速度
const OBSTACLE_SPEED: f32 = 150.0;

#[derive(Resource)]
pub struct GameManager {
    pub pipe_image: Handle<Image>,
    // 窗口尺寸
    pub window_dimensions: Vec2,
    pub game_over: bool,
}

#[derive(Component)]
struct Bird {
    pub velocity: f32,
    pub dead: bool,
}

#[derive(Component)]
struct Obstacle {
    // 方向
    pub pipe_direction: f32,
}

// 相当于unity的Start方法
fn setup_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single().unwrap();
    let pipe_image = asset_server.load("pipe.png");
    commands.insert_resource(GameManager {
        pipe_image: pipe_image.clone(),
        window_dimensions: Vec2::new(window.width(), window.height()),
        game_over: false,
    });
    // Load the background image
    commands.insert_resource(ClearColor(Color::srgb(0.5, 0.7, 0.8)));
    commands.spawn(Camera2d::default());
    // Load the ground image
    commands.spawn((
        Sprite {
            image: asset_server.load("bird.png"),
            ..Default::default()
        },
        // splat 相当于 Vec3::new(PIXEL_RATIO, PIXEL_RATIO, PIXEL_RATIO)
        Transform::IDENTITY.with_scale(Vec3::splat(PIXEL_RATIO)),
        Bird {
            velocity: 0.0,
            dead: false,
        },
    ));
    let mut rand = thread_rng();
    spawn_obstacles(&mut commands, &mut rand, window.width(), &pipe_image);
    // 512x512
    println!(
        "Game setup complete. Window size: {:?} {:?}",
        window.width(),
        window.height()
    );
}
// 去障碍物中心，中心计算 + 中间的间隙
fn get_centered_pipe_position() -> f32 {
    return (OBSTACLE_HEIGHT / 2.0 + OBSTACLE_GAP_SIZE) * PIXEL_RATIO;
}

fn spawn_obstacles(
    commands: &mut Commands,
    rand: &mut ThreadRng,
    window_width: f32,
    pipe_image: &Handle<Image>,
) {
    for i in 0..OBSTACLE_AMOUNT as u32 {
        let y_offset = generate_offset(rand);
        let x_pos = window_width / 2.0 + (i as f32 * OBSTACLE_SPACING * PIXEL_RATIO);
        // 顶部
        spawn_obstacle(
            Vec3::X * x_pos + Vec3::Y * (get_centered_pipe_position() + y_offset),
            1.,
            commands,
            pipe_image,
        );
        // 底部
        spawn_obstacle(
            Vec3::new(x_pos, -get_centered_pipe_position() + y_offset, 0.),
            -1.,
            commands,
            pipe_image,
        );
    }
}

fn generate_offset(rand: &mut ThreadRng) -> f32 {
    return rand.gen_range(-OBSTACLE_VERTICAL_OFFSET..OBSTACLE_VERTICAL_OFFSET) * PIXEL_RATIO;
}
fn spawn_obstacle(
    translation: Vec3,
    pipe_direction: f32,
    commands: &mut Commands,
    pipe_image: &Handle<Image>,
) {
    commands.spawn((
        Sprite {
            image: pipe_image.clone(),
            ..Default::default()
        },
        Transform::from_translation(translation).with_scale(Vec3::new(
            PIXEL_RATIO,
            PIXEL_RATIO * -pipe_direction,
            PIXEL_RATIO,
        )),
        Obstacle { pipe_direction },
    ));
}

fn update_bird(
    mut commands: Commands,
    mut bird_query: Query<(&mut Transform, &mut Bird, Entity), Without<Obstacle>>,
    mut obstacle_query: Query<(&Transform, Entity), With<Obstacle>>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_manager: ResMut<GameManager>,
) {
    if game_manager.game_over {
        return;
    }
    if let Ok((mut transform, mut bird, bird_entity)) = bird_query.single_mut() {
        if keyboard_input.just_pressed(KeyCode::Space) {
            // Apply flap force
            bird.velocity = FLAP_FORCE;
        }
        bird.velocity -= GRAVITY * time.delta_secs();
        transform.translation.y += bird.velocity * time.delta_secs();
        transform.rotation = Quat::from_rotation_z(
            f32::clamp(bird.velocity / VELOCITY_TO_ROTATION_RATIO, -90., 90.).to_radians(),
        );

        if transform.translation.y < -game_manager.window_dimensions.y / 2. {
            // Bird falls below the ground
            bird.dead = true;
        }

        if bird.dead {
            game_manager.game_over = true;
            // commands.entity(bird_entity).despawn();

            println!("Bird has died.");
        }
    }
}

fn update_obstacles(
    time: Res<Time>,
    mut obstacle_query: Query<(&mut Transform, &Obstacle)>,
    game_manager: Res<GameManager>,
) {
    let mut rand = thread_rng();
    let y_offset = generate_offset(&mut rand);
    for (mut transform, obstacle) in obstacle_query.iter_mut() {
        transform.translation.x -= OBSTACLE_SPEED * time.delta_secs();

        if transform.translation.x + OBSTACLE_WIDTH * PIXEL_RATIO / 2.
            < -game_manager.window_dimensions.x / 2.
        {
            transform.translation.x += OBSTACLE_AMOUNT as f32 * OBSTACLE_SPACING * PIXEL_RATIO;
            transform.translation.y =
                get_centered_pipe_position() * obstacle.pipe_direction + y_offset;
        }
    }
}
