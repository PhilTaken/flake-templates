use bevy::{
    core::FixedTimestep,
    math::{const_vec3, Vec3Swizzles, vec3, vec2},
    prelude::*
};
use rand::prelude::*;

use std::f32::consts;
use bevy_prototype_lyon::prelude::*;

// ------------------------------------------------------------------------

const BACKGROUND_COLOR: Color = Color::BLACK;

const FPS: i32 = 60;
const N_BOIDS: i32 = 100;
const BOID_SCALE: f32 = 15.;
const BOID_SPEED: f32 = 200.;

const BOID_INFLUENCE_RANGE: f32 = 250.;
const BOID_VISION_ANGLE: f32 = 2. * consts::PI / 3.;

// ------------------------------------------------------------------------

const BOID_INFLUENCE_SQ: f32 = BOID_INFLUENCE_RANGE * BOID_INFLUENCE_RANGE;
const TIME_STEP: f32 = 1.0 / FPS as f32;
const BOID_SIZE: Vec3 = const_vec3!([BOID_SCALE, BOID_SCALE, 0.]);

#[derive(Component)]
struct Boid;

#[derive(Component, Debug, PartialEq, Eq)]
struct BoidId(usize);

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct InfluenceCone;


fn main() {
    let debug_system = SystemSet::new()
        .with_system(update_influence_cone);

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        //.add_system_set(debug_system)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(update_rot)
                //.with_system(apply_update))
                .with_system(apply_update.after(update_rot)))
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

fn setup(mut commands: Commands, windows : Res<Windows>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    let window = windows.primary();
    let width = window.width();
    let height = window.height();

    let npoints = 100;
    let mut points: Vec<Vec2> = (0..npoints)
        // spread polygon points equally from -BOID_VISION_ANGLE to BOID_VISION_ANGLE across a cone
        // with radius BOID_INFLUENCE_RANGE
        .map(|n| { 2. * (n as f32 / npoints as f32) - 1. })
        .map(|f| { f * BOID_VISION_ANGLE })
        .map(|f| { BOID_INFLUENCE_RANGE * vec2(f.cos(), f.sin()) })
        .collect();
    // polygons originate at the center of the boid
    points.insert(0, vec2(0., 0.));
    let influence_cone = shapes::Polygon { points, closed: true };

    let boid_sprite = shapes::Polygon {
        points: vec![
            vec2(-0.5, -0.2),
            vec2(0.5, 0.),
            vec2(-0.5, 0.5)
        ],
        closed: true
    };

    for n in 0..N_BOIDS {
        let translation = random_boid_position(width, height);
        let vel = random_boid_velocity();

        commands.spawn()
            .insert(Boid)
            .insert_bundle(GeometryBuilder::build_as(
                &boid_sprite,
                DrawMode::Fill(FillMode::color(Color::rgb(0.0, 0.0, (n as f32) / (N_BOIDS as f32).sqrt()))),
                Transform {
                    scale: BOID_SIZE,
                    translation,
                    ..default()
                }
            ))
            .insert(Velocity(vel))
            .insert(BoidId(n as usize));

        commands.spawn()
            .insert(InfluenceCone)
            .insert(BoidId(n as usize))
            .insert_bundle(GeometryBuilder::build_as(
                &influence_cone,
                DrawMode::Outlined {
                    fill_mode: FillMode::color(Color::NONE),
                    outline_mode: StrokeMode::new(Color::RED, 1.0),
                },
                Transform {
                    translation,
                    ..default()
                },
            ))
            .insert(Visibility { is_visible: false });
    };
}

fn random_boid_position(width: f32, height: f32) -> Vec3 {
    let mut rng = rand::thread_rng();
    let x = width * (rng.gen::<f32>() - 0.5);
    let y = height * (rng.gen::<f32>() - 0.5);
    vec3(x, y, 0.)
}

fn random_boid_velocity() -> Vec2 {
    let mut rng = rand::thread_rng();
    (2. * vec2(rng.gen(), rng.gen()).normalize() - 1.).normalize() * BOID_SPEED
}


// ------------------------------------------------------------------------------

fn update_influence_cone(
    boids: Query<(&Transform, &BoidId), With<Boid>>,
    mut circles: Query<(&mut Transform, &BoidId, &mut Visibility), (With<InfluenceCone>, Without<Boid>)>) {

    circles.for_each_mut(|(mut transf, cid, mut vis)| {
        let boid = boids.iter().find(|(_, bid)| **bid == *cid).unwrap();
        let (boid_transf, _) = boid;
        transf.translation = boid_transf.translation;
        transf.rotation = boid_transf.rotation;

        //transf.scale = vec3(1., 1., 1.);
        vis.is_visible = true;
    });
}


fn update_rot(
    mut set: ParamSet<(
        Query<(&Velocity, &Transform, &BoidId)>,
        Query<(&mut Velocity, &BoidId)>
    )>) {

    let calc_query = set.p0();
    let combination = calc_query.iter_combinations();

    #[allow(unused_variables)]
    let (lot_u, _) = combination
        .filter(|[(_, _, ida), (_, _, idb)]| { ida != idb })
        .filter_map(|[(vel_a, pos_a, id), (vel_b, pos_b, _)]| {
            let pos_diff = pos_b.translation - pos_a.translation;

            if pos_diff.length_squared() < BOID_INFLUENCE_SQ {
                let theta = vel_a.angle_between(pos_diff.xy());
                //let theta = (vel_a.dot(pos_diff.xy()) / (pos_diff.length() * vel_a.length())).acos();

                if theta < BOID_VISION_ANGLE {
                    let vel_diff = vel_b.0 - vel_a.0;
                    Some((id.0, pos_diff, vel_diff, theta))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .filter(|(_, _, _, theta)| { theta < &BOID_VISION_ANGLE })
        .fold(([[vec2(0., 0.);3]; N_BOIDS as usize], [1; N_BOIDS as usize]), |(mut lot_v, mut lot_n), (id, pos_diff, vel_diff, theta)| {
            // averaging factors
            let n = lot_n[id] as f32;
            let div = n - 1.;

            // separation
            let sep_infl = -1. * pos_diff.xy() * TIME_STEP / 2.;
            //let sep_infl = 0.;

            // alignment
            let align_infl = TIME_STEP * vel_diff;

            // cohesion
            let coh_infl = TIME_STEP * pos_diff.xy();

            ////rot_update_lot[my_id.0] = 60. * (sep_infl + align_infl + coh_infl).normalize();

            lot_v[id][0] = (lot_v[id][0] * div + coh_infl) / n;
            lot_v[id][1] = (lot_v[id][1] * div + align_infl) / n;
            lot_v[id][2] += sep_infl;
            lot_n[id] += 1;

            (lot_v, lot_n)
        });

    let vel_update_lot: Vec<Vec2> = lot_u.iter().map(|lot| { lot[0] + lot[1] + lot[2] }).collect();

    let mut boid_query = set.p1();
    for (mut vel, id) in boid_query.iter_mut() {
        vel.0 = (vel.0 + vel_update_lot[id.0]).normalize() * BOID_SPEED;
    };
}

fn apply_update(mut query: Query<(&mut Transform, &Velocity)>, windows : Res<Windows>) {
    let window = windows.primary();
    let span_w = window.width() * 0.5;
    let span_h = window.height() * 0.5;

    for (mut transform, velocity) in query.iter_mut() {
        let pos = &mut transform.translation;

        pos.x += velocity.x * TIME_STEP;
        pos.y += velocity.y * TIME_STEP;

        if pos.x.abs() > span_w + 10. {
            pos.x = -1. * pos.x.signum() * span_w;
        }

        if pos.y.abs() > span_h + 10. {
            pos.y = -1. * pos.y.signum() * span_h;
        }

        // rotate the boids towards their veocity vector
        let vel = velocity.normalize();
        let theta = vel.y.atan2(vel.x);
        transform.rotation = Quat::from_rotation_z(theta + if theta < 0. { 2. * consts::PI } else { 0. });
    }
}
