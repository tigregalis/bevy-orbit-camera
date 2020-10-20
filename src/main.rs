// #![allow(dead_code)]
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_4, PI};

// core
const Y_AXIS: Vec3 = Vec3::unit_y();

// core
pub struct OrbitCameraTarget;

// core
pub struct OrbitCamera {
    /// Which entity the camera is target
    pub target: Option<Entity>,
    /// What point in the world the camera was last facing
    pub focus: Vec3,
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

// core
impl OrbitCamera {
    const MIN_DISTANCE: f32 = 5.0; // currently hardcoded - TODO: (maybe) provide option, or remove
    const MAX_DISTANCE: f32 = 100.0; // currently hardcoded - TODO: (maybe) provide option, or remove
    const MAX_PITCH: f32 = 89.9 / 180.0 * PI; // 89.9 degrees
    const MIN_PITCH: f32 = -Self::MAX_PITCH; // -89.9 degrees
    const MAX_YAW: f32 = PI; // 180 degrees
    const MIN_YAW: f32 = -Self::MAX_YAW; // -180 degrees
    pub fn new(target: Option<Entity>, mut distance: f32, pitch: f32, yaw: f32) -> Self {
        distance = distance
            .max(f32::EPSILON) // until I know otherwise, this should be sufficiently positive
            .max(Self::MIN_DISTANCE)
            .min(Self::MAX_DISTANCE);
        let focus = Vec3::default();
        Self {
            target,
            focus,
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
        self.distance = distance
            .max(f32::EPSILON)
            .max(Self::MIN_DISTANCE)
            .min(Self::MAX_DISTANCE);
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
    fn wrap(num: f32, min: f32, max: f32) -> f32 {
        if num < min {
            // TODO: (maybe) turn this into a loop rather than recursive
            Self::wrap(max - (min - num), min, max)
        } else if num > max {
            // TODO: (maybe) turn this into a loop rather than recursive
            Self::wrap(min - (max - num), min, max)
        } else {
            num
        }
    }
    // this should be part of bevy or glam
    fn calculate_relative_position(pitch: f32, yaw: f32, distance: f32) -> Vec3 {
        // https://stackoverflow.com/questions/52781607/3d-point-from-two-angles-and-a-distance
        let point = Vec3::new(
            yaw.sin() * pitch.cos(),
            pitch.sin(),
            yaw.cos() * pitch.cos(),
        );
        assert!(point.is_normalized());
        point * distance
    }
}

// example / core
// TODO: Turn into a plugin
fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_default_plugins()
        .add_startup_system(setup.system())
        .add_system(move_cube.system())
        .add_system(zoom_camera.system())
        .add_system(rotate_camera.system())
        .add_system(update_camera.system())
        .add_system(move_camera.system())
        .run();
}

// example
struct Cube;

// example
/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // add entities to the world

    // store the cube entity
    let cube_entity = commands
        // spawn a cube
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            ..Default::default()
        })
        .with(Cube)
        .with(OrbitCameraTarget)
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
            cube_entity, // provide the cube entity to the OrbitCamera
            50.0,        // 50.0 units from the origin of the entity
            FRAC_PI_4,   // 45 degrees from horizontal to vertical
            FRAC_PI_4,   // 45 degrees counter-clockwise from Z axis
        ));
}

// example
fn move_cube(time: Res<Time>, mut cube_query: Query<(&Cube, &mut Transform)>) {
    let dt = time.delta_seconds;
    let origin = Vec3::new(0.0, 11.0, -10.0);
    if let Some((_, mut cube_transform)) = cube_query.iter().iter().next() {
        let velocity = (cube_transform.translation - origin).cross(Y_AXIS);
        cube_transform.translation += velocity * dt;
    }
}

// example / default implementation
pub fn zoom_camera(
    mut mouse_wheel_event_reader: Local<EventReader<MouseWheel>>,
    mouse_wheel_events: Res<Events<MouseWheel>>,
    mut camera_query: Query<&mut OrbitCamera>,
) {
    let mut zoom = 0.0;
    for event in mouse_wheel_event_reader.iter(&mouse_wheel_events) {
        zoom += event.y;
    }
    for mut orbit_camera in &mut camera_query.iter() {
        orbit_camera.add_distance(-zoom);
    }
}

// example / default implementation
pub fn rotate_camera(
    mut mouse_motion_event_reader: Local<EventReader<MouseMotion>>,
    mouse_motion_events: Res<Events<MouseMotion>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut camera_query: Query<&mut OrbitCamera>,
) {
    if mouse_button_input.pressed(MouseButton::Middle) {
        let mut yaw = 0.0;
        let mut pitch = 0.0;
        for event in mouse_motion_event_reader.iter(&mouse_motion_events) {
            let (delta_yaw, delta_pitch) = event.delta.into();
            yaw += delta_yaw;
            pitch += delta_pitch;
        }
        let yaw = -yaw * 2.0 * PI / 1280.0; // 360 degrees from left edge of window to right edge of window - currently hardcoded
        let pitch = pitch * PI / 720.0; // 180 degrees from bottom edge of window to top edge of window - currently hardcoded
        for mut orbit_camera in &mut camera_query.iter() {
            orbit_camera.add_yaw(yaw);
            orbit_camera.add_pitch(pitch);
        }
    }
}

// core
pub fn update_camera(
    mut camera_query: Query<&mut OrbitCamera>,
    target_query: Query<(Entity, &OrbitCameraTarget, &Transform)>,
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
// TODO: make this smoother (i.e. use acceleration, deceleration and velocity)
// TODO: make this lazier (i.e. position changes when)
// https://catlikecoding.com/unity/tutorials/movement/orbit-camera/
pub fn move_camera(mut camera_query: Query<(&OrbitCamera, &mut Transform)>) {
    for (orbit_camera, mut camera_transform) in &mut camera_query.iter() {
        camera_transform.translation = orbit_camera.position();
        camera_transform.look_at(orbit_camera.focus, Y_AXIS);
    }
}
