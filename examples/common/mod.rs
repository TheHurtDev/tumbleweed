use hawthorn::math::{Quat, Vec3};
use tumbleweed::app::World;
use winit::keyboard::KeyCode;

/// Moves the player camera around the scene.
///
/// W, A, S, D, Q, and E translate the camera.
///
/// The arrow keys rotate the camera.
///
/// Holding Shift increases translation speed.
pub fn move_camera(world: &mut World) {
    const BASE_TRANSLATION_SPEED: f32 = 5.0;
    const TRANSLATION_SPEED_MULTIPLIER: f32 = 2.0;

    const ROTATION_SPEED: f32 = 2.25;

    let (cam_x, cam_y, cam_z) = world.camera.orientation.to_orthonormal_basis();
    let delta_secs = world.frame_duration.as_secs_f32();

    let mut translation = Vec3::ZERO;

    if world.input_state.is_key_down(KeyCode::KeyW) {
        translation -= cam_z;
    }
    if world.input_state.is_key_down(KeyCode::KeyS) {
        translation += cam_z;
    }
    if world.input_state.is_key_down(KeyCode::KeyA) {
        translation -= cam_x;
    }
    if world.input_state.is_key_down(KeyCode::KeyD) {
        translation += cam_x;
    }
    if world.input_state.is_key_down(KeyCode::KeyQ) {
        translation += cam_y;
    }
    if world.input_state.is_key_down(KeyCode::KeyE) {
        translation -= cam_y;
    }

    let mut translation_speed = BASE_TRANSLATION_SPEED;

    if world.input_state.is_key_down(KeyCode::ShiftLeft)
        || world.input_state.is_key_down(KeyCode::ShiftRight)
    {
        translation_speed *= TRANSLATION_SPEED_MULTIPLIER;
    }

    world.camera.position += translation_speed * delta_secs * translation.normalize_or_zero();

    let mut rotation_horiz = 0.0;
    let mut rotation_vert = 0.0;

    if world.input_state.is_key_down(KeyCode::ArrowUp) {
        rotation_vert += ROTATION_SPEED;
    }
    if world.input_state.is_key_down(KeyCode::ArrowDown) {
        rotation_vert -= ROTATION_SPEED;
    }
    if world.input_state.is_key_down(KeyCode::ArrowLeft) {
        rotation_horiz += ROTATION_SPEED;
    }
    if world.input_state.is_key_down(KeyCode::ArrowRight) {
        rotation_horiz -= ROTATION_SPEED;
    }

    const MIN_THETA: f32 = 2.0_f32.to_radians();

    let theta = cam_z.angle_between(Vec3::Y);
    let new_theta =
        (theta + rotation_vert * delta_secs).clamp(MIN_THETA, core::f32::consts::PI - MIN_THETA);

    let delta_theta = new_theta - theta;
    let delta_phi = rotation_horiz * delta_secs;

    let new_cam_z = cam_z
        .rotate_around(cam_x, delta_theta)
        .rotate_around(Vec3::Y, delta_phi);

    world.camera.orientation = Quat::look_to_rh(-new_cam_z, Vec3::Y);
}
