use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;

#[derive(Component)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub target_distance: f32,
    pub target_yaw: f32,
    pub target_pitch: f32,
    pub last_pinch_dist: Option<f32>,
}

#[derive(Resource, Default)]
pub struct PointerGestureState {
    pub drag_accum: f32,
    pub was_drag: bool,
    pub was_multitouch: bool,
}

pub fn setup_camera(mut commands: Commands) {
    let cam = OrbitCamera {
        target: Vec3::new(0.0, 1.2, 2.5),
        distance: 23.0,
        yaw: -0.62,
        pitch: 0.56,
        target_distance: 23.0,
        target_yaw: -0.62,
        target_pitch: 0.56,
        last_pinch_dist: None,
    };

    let transform = compute_orbit_transform(&cam);

    commands.spawn((
        Camera3d::default(),
        transform,
        DistanceFog {
            color: Color::srgb(0.86, 0.88, 0.90),
            falloff: FogFalloff::Linear {
                start: 34.0,
                end: 82.0,
            },
            ..default()
        },
        cam,
    ));
}

fn compute_orbit_transform(cam: &OrbitCamera) -> Transform {
    let x = cam.distance * cam.pitch.cos() * cam.yaw.sin();
    let y = cam.distance * cam.pitch.sin();
    let z = cam.distance * cam.pitch.cos() * cam.yaw.cos();
    Transform::from_translation(cam.target + Vec3::new(x, y, z)).looking_at(cam.target, Vec3::Y)
}

pub fn update_orbit_camera(
    time: Res<Time>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    touches: Res<Touches>,
    mut gesture: ResMut<PointerGestureState>,
    mut q: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let Ok((mut cam, mut transform)) = q.get_single_mut() else {
        return;
    };

    let active_touches: Vec<&bevy::input::touch::Touch> = touches.iter().collect();
    if active_touches.len() >= 2 {
        gesture.was_multitouch = true;
        let d = active_touches[0]
            .position()
            .distance(active_touches[1].position());
        if let Some(prev) = cam.last_pinch_dist {
            let delta = prev - d;
            cam.target_distance = (cam.target_distance + delta * 0.045).clamp(11.0, 42.0);
        }
        cam.last_pinch_dist = Some(d);
    } else {
        cam.last_pinch_dist = None;
        if active_touches.len() == 1 {
            let t = active_touches[0];
            let delta = t.delta();
            if t.position().distance(t.start_position()) > 14.0 {
                cam.target_yaw -= delta.x * 0.0065;
                cam.target_pitch = (cam.target_pitch + delta.y * 0.0055).clamp(0.18, 1.32);
            }
        } else if touches.iter_just_released().next().is_none() {
            gesture.was_multitouch = false;
        }
    }

    if mouse_buttons.just_pressed(MouseButton::Left) {
        gesture.drag_accum = 0.0;
        gesture.was_drag = false;
    }

    let mut delta_mouse = Vec2::ZERO;
    for ev in mouse_motion.read() {
        delta_mouse += ev.delta;
    }

    if mouse_buttons.pressed(MouseButton::Right) || mouse_buttons.pressed(MouseButton::Middle) {
        cam.target_yaw -= delta_mouse.x * 0.006;
        cam.target_pitch = (cam.target_pitch + delta_mouse.y * 0.005).clamp(0.18, 1.32);
    } else if mouse_buttons.pressed(MouseButton::Left) {
        gesture.drag_accum += delta_mouse.length();
        if gesture.drag_accum > 10.0 {
            gesture.was_drag = true;
            cam.target_yaw -= delta_mouse.x * 0.006;
            cam.target_pitch = (cam.target_pitch + delta_mouse.y * 0.005).clamp(0.18, 1.32);
        }
    }

    for ev in mouse_wheel.read() {
        cam.target_distance = (cam.target_distance - ev.y * 1.6).clamp(11.0, 42.0);
    }

    let k = (time.delta_secs() * 12.0).min(1.0);
    cam.distance = cam.distance.lerp(cam.target_distance, k);
    cam.yaw = cam.yaw.lerp(cam.target_yaw, k);
    cam.pitch = cam.pitch.lerp(cam.target_pitch, k);

    *transform = compute_orbit_transform(&cam);
}
