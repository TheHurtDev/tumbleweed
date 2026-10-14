use hawthorn::math::{Mat3, Mat4, Quat, Vec3};
use tumbleweed::app::{App, Body, DrawCommand, Mesh, World};
use tumbleweed::physics::{self, Particle};
use winit::keyboard::KeyCode;

mod common;

const ATTRACTION_POINT: Vec3 = Vec3::new(0.0, 0.0, 25.0);
const BUNGEE_ANCHOR: Vec3 = Vec3::new(-15.0, 10.0, -20.0);
const ORBITED_POSITION: Vec3 = Vec3::new(15.0, 0.0, -20.0);

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let mut app = App::new("Tumbleweed App");

    app.create_mesh("cube", Mesh::new_flat_shaded_cube());

    app.add_system(common::move_camera);
    app.add_system(apply_attraction_force);
    app.add_system(apply_bungee_forces);
    app.add_system(apply_gravity_in_orbit);
    app.add_system(physics::physics_pipeline);
    app.add_system(update_bungee_cord);
    app.add_system(draw_particles);

    const PARTICLE_POS_AND_VEL: [(Vec3, Vec3); 12] = [
        (
            Vec3::new(0.399653, 0.916667, 0.000000),
            Vec3::new(0.838007, -0.365358, -0.405288),
        ),
        (
            Vec3::new(-0.487724, 0.750000, 0.446795),
            Vec3::new(-0.802105, -0.182958, -0.568466),
        ),
        (
            Vec3::new(0.071010, 0.583333, -0.809123),
            Vec3::new(-0.723029, 0.588915, 0.361121),
        ),
        (
            Vec3::new(0.553107, 0.416667, 0.721430),
            Vec3::new(0.517590, -0.850412, 0.094334),
        ),
        (
            Vec3::new(-0.953445, 0.250000, -0.168651),
            Vec3::new(-0.301404, -0.808433, 0.505562),
        ),
        (
            Vec3::new(0.840820, 0.083333, -0.534861),
            Vec3::new(0.078593, -0.996403, -0.031692),
        ),
        (
            Vec3::new(-0.258701, -0.083333, 0.962356),
            Vec3::new(-0.750249, 0.644869, -0.145841),
        ),
        (
            Vec3::new(-0.446271, -0.250000, -0.859268),
            Vec3::new(-0.676483, -0.534342, 0.506804),
        ),
        (
            Vec3::new(0.853899, -0.416667, 0.311842),
            Vec3::new(-0.044823, -0.655843, -0.753566),
        ),
        (
            Vec3::new(-0.750784, -0.583333, 0.309913),
            Vec3::new(0.009111, 0.459985, 0.887880),
        ),
        (
            Vec3::new(0.280348, -0.750000, -0.599087),
            Vec3::new(-0.923711, -0.041034, -0.380887),
        ),
        (
            Vec3::new(0.119610, -0.916667, 0.381334),
            Vec3::new(0.984886, 0.158015, 0.070924),
        ),
    ];

    // Attractor

    const ATTRACTABLE_MASS_DELTA: f32 = 1.0;

    let mut attractable_mass = 0.0_f32;

    for (i, (position, _)) in PARTICLE_POS_AND_VEL.iter().enumerate() {
        const RADIUS: f32 = 10.0;

        if i % 3 == 0 {
            attractable_mass += ATTRACTABLE_MASS_DELTA;
        }

        app.world.entities.spawn(Body {
            tag: "attractable".to_owned(),
            orientation: Quat::IDENTITY,
            scale: Vec3::splat(attractable_mass),
            mesh_name: "cube".to_owned(),
            color: Vec3::new(
                0.8 / attractable_mass,
                0.8 / attractable_mass,
                1.5 / attractable_mass,
            ),
            particle: Particle::new_dynamic(attractable_mass)
                .with_position(ATTRACTION_POINT + position * RADIUS),
        });
    }

    // Bungee cord

    app.world.entities.spawn(Body {
        tag: "bungee_attached".to_owned(),
        orientation: Quat::IDENTITY,
        scale: Vec3::splat(1.5),
        mesh_name: "cube".to_owned(),
        color: Vec3::new(0.77, 0.64, 0.23),
        particle: Particle::new_dynamic(3.0).with_position(BUNGEE_ANCHOR),
    });

    app.world.entities.spawn(Body {
        tag: "bungee_cord".to_owned(),
        orientation: Quat::IDENTITY,
        scale: Vec3::ONE,
        mesh_name: "cube".to_owned(),
        color: Vec3::new(0.545, 0.353, 0.169),
        particle: Particle::new_static(),
    });

    // Gravity and satellites

    for (position, velocity) in PARTICLE_POS_AND_VEL {
        const SATELLITE_MASS: f32 = 1.0;
        const SATELLITE_SPEED: f32 = 24.0;
        const RADIUS: f32 = 5.0;

        app.world.entities.spawn(Body {
            tag: "satellite".to_owned(),
            orientation: Quat::IDENTITY,
            scale: Vec3::splat(0.7),
            mesh_name: "cube".to_owned(),
            color: Vec3::new(0.5, 0.0, 0.0),
            particle: Particle::new_dynamic(SATELLITE_MASS)
                .with_position(ORBITED_POSITION + position * RADIUS)
                .with_velocity(velocity * SATELLITE_SPEED),
        });
    }

    app.start()
}

/// A generic "attraction" force.
///
/// When the L key is held, a constant force directed towards a certain point is
/// applied to affected bodies.
///
/// When the O key is held, a force directed away from the same point is applied.
fn apply_attraction_force(world: &mut World) {
    const ATTRACTION_STRENGTH: f32 = 8.0;

    for body in world.entities.each_mut::<Body>() {
        if body.tag != "attractable" {
            continue;
        }

        let particle = &mut body.particle;

        let to_attractor = (ATTRACTION_POINT - particle.position).normalize_or_zero();

        let mut force_direction = Vec3::ZERO;

        if world.input_state.is_key_down(KeyCode::KeyL) {
            force_direction += to_attractor;
        }
        if world.input_state.is_key_down(KeyCode::KeyO) {
            force_direction += -to_attractor;
        }

        particle.add_force(ATTRACTION_STRENGTH * force_direction);
    }
}

/// Applies forces simulating a bungee cord.
///
/// There are three forces in effect on the attached body:
/// 1) A spring force representing the bungee cord, applied only
///    when the cord is stretched, but not compressed.
/// 2) A constant gravitational force pulling the body downward.
/// 3) When the player holds the Space bar, an additional force is
///    applied, representing the player pulling on the body.
fn apply_bungee_forces(world: &mut World) {
    // Stiffness of the bungee cord - greater values will increase the
    // amount of spring force.
    const STIFFNESS: f32 = 150.0;
    // The maximum length of the cord at which it applies no spring force.
    const REST_LENGTH: f32 = 15.0;
    // Acceleration due to gravity. Adjust as appropriate - this doesn't
    // have to equal the real value of g on Earth's surface.
    const G: f32 = 80.0;
    // The amount of force applied when the body is pulled by the player.
    const PULL_STRENGTH: f32 = 1_000.0;

    let pull_force = PULL_STRENGTH * Vec3::new(1.0, -1.0, 1.0);

    let Some(body) = world
        .entities
        .each_mut::<Body>()
        .find(|body| body.tag == "bungee_attached")
    else {
        return;
    };

    let particle = &mut body.particle;

    let Some(particle_mass) = particle.mass() else {
        return;
    };

    let (body_to_anchor, bungee_length) = (BUNGEE_ANCHOR - particle.position)
        .try_normalize_and_len()
        .unwrap_or_default();

    let spring_force = if bungee_length > REST_LENGTH {
        let displacement_amount = bungee_length - REST_LENGTH;

        STIFFNESS * displacement_amount * body_to_anchor
    } else {
        Vec3::ZERO
    };

    let gravitational_force = particle_mass * G * Vec3::NEG_Y;

    let pull_force = if world.input_state.is_key_down(KeyCode::Space) {
        pull_force
    } else {
        Vec3::ZERO
    };

    particle.add_force(spring_force);
    particle.add_force(gravitational_force);
    particle.add_force(pull_force);

    // Dampen the velocity to help the body slow to a stop. We'll discuss this technique
    // in more detail in a future episode.
    particle.velocity *= 0.25_f32.powf(world.frame_duration.as_secs_f32());
}

/// Applies gravitational force to orbiting bodies.
fn apply_gravity_in_orbit(world: &mut World) {
    // Adjust as appropriate - this doesn't have to equal the real
    // gravitational constant.
    const BIG_G: f32 = 10.0;
    const ORBITED_MASS: f32 = 288.0;

    for body in world.entities.each_mut::<Body>() {
        if body.tag != "satellite" {
            continue;
        }

        let particle = &mut body.particle;

        let Some(satellite_mass) = particle.mass() else {
            continue;
        };

        let Some((direction, distance)) =
            (ORBITED_POSITION - particle.position).try_normalize_and_len()
        else {
            continue;
        };

        let gravitational_force =
            direction * BIG_G * ORBITED_MASS * satellite_mass / distance.powi(2);

        particle.add_force(gravitational_force);
    }
}

/// Updates the transform of the bungee cord rendered between the anchor point and the
/// attached body.
///
/// The bungee cord itself is not physically simulated in this demo. It only reacts to
/// the movement of the attached body.
fn update_bungee_cord(world: &mut World) {
    const CORD_WIDTH: f32 = 0.25;

    let Some(attachment_position) = world
        .entities
        .each_mut::<Body>()
        .find(|body| body.tag == "bungee_attached")
        .map(|body| body.particle.position)
    else {
        return;
    };

    let Some(bungee_cord) = world
        .entities
        .each_mut::<Body>()
        .find(|body| body.tag == "bungee_cord")
    else {
        return;
    };

    bungee_cord.particle.position = (BUNGEE_ANCHOR + attachment_position) / 2.0;

    let to_anchor = BUNGEE_ANCHOR - attachment_position;

    bungee_cord.orientation = Quat::try_look_to_rh(to_anchor, Vec3::Y)
        .unwrap_or_else(|| Quat::from_rotation_x(90.0_f32.to_radians()));

    bungee_cord.scale = Vec3::new(CORD_WIDTH, CORD_WIDTH, to_anchor.len());
}

fn draw_particles(world: &mut World) {
    for body in world.entities.each::<Body>() {
        let scale_matrix = Mat3::from_diagonal(body.scale).to_mat4_homogeneous();
        let rotation_matrix = body.orientation.to_rotation_mat4();
        let translation_matrix = Mat4::IDENTITY.with_col(3, body.particle.position.to_vec4(1.0));

        world.draw_commands.push(DrawCommand {
            mesh_name: body.mesh_name.clone(),
            world_matrix: translation_matrix * rotation_matrix * scale_matrix,
            color: body.color,
        });
    }
}
