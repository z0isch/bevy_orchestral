use std::time::Duration;

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

use crate::{
    metronome::{Metronome, is_down_beat, nanos_from_beat, nanos_per_beat},
    player::Player,
    window_size::{WINDOW_HEIGHT, WINDOW_WIDTH},
};

const HIGHWAY_WIDTH: f32 = WINDOW_WIDTH as f32 / 30.;
const HIGHWAY_HEIGHT: f32 = WINDOW_HEIGHT as f32 / 8.;

// Perspective parameters
const PERSPECTIVE_SCALE_MIN: f32 = 0.4; // Scale at the far end (top)
const PERSPECTIVE_SCALE_MAX: f32 = 1.0; // Scale at the near end (bottom)

/// Converts a normalized position (0.0 at bottom, 1.0 at top) to a perspective scale
fn get_perspective_scale(normalized_y: f32) -> f32 {
    // Interpolate between max scale (bottom) and min scale (top)
    PERSPECTIVE_SCALE_MAX - normalized_y * (PERSPECTIVE_SCALE_MAX - PERSPECTIVE_SCALE_MIN)
}

/// Converts a y position within the highway to a perspective-transformed position and scale
fn apply_perspective(y_offset: f32) -> (f32, f32) {
    // Normalize y from -HIGHWAY_HEIGHT/2 to HIGHWAY_HEIGHT/2 into 0.0 to 1.0
    let normalized_y = (y_offset + HIGHWAY_HEIGHT / 2.) / HIGHWAY_HEIGHT;
    let scale = get_perspective_scale(normalized_y);

    // Apply perspective: things further away (higher y, closer to 1.0) are more compressed
    let perspective_y = y_offset;

    (perspective_y, scale)
}

#[derive(Component)]
pub struct NoteHighway;

/// Creates a trapezoid mesh for the perspective highway
fn create_trapezoid_mesh() -> Mesh {
    let top_width = HIGHWAY_WIDTH * PERSPECTIVE_SCALE_MIN;
    let bottom_width = HIGHWAY_WIDTH * PERSPECTIVE_SCALE_MAX;
    let height = HIGHWAY_HEIGHT;

    // Define vertices for a trapezoid (wider at bottom, narrower at top)
    let vertices = vec![
        // Bottom left
        [-bottom_width / 2., -height / 2., 0.],
        // Bottom right
        [bottom_width / 2., -height / 2., 0.],
        // Top right
        [top_width / 2., height / 2., 0.],
        // Top left
        [-top_width / 2., height / 2., 0.],
    ];

    let indices = vec![
        0, 1, 2, // First triangle
        0, 2, 3, // Second triangle
    ];

    let uvs = vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

pub fn setup_note_highway(
    metronome: Res<Metronome>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let note_lines: Vec<_> = (0..8)
        .map(|i| BeatLineBundle {
            beat_line: BeatLine { beat: i * 2 },
            material: MeshMaterial2d(materials.add(Color::hsva(
                0.,
                0.,
                if i % 2 == 0 { 1. } else { 0. },
                0.3,
            ))),
            transform: Transform::from_xyz(
                0.,
                -HIGHWAY_HEIGHT / 2. + (i as f32) * HIGHWAY_HEIGHT / 8.,
                11.,
            ),
        })
        .collect();

    commands
        .spawn((
            NoteHighway,
            Mesh2d(meshes.add(create_trapezoid_mesh())),
            MeshMaterial2d(materials.add(Color::hsva(0., 0., 0., 0.1))),
            Transform::from_xyz(
                0.,
                WINDOW_HEIGHT as f32 / 4. - HIGHWAY_HEIGHT as f32 / 2.,
                10.,
            ),
        ))
        .with_children(|parent| {
            // Apply perspective to the on-beat line
            let y_pos = -HIGHWAY_HEIGHT / 2.;
            let (_, scale) = apply_perspective(y_pos);
            let scaled_width = HIGHWAY_WIDTH * scale;

            parent.spawn((
                OnBeatLine {
                    timer: Timer::new(
                        Duration::from_nanos(nanos_per_beat(metronome.bpm) * 2),
                        TimerMode::Repeating,
                    ),
                },
                Mesh2d(meshes.add(Rectangle::new(scaled_width, 3.))),
                MeshMaterial2d(materials.add(Color::hsva(1., 1., 1., 0.8))),
                Transform::from_xyz(0., -HIGHWAY_HEIGHT / 2., 12.),
            ));
            for bundle in note_lines.into_iter() {
                parent.spawn(bundle);
            }
        });
}

#[derive(Component)]
pub struct OnBeatLine {
    timer: Timer,
}

#[derive(Component)]
pub struct BeatLine {
    beat: u8,
}

#[derive(Bundle)]
struct BeatLineBundle {
    beat_line: BeatLine,
    material: MeshMaterial2d<ColorMaterial>,
    transform: Transform,
}

pub fn beat_line_system(
    metronome: Res<Metronome>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &BeatLine)>,
) {
    for (entity, note_line) in query.iter_mut() {
        if metronome.is_beat_start_frame && metronome.beat == note_line.beat {
            commands.entity(entity).remove::<Transform>();
        } else {
            let percentage_from_beat =
                nanos_from_beat(&metronome, note_line.beat) / (nanos_per_beat(metronome.bpm) * 16);
            let distance_from_bottom: f32 = (percentage_from_beat * HIGHWAY_HEIGHT)
                .floor()
                .try_into()
                .unwrap();
            let y_pos = if distance_from_bottom >= 0. { -1. } else { 1. } * HIGHWAY_HEIGHT / 2.0
                + distance_from_bottom;

            // Apply perspective transformation
            let (perspective_y, scale) = apply_perspective(y_pos);

            // Update mesh with scaled width
            let scaled_width = HIGHWAY_WIDTH * scale;
            commands.entity(entity).insert((
                Mesh2d(meshes.add(Rectangle::new(scaled_width, 1.))),
                Transform::from_xyz(0., perspective_y, 11.).with_scale(Vec3::new(1.0, 1.0, 1.0)),
            ));
        }
    }
}

pub fn on_beat_line_system(
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    metronome: Res<Metronome>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut OnBeatLine)>,
) {
    let (entity, mut beat_line) = query.single_mut().unwrap();

    // Apply perspective to the on-beat line at the bottom
    let y_pos = -HIGHWAY_HEIGHT / 2.;
    let (_, scale) = apply_perspective(y_pos);
    let scaled_width = HIGHWAY_WIDTH * scale;

    if beat_line.timer.tick(time.delta()).just_finished() {
        beat_line.timer.reset();
        beat_line.timer.pause();
        commands
            .entity(entity)
            .insert(Mesh2d(meshes.add(Rectangle::new(scaled_width, 2.))));
    } else {
        if metronome.is_beat_start_frame && is_down_beat(&metronome) {
            beat_line.timer.unpause();
            commands
                .entity(entity)
                .insert(Mesh2d(meshes.add(Rectangle::new(scaled_width, 4.))));
        }
    }
}

pub fn note_highway_system(
    mut query: Query<&mut Transform, (With<NoteHighway>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
) {
    if let Ok(player_transform) = player_query.single() {
        if let Ok(mut note_highway_transform) = query.single_mut() {
            note_highway_transform.translation.x = player_transform.translation.x;
            note_highway_transform.translation.y =
                player_transform.translation.y + HIGHWAY_HEIGHT / 2. + 15.;
        }
    }
}
