use bevy::ecs::{resource::Resource, system::Commands};

pub const WINDOW_WIDTH: f32 = 1920.;
pub const WINDOW_HEIGHT: f32 = 1080.;

#[derive(Resource)]
pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

pub fn setup_window_size(mut commands: Commands) {
    commands.insert_resource(WindowSize {
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
    });
}
