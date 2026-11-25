mod model;

use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use model::{Voxel, VoxelWorld, altura_terreno_seed};
use std::collections::HashMap;

const SIDE: i32 = 32;
const EYE_HEIGHT: f32 = 1.65;
const TITLE: &str = "CaveGame — WASD | Espaço: pular | Mouse: blocos | Esc: soltar cursor";

#[derive(Component, Default)]
struct Player {
    yaw: f32,
    pitch: f32,
    vertical_speed: f32,
}

#[derive(Resource)]
struct Terrain {
    model: VoxelWorld,
    entities: HashMap<(i32, i32, i32), Entity>,
    mesh: Handle<Mesh>,
    palette: [Handle<StandardMaterial>; 3],
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: TITLE.into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (edit_blocks, cursor, player).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut terrain = Terrain {
        model: VoxelWorld::novo_com_seed(2026),
        entities: HashMap::new(),
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        palette: [
            materials.add(Color::srgb(0.43, 0.67, 0.33)),
            materials.add(Color::srgb(0.50, 0.34, 0.23)),
            materials.add(Color::srgb(0.45, 0.50, 0.57)),
        ],
    };
    for z in 0..SIDE {
        for x in 0..SIDE {
            let large = altura_terreno_seed(terrain.model.seed, x / 5, z / 5);
            let height = 3 + (large - 4) / 3;
            for y in 0..=height {
                let block = if y == height {
                    Voxel::Grama
                } else if y + 2 < height {
                    Voxel::Pedra
                } else {
                    Voxel::Terra
                };
                terrain.model.set(x, y, z, block);
            }
        }
    }
    let visible: Vec<_> = terrain
        .model
        .blocos
        .keys()
        .copied()
        .filter(|&(x, y, z)| terrain.model.faces_visiveis(x, y, z) > 0)
        .collect();
    for position in visible {
        reconcile(&mut commands, &mut terrain, position);
    }
    commands.insert_resource(terrain);
    commands.spawn((
        Camera3d::default(),
        Player::default(),
        Transform::from_xyz(16.5, 11.0, 21.5),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.45, 0.0)),
    ));
}

fn cursor(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut windows: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let Ok((window, mut pointer)) = windows.single_mut() else {
        return;
    };
    if keys.just_pressed(KeyCode::Escape) || !window.focused {
        pointer.grab_mode = CursorGrabMode::None;
        pointer.visible = true;
    } else if mouse.just_pressed(MouseButton::Left) && pointer.visible {
        pointer.grab_mode = CursorGrabMode::Locked;
        pointer.visible = false;
    }
}

fn player(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<AccumulatedMouseMotion>,
    terrain: Res<Terrain>,
    pointer: Query<&CursorOptions, With<PrimaryWindow>>,
    mut players: Query<(&mut Transform, &mut Player)>,
) {
    let Ok((mut transform, mut player)) = players.single_mut() else {
        return;
    };
    let captured = pointer.single().is_ok_and(|pointer| !pointer.visible);
    if captured {
        player.yaw -= mouse.delta.x * 0.0025;
        player.pitch = (player.pitch - mouse.delta.y * 0.0025).clamp(-1.54, 1.54);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0);
    }
    let dt = time.delta_secs().min(0.05);
    let pressed = |key| {
        if captured && keys.pressed(key) {
            1.0
        } else {
            0.0
        }
    };
    let direction = Vec3::new(
        pressed(KeyCode::KeyD) - pressed(KeyCode::KeyA),
        0.0,
        pressed(KeyCode::KeyS) - pressed(KeyCode::KeyW),
    )
    .normalize_or_zero();
    let delta = Quat::from_rotation_y(player.yaw) * direction * (5.0 * dt);
    let mut feet = transform.translation - Vec3::Y * EYE_HEIGHT;
    for movement in [Vec3::X * delta.x, Vec3::Z * delta.z] {
        let target = feet + movement;
        if !occupied(&terrain.model, target) {
            feet = target;
        }
    }
    let grounded = occupied(&terrain.model, feet - Vec3::Y * 0.05);
    if captured && grounded && keys.just_pressed(KeyCode::Space) {
        player.vertical_speed = 6.5;
    }
    player.vertical_speed = (player.vertical_speed - dt * 18.0).max(-9.0);
    let destination = feet + Vec3::Y * player.vertical_speed * dt;
    if occupied(&terrain.model, destination) {
        player.vertical_speed = 0.0;
    } else {
        feet = destination;
    }
    if feet.y < -4.0 {
        feet = Vec3::new(16.5, 10.0, 21.5);
        player.vertical_speed = 0.0;
    }
    transform.translation = feet + Vec3::Y * EYE_HEIGHT;
}

fn occupied(world: &VoxelWorld, feet: Vec3) -> bool {
    if feet.x < 0.15
        || feet.z < 0.15
        || feet.x >= SIDE as f32 - 0.15
        || feet.z >= SIDE as f32 - 0.15
    {
        return true;
    }
    for y in [0.02, 0.8, EYE_HEIGHT] {
        for (x, z) in [(-0.16, -0.16), (-0.16, 0.16), (0.16, -0.16), (0.16, 0.16)] {
            let (x, y, z) = cell(feet + Vec3::new(x, y, z));
            if world.get(x, y, z) != Voxel::Ar {
                return true;
            }
        }
    }
    false
}

fn edit_blocks(
    mut commands: Commands,
    mut terrain: ResMut<Terrain>,
    mouse: Res<ButtonInput<MouseButton>>,
    pointer: Query<&CursorOptions, With<PrimaryWindow>>,
    player: Query<&Transform, With<Player>>,
) {
    let left = mouse.just_pressed(MouseButton::Left);
    let right = mouse.just_pressed(MouseButton::Right);
    if (!left && !right) || !pointer.single().is_ok_and(|pointer| !pointer.visible) {
        return;
    }
    let Ok(transform) = player.single() else {
        return;
    };
    let direction = *transform.forward();
    let mut last = None;
    for step in 2..130 {
        let position = cell(transform.translation + direction * (step as f32 * 0.045));
        if terrain.model.get(position.0, position.1, position.2) == Voxel::Ar {
            last = Some(position);
            continue;
        }
        let (target, material) = if right {
            let Some(target) = last else {
                return;
            };
            (target, Voxel::Terra)
        } else {
            (position, Voxel::Ar)
        };
        let (x, y, z) = target;
        if !(0..SIDE).contains(&x) || !(0..SIDE).contains(&z) || !(0..24).contains(&y) {
            return;
        }
        terrain.model.set(x, y, z, material);
        if material != Voxel::Ar
            && occupied(&terrain.model, transform.translation - Vec3::Y * EYE_HEIGHT)
        {
            terrain.model.set(x, y, z, Voxel::Ar);
            return;
        }
        for (dx, dy, dz) in [
            (0, 0, 0),
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ] {
            reconcile(&mut commands, &mut terrain, (x + dx, y + dy, z + dz));
        }
        break;
    }
}

fn reconcile(commands: &mut Commands, terrain: &mut Terrain, position: (i32, i32, i32)) {
    let (x, y, z) = position;
    if terrain.model.faces_visiveis(x, y, z) == 0 {
        if let Some(entity) = terrain.entities.remove(&position) {
            commands.entity(entity).despawn();
        }
        return;
    }
    if terrain.entities.contains_key(&position) {
        return;
    }
    let material = match terrain.model.get(x, y, z) {
        Voxel::Grama => 0,
        Voxel::Terra => 1,
        Voxel::Pedra => 2,
        Voxel::Ar => return,
    };
    let entity = commands
        .spawn((
            Mesh3d(terrain.mesh.clone()),
            MeshMaterial3d(terrain.palette[material].clone()),
            Transform::from_xyz(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5),
        ))
        .id();
    terrain.entities.insert(position, entity);
}

fn cell(pos: Vec3) -> (i32, i32, i32) {
    (
        pos.x.floor() as i32,
        pos.y.floor() as i32,
        pos.z.floor() as i32,
    )
}
