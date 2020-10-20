// #![allow(dead_code)]
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_4, PI};

pub struct OrbitCamera {
    /// Which entity the camera is target
    pub target: Option<Entity>,
    /// What point in the world the camera was last facing
    pub focus: Vec3,
    /// Where the camera should be
    position: Vec3,
    /// The distance the camera should be from the entity it is target
    distance: f32,
    // The minimum distance away from the target, must be more than 0
    // min_distance: f32,
    // The maximum distance away from the target, must be more than `min_distance`
    // max_distance: f32,
    /// pitch aka aradians from negative XZ plane
    pitch: f32,
    /// radians from positive Z axis
    yaw: f32,
}

impl OrbitCamera {
    const MIN_DISTANCE: f32 = 5.0; // currently hardcoded
    const MAX_DISTANCE: f32 = 100.0; // currently hardcoded
    const MAX_PITCH: f32 = 89.0 / 180.0 * PI; // 89 degrees
    const MIN_PITCH: f32 = -Self::MAX_PITCH; // -89 degrees
    const MAX_YAW: f32 = PI; // 180 degrees
    const MIN_YAW: f32 = -Self::MAX_YAW; // -180 degrees
    pub fn new(target: Option<Entity>, mut distance: f32, pitch: f32, yaw: f32) -> Self {
        distance = distance
            .max(f32::EPSILON) // until I know otherwise, this should be sufficiently positive
            .max(Self::MIN_DISTANCE)
            .min(Self::MAX_DISTANCE);
        let focus = Vec3::default();
        let position = focus + Self::calculate_relative_position(pitch, yaw, distance);
        Self {
            target,
            focus,
            position,
            distance,
            pitch,
            yaw,
        }
    }
    pub fn set_focus(&mut self, focus: Vec3) -> &mut Self {
        self.focus = focus;
        self
    }
    pub fn distance(&self) -> f32 {
        self.distance
    }
    pub fn set_distance(&mut self, distance: f32) -> &mut Self {
        self.distance = distance.max(Self::MIN_DISTANCE).min(Self::MAX_DISTANCE);
        self
    }
    pub fn add_distance(&mut self, distance: f32) -> &mut Self {
        self.set_distance(self.distance() + distance);
        self
    }
    pub fn pitch(&self) -> f32 {
        self.pitch
    }
    pub fn set_pitch(&mut self, pitch: f32) -> &mut Self {
        self.pitch = pitch.max(Self::MIN_PITCH).min(Self::MAX_PITCH);
        self
    }
    pub fn add_pitch(&mut self, pitch: f32) -> &mut Self {
        self.set_pitch(self.pitch() + pitch);
        self
    }
    pub fn yaw(&self) -> f32 {
        self.yaw
    }
    pub fn set_yaw(&mut self, yaw: f32) -> &mut Self {
        self.yaw = Self::wrap(yaw, Self::MIN_YAW, Self::MAX_YAW);
        self
    }
    pub fn add_yaw(&mut self, yaw: f32) -> &mut Self {
        self.set_yaw(self.yaw() + yaw);
        self
    }
    pub fn position(&self) -> Vec3 {
        self.focus + Self::calculate_relative_position(self.pitch, self.yaw, self.distance)
    }
    pub fn update_position(&mut self) -> &mut Self {
        self.position = self.position();
        self
    }
    fn wrap(num: f32, min: f32, max: f32) -> f32 {
        if num < min {
            Self::wrap(max - (min - num), min, num)
        } else if num > max {
            Self::wrap(min - (max - num), min, num)
        } else {
            num
        }
    }
    fn calculate_relative_position(pitch: f32, yaw: f32, distance: f32) -> Vec3 {
        let point = Vec3::new(
            yaw.sin() * pitch.cos(),
            pitch.sin(),
            yaw.cos() * pitch.cos(),
        );
        assert!(point.is_normalized());
        point * distance
    }
}

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_default_plugins()
        .add_startup_system(setup.system())
        .add_system(zoom_camera.system())
        .add_system(update_camera.system())
        .add_system(move_camera.system())
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // add entities to the world
    let cube_entity = commands
        // cube
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            ..Default::default()
        })
        .current_entity();
    commands
        // plane
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        // light
        .spawn(LightComponents {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        })
        // camera
        .spawn(Camera3dComponents {
            transform: Transform::from_translation(Vec3::new(-3.0, 5.0, 8.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        })
        .with(OrbitCamera::new(
            cube_entity,
            50.0,
            FRAC_PI_4,
            FRAC_PI_4,
        ));
}

// example
fn zoom_camera(
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
    mut zooming_out: Local<bool>,
    mut camera_query: Query<&mut OrbitCamera>,
) {
    // change the distance only
    if let None = *timer {
        *timer = Some(Timer::from_seconds(1.0, true));
    }
    let dt = time.delta_seconds;
    if let Some(timer) = &mut *timer {
        for mut orbit_camera in &mut camera_query.iter() {
            orbit_camera.add_distance(dt * if *zooming_out { 100.0 } else { -100.0 });
            timer.tick(dt);
            if timer.just_finished {
                *zooming_out = !*zooming_out;
            }
        }
    }
}

// core
fn update_camera(
    mut camera_query: Query<&mut OrbitCamera>,
    target_query: Query<(Entity, &Transform)>,
) {
    for mut orbit_camera in &mut camera_query.iter() {
        if let Some(target_entity) = orbit_camera.target {
            if let Ok(target_transform) = target_query.get::<Transform>(target_entity) {
                orbit_camera.focus = target_transform.translation;
            }
        }
    }
}

// core
fn move_camera(mut camera_query: Query<(&OrbitCamera, &mut Transform)>) {
    const UP: Vec3 = Vec3::unit_y();
    for (orbit_camera, mut transform) in &mut camera_query.iter() {
        transform.translation = orbit_camera.position();
        transform.look_at(orbit_camera.focus, UP);
    }
}
