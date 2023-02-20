use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    sprite::MaterialMesh2dBundle,
    utils::Duration,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::Rng;

//player
const PLAYER_SIZE: Vec3 = Vec3::new(120.0, 20.0, 0.0);
const GAP_BETWEEN_PLAYER_AND_FLOOR: f32 = 60.0;
const PLAYER_SPEED: f32 = 500.0;
const PLAYER_PADDING: f32 = 10.0;
const PLAYER_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);

//ball
const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

const BALL_SIZE: Vec3 = Vec3::new(30.0, 30.0, 0.0);
const BALL_SPEED: f32 = 400.0;

const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -150.0, 1.0);
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

//walls
const WALL_THICKNESS: f32 = 10.0;
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

const LEFT_WALL: f32 = -450.0;
const RIGHT_WALL: f32 = 450.0;
const BOTTOM_WALL: f32 = -300.0;
const TOP_WALL: f32 = 300.0;

//enemies
const ENEMY_SIZE: Vec3 = Vec3::new(60.0, 20.0, 1.0);
const STARTING_ENEMY_POSITION: Vec3 = Vec3::new(-350.0, 250.0, 0.0);
const ENEMY_COLOR: Color = Color::rgb(0.96, 0.55, 0.54);

//particles
const PARTICLE_COLOR: Color = Color::rgb(0.46, 0.78, 0.47);
const PARTICLE_SIZE: Vec3 = Vec3::new(10.0, 10.0, 1.0);
const PARTICLE_LIFETIME: u64 = 100;
const SCALING_FACTOR: f32 = 0.9;

//other
const TIME_STEP: f32 = 1.0 / 60.0;
const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    InGame,
    Paused,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_state(AppState::InGame)
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .add_event::<GameOverEvent>()
        .add_system(bevy::window::close_on_esc)
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(check_for_collisions)
                .with_system(player_movement.before(check_for_collisions))
                .with_system(ball_movement.before(check_for_collisions))
                .with_system(tick_particles_lifetime)
                .with_system(update_particles_size)
                .with_system(change_game_state)
                .with_system(update_score_board)
                .with_system(setup_resetable)
                .with_system(game_over.before(setup_resetable))
        )
        .add_system_set(SystemSet::on_update(AppState::Paused).with_system(change_game_state))
        .run();
}

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct EnemyMarker;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Default)]
struct CollisionEvent;

#[derive(Bundle)]
struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

#[derive(Component)]
struct ParticleMarker;

#[derive(Component)]
struct Particle {
    lifetime: Timer,
}

#[derive(Component)]
struct Scoreboard;

#[derive(Resource)]
struct ScoreboardCounter {
    counter: u32,
}

#[derive(Component)]
struct LavaMarker;

#[derive(Default)]
struct GameOverEvent;

#[derive(Component)]
struct Resetable;

impl WallBundle {
    fn new(pos: Vec2, scale: Vec2) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: pos.extend(0.0),
                    scale: scale.extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

#[derive(Bundle)]
struct Enemy {
    sprite_bundle: SpriteBundle,
    collider: Collider,
    enemy_marker: EnemyMarker,
    name: Name,
    reset: Resetable,
}

impl Enemy {
    fn new(pos: Vec3, enemy_id: i32) -> Enemy {
        let mut s = String::from("Enemy ");
        s.push_str(&enemy_id.to_string());
        Enemy {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: pos,
                    scale: ENEMY_SIZE,
                    ..default()
                },
                sprite: Sprite {
                    color: ENEMY_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
            enemy_marker: EnemyMarker,
            name: Name::new(s),
            reset: Resetable,
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Walls
    commands.spawn(WallBundle::new(
        Vec2::new(LEFT_WALL, 0.0),
        Vec2::new(WALL_THICKNESS, (TOP_WALL - BOTTOM_WALL) + WALL_THICKNESS),
    ));
    commands.spawn(WallBundle::new(
        Vec2::new(RIGHT_WALL, 0.0),
        Vec2::new(WALL_THICKNESS, (TOP_WALL - BOTTOM_WALL) + WALL_THICKNESS),
    ));
    //commands.spawn(WallBundle::new(Vec2::new(0.0, BOTTOM_WALL), Vec2::new((RIGHT_WALL - LEFT_WALL) + WALL_THICKNESS, WALL_THICKNESS)));
    commands.spawn(WallBundle::new(
        Vec2::new(0.0, TOP_WALL),
        Vec2::new((RIGHT_WALL - LEFT_WALL) + WALL_THICKNESS, WALL_THICKNESS),
    ));

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -340.5, 0.0),
                scale: Vec3::new(1296.8, 40.2, 1.0),
                ..default()
            },
            sprite: Sprite {
                color: Color::rgb(1.0, 0.66, 0.38),
                ..default()
            },
            ..default()
        },
        Collider,
        LavaMarker,
        Name::new("Lava"),
    ));

    // Paddle
    let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PLAYER_AND_FLOOR;
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, paddle_y, 0.0),
                scale: PLAYER_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PLAYER_COLOR,
                ..default()
            },
            ..default()
        },
        Paddle,
        Collider,
        Resetable,
        Name::new("Player"),
    ));

    // Ball
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(BALL_COLOR)),
            transform: Transform::from_translation(BALL_STARTING_POSITION).with_scale(BALL_SIZE),
            ..default()
        },
        Ball,
        Resetable,
        Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
        Name::new("Ball"),
    ));

    //enemies
    for j in 0..8 {
        for i in 0..9 {
            commands.spawn(Enemy::new(
                Vec3::new(
                    STARTING_ENEMY_POSITION.x + (90.0 * i as f32),
                    STARTING_ENEMY_POSITION.y - (40.0 * j as f32),
                    STARTING_ENEMY_POSITION.z,
                ),
                i + (j * 9),
            ));
        }
    }

    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "100.0",
            TextStyle {
                font: asset_server.load("fonts/Roboto-Black.ttf"),
                font_size: 50.0,
                color: Color::WHITE,
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::TOP_CENTER)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(5.0),
                left: Val::Px(1130.0),
                top: Val::Px(0.0),
                ..default()
            },
            ..default()
        }),
        Scoreboard,
        Resetable,
    ));

    commands.insert_resource(ScoreboardCounter { counter: 0 });
}

fn setup_resetable(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut ev_game_over: EventReader<GameOverEvent>,
) {
    for _ in ev_game_over.iter() {
        // Paddle
        let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PLAYER_AND_FLOOR;
        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(0.0, paddle_y, 0.0),
                    scale: PLAYER_SIZE,
                    ..default()
                },
                sprite: Sprite {
                    color: PLAYER_COLOR,
                    ..default()
                },
                ..default()
            },
            Paddle,
            Collider,
            Resetable,
            Name::new("Player"),
        ));

        // Ball
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::default().into()).into(),
                material: materials.add(ColorMaterial::from(BALL_COLOR)),
                transform: Transform::from_translation(BALL_STARTING_POSITION)
                    .with_scale(BALL_SIZE),
                ..default()
            },
            Ball,
            Resetable,
            Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
            Name::new("Ball"),
        ));

        //enemies
        for j in 0..8 {
            for i in 0..9 {
                commands.spawn(Enemy::new(
                    Vec3::new(
                        STARTING_ENEMY_POSITION.x + (90.0 * i as f32),
                        STARTING_ENEMY_POSITION.y - (40.0 * j as f32),
                        STARTING_ENEMY_POSITION.z,
                    ),
                    i + (j * 9),
                ));
            }
        }

        commands.spawn((
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "100.0",
                TextStyle {
                    font: asset_server.load("fonts/Roboto-Black.ttf"),
                    font_size: 50.0,
                    color: Color::WHITE,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::TOP_CENTER)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(1130.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                ..default()
            }),
            Scoreboard,
            Resetable,
        ));

        commands.insert_resource(ScoreboardCounter { counter: 0 });
    }
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Paddle>>,
) {
    for mut paddle_transform in query.iter_mut() {
        let mut direction = 0.0;

        if keyboard_input.pressed(KeyCode::A) {
            direction -= 1.0;
        }

        if keyboard_input.pressed(KeyCode::D) {
            direction += 1.0;
        }

        // Calculate the new horizontal paddle position based on player input
        let new_paddle_position =
            paddle_transform.translation.x + direction * PLAYER_SPEED * TIME_STEP;

        // Update the paddle position,
        // making sure it doesn't cause the paddle to leave the arena
        let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PLAYER_SIZE.x / 2.0 + PLAYER_PADDING;
        let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PLAYER_SIZE.x / 2.0 - PLAYER_PADDING;

        paddle_transform.translation.x = new_paddle_position.clamp(left_bound, right_bound);
    }
}

fn ball_movement(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}

fn check_for_collisions(
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<
        (
            Entity,
            &Transform,
            Option<&EnemyMarker>,
            Option<&LavaMarker>,
        ),
        With<Collider>,
    >,
    mut collision_events: EventWriter<CollisionEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut counter: ResMut<ScoreboardCounter>,
    mut ev_game_over: EventWriter<GameOverEvent>,
) {
    for (mut ball_velocity, ball_transform) in ball_query.iter_mut() {
        let ball_size = ball_transform.scale.truncate();

        // check collision with walls
        for (entity_id, transform, maybe_enemy, maybe_lava) in &collider_query {
            let collision = collide(
                ball_transform.translation,
                ball_size,
                transform.translation,
                transform.scale.truncate(),
            );
            if let Some(collision) = collision {
                // Sends a collision event so that other systems can react to the collision
                collision_events.send_default();

                //destroying enemies
                if maybe_enemy.is_some() {
                    commands.entity(entity_id).despawn();
                    let music = asset_server.load("enemy_destroy.ogg");
                    audio.play(music);
                    spawn_particles(transform.translation, &mut commands);
                    counter.counter += 50;
                }

                if maybe_lava.is_some() {
                    ev_game_over.send_default();
                }

                // reflect the ball when it collides
                let mut reflect_x = false;
                let mut reflect_y = false;

                // only reflect if the ball's velocity is going in the opposite direction of the
                // collision
                match collision {
                    Collision::Left => reflect_x = ball_velocity.x > 0.0,
                    Collision::Right => reflect_x = ball_velocity.x < 0.0,
                    Collision::Top => reflect_y = ball_velocity.y < 0.0,
                    Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
                    Collision::Inside => { /* do nothing */ }
                }

                // reflect velocity on the x-axis if we hit something on the x-axis
                if reflect_x {
                    ball_velocity.x = -ball_velocity.x;
                }

                // reflect velocity on the y-axis if we hit something on the y-axis
                if reflect_y {
                    ball_velocity.y = -ball_velocity.y;
                }
            }
        }
    }
}

fn update_score_board(
    mut query: Query<&mut Text, With<Scoreboard>>,
    counter: Res<ScoreboardCounter>,
) {
    for mut text in &mut query {
        text.sections[0].value = format!("{0}", counter.counter);
    }
}

//diff x = 20, diff y = 10
fn spawn_particles(pos: Vec3, commands: &mut Commands) {
    let mut rng = rand::thread_rng();
    let particle_count = rng.gen_range(2..7);

    for _ in 0..particle_count {
        let rand_x = rng.gen_range(pos.x - 30.0..pos.x + 30.0);
        let rand_y = rng.gen_range(pos.y - 20.0..pos.y + 20.0);

        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(rand_x, rand_y, pos.z),
                    scale: PARTICLE_SIZE,
                    ..default()
                },
                sprite: Sprite {
                    color: PARTICLE_COLOR,
                    ..default()
                },
                ..default()
            },
            ParticleMarker,
            Particle {
                lifetime: Timer::new(Duration::from_millis(PARTICLE_LIFETIME), TimerMode::Once),
            },
            Name::new("Particle"),
        ));
    }

    /*
    FuseTime {
                // create the non-repeating fuse timer
                timer: Timer::new(Duration::from_secs(5), TimerMode::Once),
            },
    */
}

fn tick_particles_lifetime(
    time: Res<Time>,
    mut commands: Commands,
    mut particle_query: Query<(Entity, &mut Particle), With<ParticleMarker>>,
) {
    for (entity, mut particle) in particle_query.iter_mut() {
        particle.lifetime.tick(time.delta());

        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn update_particles_size(mut particle_query: Query<&mut Transform, With<ParticleMarker>>) {
    for mut trans in particle_query.iter_mut() {
        trans.scale.x += SCALING_FACTOR;
        trans.scale.y += SCALING_FACTOR;
    }
}

fn change_game_state(keyboard_input: Res<Input<KeyCode>>, mut app_state: ResMut<State<AppState>>) {
    if keyboard_input.just_pressed(KeyCode::Key1) {
        match app_state.current() {
            AppState::InGame => app_state.set(AppState::Paused).unwrap(),
            AppState::Paused => println!("nothing"),
        }
    }

    if keyboard_input.just_pressed(KeyCode::Key2) {
        match app_state.current() {
            AppState::InGame => println!("nothing"),
            AppState::Paused => app_state.set(AppState::InGame).unwrap(),
        }
    }
}

fn game_over(
    mut commands: Commands,
    query: Query<Entity, With<Resetable>>,
    mut ev_game_over: EventReader<GameOverEvent>,
) {
    for _ in ev_game_over.iter() {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    }
}