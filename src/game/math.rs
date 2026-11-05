use std::f32::consts::PI;

use crate::game::cvars::MoveSpeed;

#[inline]
pub fn deg_to_rad(deg: f32) -> f32 {
    deg * PI / 180.0
}

#[inline]
pub fn rad_to_deg(rad: f32) -> f32 {
    rad * 180.0 / PI
}

pub type Vec2 = (f32, f32);

pub fn normalize_vec2(v: Vec2, move_speed: MoveSpeed) -> Option<Vec2> {
    let len_sq = v.0.powi(2) + v.1.powi(2);

    if len_sq <= f32::EPSILON || !len_sq.is_finite() {
        return None;
    }

    let length_y = if v.1 >= 0.0 {
        move_speed.forward
    } else {
        move_speed.back
    };
    let inv_sqrt = 1.0 / len_sq.sqrt();
    Some((v.0 * move_speed.side * inv_sqrt, v.1 * length_y * inv_sqrt))
}

pub type Vec3 = (f32, f32, f32);

pub fn angles_to_forward(pitch_deg: f32, yaw_deg: f32) -> Vec3 {
    let (sp, cp) = deg_to_rad(pitch_deg).sin_cos();
    let (sy, cy) = deg_to_rad(yaw_deg).sin_cos();
    (cp * cy, cp * sy, -sp)
}

pub fn forward_to_angles(fx: f32, fy: f32, fz: f32) -> (f32, f32) {
    let mut yaw = rad_to_deg(fy.atan2(fx));
    if yaw < 0.0 {
        yaw += 360.0;
    }
    let hyp = (fx * fx + fy * fy).sqrt();
    let pitch = rad_to_deg(-fz.atan2(hyp));
    (pitch, yaw)
}

pub fn angle_vectors_full(pitch: f32, yaw: f32, roll: f32) -> (Vec3, Vec3, Vec3) {
    let (sp, cp) = deg_to_rad(pitch).sin_cos();
    let (sy, cy) = deg_to_rad(yaw).sin_cos();
    let (sr, cr) = deg_to_rad(roll).sin_cos();

    let fwd = (cp * cy, cp * sy, -sp);
    let right = (-sr * sp * cy + cr * sy, -sr * sp * sy - cr * cy, -sr * cp);
    let up = (cr * sp * cy + sr * sy, cr * sp * sy - sr * cy, cr * cp);
    (fwd, right, up)
}

pub fn angle_matrix(pitch: f32, yaw: f32, roll: f32, pos: Vec3) -> [[f32; 4]; 4] {
    let (fwd, rgt, up) = angle_vectors_full(pitch, yaw, roll);
    [
        [fwd.0, rgt.0, up.0, pos.0],
        [fwd.1, rgt.1, up.1, pos.1],
        [fwd.2, rgt.2, up.2, pos.2],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub const fn matrix_inverse_tr(m: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let t00 = m[0][0];
    let t01 = m[1][0];
    let t02 = m[2][0];
    let t10 = m[0][1];
    let t11 = m[1][1];
    let t12 = m[2][1];
    let t20 = m[0][2];
    let t21 = m[1][2];
    let t22 = m[2][2];
    let tx = m[0][3];
    let ty = m[1][3];
    let tz = m[2][3];

    let ntx = -(t00 * tx + t01 * ty + t02 * tz);
    let nty = -(t10 * tx + t11 * ty + t12 * tz);
    let ntz = -(t20 * tx + t21 * ty + t22 * tz);

    [
        [t00, t01, t02, ntx],
        [t10, t11, t12, nty],
        [t20, t21, t22, ntz],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub fn mat_mul(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.0f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            let mut s = 0.0f32;
            for k in 0..4 {
                s += a[i][k] * b[k][j];
            }
            out[i][j] = s;
        }
    }
    out
}

pub fn compute_portal_matrix(
    this_pos: Vec3,
    this_ang: Vec3,
    linked_pos: Vec3,
    linked_ang: Vec3,
) -> [[f32; 4]; 4] {
    let this_to_world = angle_matrix(this_ang.0, this_ang.1, this_ang.2, this_pos);
    let linked_to_world = angle_matrix(linked_ang.0, linked_ang.1, linked_ang.2, linked_pos);
    let this_to_world_inv = matrix_inverse_tr(&this_to_world);

    let rot = [
        [-1.0, 0.0, 0.0, 0.0],
        [0.0, -1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    mat_mul(&mat_mul(&linked_to_world, &rot), &this_to_world_inv)
}

pub const fn apply_portal_matrix(mat: &[[f32; 4]; 4], v: Vec3) -> Vec3 {
    let (x, y, z) = v;
    (
        mat[0][0] * x + mat[0][1] * y + mat[0][2] * z + mat[0][3],
        mat[1][0] * x + mat[1][1] * y + mat[1][2] * z + mat[1][3],
        mat[2][0] * x + mat[2][1] * y + mat[2][2] * z + mat[2][3],
    )
}

pub const fn rotate_vec3(mat: &[[f32; 4]; 4], v: Vec3) -> Vec3 {
    let (x, y, z) = v;
    (
        mat[0][0] * x + mat[0][1] * y + mat[0][2] * z,
        mat[1][0] * x + mat[1][1] * y + mat[1][2] * z,
        mat[2][0] * x + mat[2][1] * y + mat[2][2] * z,
    )
}

pub fn invert_affine(mat: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut inv = [[0f32; 4]; 4];
    for r in 0..3 {
        for c in 0..3 {
            inv[r][c] = mat[c][r];
        }
    }
    let tx = mat[0][3];
    let ty = mat[1][3];
    let tz = mat[2][3];
    inv[0][3] = -(inv[0][0] * tx + inv[0][1] * ty + inv[0][2] * tz);
    inv[1][3] = -(inv[1][0] * tx + inv[1][1] * ty + inv[1][2] * tz);
    inv[2][3] = -(inv[2][0] * tx + inv[2][1] * ty + inv[2][2] * tz);
    inv[3] = [0.0, 0.0, 0.0, 1.0];
    inv
}

pub fn reconstruct_angles(fwd: Vec3, rgt: Vec3, up: Vec3) -> Vec3 {
    let (pitch, yaw) = forward_to_angles(fwd.0, fwd.1, fwd.2);
    let cp = (pitch.to_radians()).cos();
    if cp.abs() < 0.001 {
        return (pitch, yaw, 0.0);
    }
    let roll = (-rgt.2).atan2(up.2).to_degrees();
    (pitch, yaw, roll)
}

pub fn calculate_aim_angles(eye: Vec3, target: Vec3) -> (f32, f32) {
    let dx = target.0 - eye.0;
    let dy = target.1 - eye.1;
    let dz = target.2 - eye.2;

    let hyp = (dx.powi(2) + dy.powi(2)).sqrt();

    let mut yaw = rad_to_deg(dy.atan2(dx));
    if yaw < 0.0 {
        yaw += 360.0;
    }

    let pitch = rad_to_deg(-dz.atan2(hyp));
    (pitch, yaw)
}

pub const fn shortest_dyaw(delta: f32) -> f32 {
    let mut d = delta;
    while d > 180.0 {
        d -= 360.0;
    }
    while d <= -180.0 {
        d += 360.0;
    }
    d
}

pub const fn snap(mut x: f32, eps: f32) -> f32 {
    x += 0.0;
    let r = x.round();
    if (x - r).abs() <= eps { r } else { x }
}
