use hawthorn::math::{Mat3, Mat4, Quat, Vec3};
use std::time::Duration;
use tumbleweed::app::{App, Body, DrawCommand, Mesh, World};
use tumbleweed::physics::{self, Particle, ParticleSystem};
use winit::keyboard::KeyCode;

mod common;

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let mut app = App::new("Tumbleweed App");

    app.create_mesh("cube", Mesh::new_flat_shaded_cube());

    app.add_system(common::move_camera);
    app.add_system(move_particle);
    app.add_system(physics::physics_pipeline);
    app.add_system(remove_particles);
    app.add_system(draw_particles);

    // Build scene

    const SPREAD: f32 = 20.0;

    app.world.entities.spawn(Body {
        particle: Particle {
            position: -SPREAD * Vec3::Z,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
        },

        // These fields are only used by the renderer
        orientation: Quat::IDENTITY,
        scale: Vec3::ONE,
        mesh_name: String::from("cube"),
        color: Vec3::new(0.9961, 0.8275, 0.1725),
    });

    app.world.entities.spawn(ParticleSystem::new(
        Duration::from_millis(20),
        Vec3::new(-SPREAD, 0.0, SPREAD),
        0.0_f32.to_radians()..=5.0_f32.to_radians(),
        25.0..=35.0,
        40.0,
    ));

    app.world.entities.spawn(ParticleSystem::new(
        Duration::from_millis(20),
        Vec3::new(0.0, 0.0, SPREAD),
        0.0_f32.to_radians()..=180.0_f32.to_radians(),
        5.0..=8.0,
        25.0,
    ));

    app.world.entities.spawn(ParticleSystem::new(
        Duration::from_millis(100),
        Vec3::new(SPREAD, 0.0, SPREAD),
        60.0_f32.to_radians()..=65.0_f32.to_radians(),
        10.0..=10.5,
        15.0,
    ));

    app.start()
}

/// Player controls for moving a particle around the scene.
///
/// Holding the I or O key applies a constant acceleration.
///
/// Pressing the K or L key applies an abrupt change in velocity.
///
/// Pressing the Comma or Period key applies an abrupt change in position.
fn move_particle(world: &mut World) {
    const DELTA_ACCEL: f32 = 4.0;
    const DELTA_VEL: f32 = 8.0;
    const DELTA_POS: f32 = 1.0;

    let Some(particle) = &mut world
        .entities
        .each_mut::<Body>()
        .next()
        .map(|b| &mut b.particle)
    else {
        return;
    };

    // Don't accelerate when no keys are held
    particle.acceleration.x = 0.0;

    if world.input_state.is_key_down(KeyCode::KeyI) {
        particle.acceleration.x -= DELTA_ACCEL;
    }
    if world.input_state.is_key_down(KeyCode::KeyO) {
        particle.acceleration.x += DELTA_ACCEL;
    }

    if world.input_state.is_key_pressed(KeyCode::KeyK) {
        particle.velocity.x -= DELTA_VEL;
    }
    if world.input_state.is_key_pressed(KeyCode::KeyL) {
        particle.velocity.x += DELTA_VEL;
    }

    if world.input_state.is_key_pressed(KeyCode::Comma) {
        particle.position.x -= DELTA_POS;
    }
    if world.input_state.is_key_pressed(KeyCode::Period) {
        particle.position.x += DELTA_POS;
    }
}

fn remove_particles(world: &mut World) {
    const DESPAWN_Y: f32 = -10.0;

    world.entities.despawn_all(|body: &Body| {
        let particle = &body.particle;

        particle.position.y < DESPAWN_Y
    });
}

fn draw_particles(world: &mut World) {
    for body in world.entities.each::<Body>() {
        let scale_matrix = Mat3::from_diagonal(body.scale).to_mat4_homogeneous();
        let translation_matrix = Mat4::IDENTITY.with_col(3, body.particle.position.to_vec4(1.0));

        world.draw_commands.push(DrawCommand {
            mesh_name: body.mesh_name.clone(),
            world_matrix: translation_matrix * scale_matrix,
            color: body.color,
        });
    }
}
