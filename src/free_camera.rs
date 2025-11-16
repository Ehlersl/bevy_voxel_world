use std::f32::consts::{FRAC_PI_2, PI};
use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, Startup, Update},
    camera::{Camera3d, PerspectiveProjection, Projection},
    ecs::{
        component::Component,
        entity::Entity,
        name::Name,
        query::With,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    input::{
        ButtonInput,
        keyboard::KeyCode,
        mouse::AccumulatedMouseMotion,
    },
    math::{EulerRot, Quat, Vec2, Vec3},
    time::Time,
    transform::components::Transform,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::voxel_world::VoxelWorldCamera;

pub struct FreeCamPlugin<W> {
    pub settings: FreeCamSettings,
    pub spawn_on_startup: bool,
    _marker: PhantomData<W>,
}

impl<W> Default for FreeCamPlugin<W> {
    fn default() -> Self {
        Self {
            settings: FreeCamSettings::default(),
            spawn_on_startup: true,
            _marker: PhantomData,
        }
    }
}

impl<W> FreeCamPlugin<W> {
    pub fn with_settings(settings: FreeCamSettings) -> Self {
        Self {
            settings,
            spawn_on_startup: true,
            _marker: PhantomData,
        }
    }

    pub fn with_settings_no_spawn(settings: FreeCamSettings) -> Self {
        Self {
            settings,
            spawn_on_startup: false,
            _marker: PhantomData,
        }
    }
}

impl<W> Plugin for FreeCamPlugin<W>
where
    W: Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(self.settings.clone())
            .insert_resource(MouseLocked(true))
            .add_systems(
                Update,
                (
                    freecam_mouse_look,
                    freecam_mouse_lock,
                    freecam_move,
                    freecam_zoom,
                ),
            );

        if self.spawn_on_startup {
            app.add_systems(Startup, spawn_freecam::<W>);
        }
    }
}

#[derive(Resource)]
struct MouseLocked(bool);

#[derive(Component)]
pub struct FreeCam;

#[derive(Resource, Clone)]
pub struct FreeCamSettings {
    /// m/s
    pub speed: f32,
    /// m/s
    pub speed_fast: f32,
    /// Maus-Sensitivität
    pub mouse_sensitivity: Vec2,
    /// FOV Limit
    pub fov_range: std::ops::Range<f32>,
    /// Schrittweite fürs FOV pro Tick der Pfeile
    pub fov_step: f32,
}

impl Default for FreeCamSettings {
    fn default() -> Self {
        Self {
            speed: 8.0,
            speed_fast: 16.0,
            mouse_sensitivity: Vec2::new(0.003, 0.002),
            fov_range: (PI / 5.0)..(PI - 0.2), // 36° .. 160°
            fov_step: 0.8_f32.to_radians(),    // ~0.014 rad ≈ 0.8°
        }
    }
}

fn spawn_freecam<W>(
    mut commands: Commands,
    q_cam: Query<Entity, With<FreeCam>>,
) where
    W: Send + Sync + 'static,
{
    if !q_cam.is_empty() {
        return;
    }

    commands.spawn((
        Name::new("FreeCam"),
        FreeCam,
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: PI / 3.0, // 60°
            ..Default::default()
        }),
        Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        VoxelWorldCamera::<W>::default(),
    ));
}

fn freecam_mouse_lock(
    kb: Res<ButtonInput<KeyCode>>,
    mut q_cursor: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut locked: ResMut<MouseLocked>,
) {
    let Ok(mut cursor) = q_cursor.single_mut() else {
        return;
    };

    if kb.just_pressed(KeyCode::Escape) {
        locked.0 = !locked.0;
    }

    if locked.0 {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    } else {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
}

fn freecam_mouse_look(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    mut q_cam: Query<(&mut Transform, &mut Projection), With<FreeCam>>,
    settings: Res<FreeCamSettings>,
) {
    let delta = accumulated_mouse_motion.delta;
    if delta == Vec2::ZERO {
        return;
    }

    if let Ok((mut transform, _proj)) = q_cam.single_mut() {
        let (mut yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);

        let delta_yaw = -delta.x * settings.mouse_sensitivity.x;
        let delta_pitch = -delta.y * settings.mouse_sensitivity.y;

        yaw += delta_yaw;

        const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
        let new_pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, new_pitch, roll);
    }
}

/// WASD bewegen (relativ zur Blickrichtung), Shift = schneller.
fn freecam_move(
    time: Res<Time>,
    kb: Res<ButtonInput<KeyCode>>,
    settings: Res<FreeCamSettings>,
    mut q_cam: Query<&mut Transform, With<FreeCam>>,
) {
    let Ok(mut transform) = q_cam.single_mut() else {
        return;
    };

    let mut input = Vec3::ZERO;

    // Vor/Zurück + Links/Rechts (in Kamera-Ebene)
    if kb.pressed(KeyCode::KeyW) {
        input.z += 1.0;
    }
    if kb.pressed(KeyCode::KeyS) {
        input.z -= 1.0;
    }
    if kb.pressed(KeyCode::KeyA) {
        input.x -= 1.0;
    }
    if kb.pressed(KeyCode::KeyD) {
        input.x += 1.0;
    }
    if kb.pressed(KeyCode::ShiftLeft) || kb.pressed(KeyCode::ShiftRight) {
        input.y -= 1.0;
    }
    if kb.pressed(KeyCode::Space) {
        input.y += 1.0;
    }

    if input == Vec3::ZERO {
        return;
    }

    // Richtung relativ zur Kamera
    let forward = transform.forward().normalize_or_zero();
    let right = transform.right();
    let up = Vec3::Y;

    let world_dir = (right * input.x) + (forward * input.z) + (up * input.y);
    let speed = if kb.pressed(KeyCode::ControlLeft) || kb.pressed(KeyCode::ControlRight) {
        settings.speed_fast
    } else {
        settings.speed
    };

    transform.translation += world_dir.normalize_or_zero() * speed * time.delta_secs();
}

fn freecam_zoom(
    kb: Res<ButtonInput<KeyCode>>,
    settings: Res<FreeCamSettings>,
    mut q_cam: Query<&mut Projection, With<FreeCam>>,
) {
    let Ok(mut proj) = q_cam.single_mut() else {
        return;
    };

    if let Projection::Perspective(ref mut persp) = *proj {
        let mut fov = persp.fov;

        let mut changed = false;
        if kb.pressed(KeyCode::ArrowUp) {
            fov -= settings.fov_step;
            changed = true;
        }
        if kb.pressed(KeyCode::ArrowDown) {
            fov += settings.fov_step;
            changed = true;
        }

        if changed {
            persp.fov = fov.clamp(settings.fov_range.start, settings.fov_range.end);
        }
    }
}
