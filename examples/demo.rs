use hawthorn::math::{Mat3, Mat4, Quat, Vec3, vec3};
use tumbleweed::app::{App, Body, DrawCommand, Mesh, World};
use tumbleweed::physics::{self, Particle};
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

mod common;

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let mut app = App::new("Tumbleweed App");

    app.create_mesh("sphere", Mesh::new_uv_sphere(5));

    app.add_system(common::move_camera);
    app.add_system(repel_particles);
    app.add_system(update_bouncy_particles);
    app.add_system(physics::physics_pipeline);
    app.add_system(draw_particles);

    const REPELLABLE_POSITIONS: [Vec3; 8] = [
        vec3(1.0, 0.0, 0.0),
        vec3(0.7071, 0.0, 0.7071),
        vec3(0.0, 0.0, 1.0),
        vec3(-0.7071, 0.0, 0.7071),
        vec3(-1.0, 0.0, 0.0),
        vec3(-0.7071, 0.0, -0.7071),
        vec3(0.0, 0.0, -1.0),
        vec3(0.7071, 0.0, -0.7071),
    ];

    for p in REPELLABLE_POSITIONS {
        const RADIUS: f32 = 5.0;

        app.world.entities.spawn(Body {
            tag: "repellable".to_owned(),
            orientation: Quat::IDENTITY,
            scale: Vec3::ONE,
            mesh_name: "sphere".to_owned(),
            color: Vec3::new(0.20, 0.38, 0.56),
            particle: Particle::new_dynamic(1.0)
                .with_position(RADIUS * p)
                .with_damping(0.5),
        });
    }

    const BOUNCY_POSITIONS: [Vec3; 16] = [
        vec3(-0.43, 0.0, -0.46),
        vec3(-0.14, 0.0, -0.39),
        vec3(0.19, 0.0, -0.47),
        vec3(0.44, 0.0, -0.32),
        vec3(-0.37, 0.0, -0.17),
        vec3(-0.08, 0.0, -0.12),
        vec3(0.27, 0.0, -0.21),
        vec3(0.48, 0.0, 0.04),
        vec3(-0.46, 0.0, 0.13),
        vec3(-0.18, 0.0, 0.09),
        vec3(0.13, 0.0, 0.16),
        vec3(0.36, 0.0, 0.28),
        vec3(-0.32, 0.0, 0.41),
        vec3(-0.03, 0.0, 0.35),
        vec3(0.23, 0.0, 0.47),
        vec3(-0.49, 0.0, 0.34),
    ];

    for p in BOUNCY_POSITIONS {
        const SPREAD: f32 = 15.0;
        const OFFSET: Vec3 = Vec3::new(15.0, 0.0, -15.0);
        const MAX_INITIAL_SPEED: f32 = 1.0;

        let init_velocity = vec3(
            rand::random_range(-1.0..=1.0),
            rand::random_range(-1.0..=1.0),
            rand::random_range(-1.0..=1.0),
        );

        app.world.entities.spawn(Body {
            tag: "bouncy".to_owned(),
            orientation: Quat::IDENTITY,
            scale: Vec3::ONE,
            mesh_name: "sphere".to_owned(),
            color: Vec3::new(0.21, 0.46, 0.34),
            particle: Particle::new_dynamic(1.0)
                .with_position(SPREAD * p + OFFSET)
                .with_velocity(MAX_INITIAL_SPEED * init_velocity),
        });
    }

    app.start()
}

fn repel_particles(world: &mut World) {
    const IMPULSE_MAGNITUDE: f32 = 10.0;

    // Apply an impulse to all nearby particles, pushing them away from the camera.
    if world.input_state.is_key_pressed(KeyCode::Space) {
        const MAX_DISTANCE: f32 = 15.0;

        for body in world.entities.each_mut::<Body>() {
            let particle = &mut body.particle;

            let (impulse_direction, distance) = (particle.position - world.camera.position)
                .try_normalize_and_len()
                .unwrap_or((Vec3::NEG_Z, 0.0));

            if distance <= MAX_DISTANCE {
                particle.apply_impulse(impulse_direction * IMPULSE_MAGNITUDE);
            }
        }
    }

    // Apply an impulse to particles clicked by the user.
    if world.input_state.is_mouse_button_pressed(MouseButton::Left) {
        let ray = world
            .camera
            .screen_ray(&world.surface_info, world.input_state.cursor_position());

        let mesh = world.meshes.get("sphere").unwrap();

        if let Some(particle) = world
            .entities
            .each_mut::<Body>()
            .filter_map(|body| {
                let positions = mesh
                    .positions()
                    .map(|p| (body.to_world_matrix() * p.to_vec4(1.0)).xyz())
                    .collect::<Vec<_>>();

                let triangles = mesh.indices().chunks(3).map(|i| {
                    (
                        positions[i[0] as usize],
                        positions[i[1] as usize],
                        positions[i[2] as usize],
                    )
                });

                let intersection = physics::cast_ray(ray, triangles)?;

                Some((intersection, &mut body.particle))
            })
            .min_by(|(a, _), (b, _)| {
                a.point
                    .distance_squared(ray.source)
                    .total_cmp(&b.point.distance_squared(ray.source))
            })
            .map(|(_, particle)| particle)
        {
            particle.apply_impulse(ray.direction * IMPULSE_MAGNITUDE);
        }
    }
}

fn update_bouncy_particles(world: &mut World) {
    const G: f32 = 20.0;
    const BOUNCE_HEIGHT: f32 = -20.0;

    for body in world.entities.each_mut::<Body>() {
        if body.tag != "bouncy" {
            continue;
        }

        let particle = &mut body.particle;

        // Apply gravity
        particle.add_force(G * Vec3::NEG_Y);

        // Do bouncing!
        // You may notice the particles slowly sink into the floor after they stop bouncing.
        // We'll address this over the next couple of episodes in the series.
        if particle.position.y <= BOUNCE_HEIGHT && particle.velocity.y < 0.0 {
            // Non-negative value controlling how strongly the particle bounces back up.
            const BOUNCINESS: f32 = 1.0;

            // When BOUNCINESS is 0.0, the impulse is just large enough to stop the particle
            // from moving into the floor. When BOUNCINESS is 1.0, the particle bounces back
            // upward at the same speed it had when hitting the floor.
            let impulse = (1.0 + BOUNCINESS) * -particle.velocity.y * particle.mass().unwrap();

            particle.apply_impulse(impulse * Vec3::Y);
        }
    }
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
