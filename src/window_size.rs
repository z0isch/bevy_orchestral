use bevy::ecs::{resource::Resource, system::Commands};

pub const WINDOW_WIDTH: u32 = 1920;
pub const WINDOW_HEIGHT: u32 = 1080;

#[derive(Resource)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

pub fn setup_window_size(mut commands: Commands) {
    commands.insert_resource(WindowSize {
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
    });
}
