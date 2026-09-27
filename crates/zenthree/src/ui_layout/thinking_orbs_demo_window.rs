//! Dedicated Floating Showcase Window for Jakubantalik/thinking-orbs
//! Implements all 9 official animation modes with the exact direct mathematics:
//! 1. working     (orbits.ts)
//! 2. searching   (lattice.ts - globe)
//! 3. solving     (lattice.ts - rubik)
//! 4. listening   (lattice.ts - wave)
//! 5. connecting  (web.ts)
//! 6. weaving     (braid.ts)
//! 7. composing   (ribbon.ts)
//! 8. breathing   (ribbon.ts - ring)
//! 9. shaping     (morph.ts)

use crate::state::ZenthreeState;
use crate::theme::ThemeColors;
use zenthra::{use_context, Align, Color, CursorIcon, FontWeight, Id, Ui};
use zenthra_render::RectInstance;
use zenthra_widgets::ui::{DrawCommand, RectDraw};

// ============================================================================
// 1. DATA STRUCTURES & ENGINE CONSTANTS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrbStateKind {
    Working,
    Searching,
    Solving,
    Listening,
    Connecting,
    Weaving,
    Composing,
    Breathing,
    Shaping,
}

impl OrbStateKind {
    pub const ALL: [OrbStateKind; 9] = [
        OrbStateKind::Working,
        OrbStateKind::Searching,
        OrbStateKind::Solving,
        OrbStateKind::Listening,
        OrbStateKind::Connecting,
        OrbStateKind::Weaving,
        OrbStateKind::Composing,
        OrbStateKind::Breathing,
        OrbStateKind::Shaping,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            OrbStateKind::Working => "working",
            OrbStateKind::Searching => "searching",
            OrbStateKind::Solving => "solving",
            OrbStateKind::Listening => "listening",
            OrbStateKind::Connecting => "connecting",
            OrbStateKind::Weaving => "weaving",
            OrbStateKind::Composing => "composing",
            OrbStateKind::Breathing => "breathing",
            OrbStateKind::Shaping => "shaping",
        }
    }

    pub fn subtitle(&self) -> &'static str {
        match self {
            OrbStateKind::Working => "Orbits · Particles on tilted orbital planes",
            OrbStateKind::Searching => "Globe · Meridian scan wave sweeping a dotted sphere",
            OrbStateKind::Solving => "Rubik · Quarter-turn band twists scramble & solve",
            OrbStateKind::Listening => "Wave · Traveling sinusoidal ripples through rings",
            OrbStateKind::Connecting => "Web · Constellation nodes with proximity edges & signal packets",
            OrbStateKind::Weaving => "Braid · Three helical strands plaiting pole to pole",
            OrbStateKind::Composing => "Ribbon · Undulating multi-lane sash on great circle",
            OrbStateKind::Breathing => "Ring · Pure face-on pulsing radial harmonic ring",
            OrbStateKind::Shaping => "Morph · Continuous closed path: Circle → Triangle → Square",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Dot {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
    pub white: f32,
    pub a: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub white: f32,
    pub a: f32,
    pub w: f32,
}

#[derive(Debug, Clone, Default)]
pub struct OrbFrame {
    pub dots: Vec<Dot>,
    pub lines: Vec<Line>,
}

// ============================================================================
// 2. MATHEMATICAL CORE ENGINE (verbatim port from src/engine/core.ts)
// ============================================================================

pub fn lerp(a: f32, b: f32, f: f32) -> f32 {
    a + (b - a) * f
}

pub fn frac(x: f32) -> f32 {
    x - x.floor()
}

pub fn hash_d(a: f32, b: f32) -> f32 {
    let h = (a * 12.9898 + b * 78.233).sin() * 43758.5453;
    h - h.floor()
}

pub fn vnoise(x: f32, y: f32) -> f32 {
    let xi = x.floor();
    let yi = y.floor();
    let mut fx = x - xi;
    let mut fy = y - yi;
    fx = fx * fx * (3.0 - 2.0 * fx);
    fy = fy * fy * (3.0 - 2.0 * fy);
    let a = hash_d(xi, yi);
    let b = hash_d(xi + 1.0, yi);
    let c = hash_d(xi, yi + 1.0);
    let d = hash_d(xi + 1.0, yi + 1.0);
    a + (b - a) * fx + (c - a) * fy + (a - b - c + d) * fx * fy
}

pub fn fib_dir(i: usize, n: usize) -> [f32; 3] {
    let golden = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
    let y = 1.0 - (2.0 * (i as f32 + 0.5)) / n.max(1) as f32;
    let rad = (1.0 - y * y).max(0.0).sqrt();
    let a = i as f32 * golden;
    [rad * a.cos(), y, rad * a.sin()]
}

pub fn angle_delta(a: f32, b: f32) -> f32 {
    (a - b).sin().atan2((a - b).cos())
}

pub fn radius_scale(size: f32, pow: f32) -> f32 {
    (size / 300.0).powf(pow)
}

pub fn project_3d(yaw: f32, tilt: f32, cx: f32, cy: f32, scale: f32, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
    let st = tilt.sin();
    let ct = tilt.cos();
    let sy = yaw.sin();
    let cyw = yaw.cos();
    let x1 = x * cyw + z * sy;
    let z1 = -x * sy + z * cyw;
    let y1 = y * ct - z1 * st;
    let z2 = y * st + z1 * ct;
    (cx + x1 * scale, cy - y1 * scale, z2)
}

pub fn finalize_frame(mut dots: Vec<Dot>, lines: Vec<Line>, r_min: f32) -> OrbFrame {
    dots.retain(|d| d.a >= 0.02);
    for d in &mut dots {
        d.r = d.r.max(r_min);
    }
    dots.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal));
    let filtered_lines = lines.into_iter().filter(|l| l.a >= 0.02).collect();
    OrbFrame {
        dots,
        lines: filtered_lines,
    }
}

// ============================================================================
// 3. MODE IMPLEMENTATIONS (from src/engine/orbits, lattice, braid, ribbon, morph, web)
// ============================================================================

/// 1. Working (orbits.ts)
pub fn frame_orbits(size: f32, t: f32, size_factor: f32) -> OrbFrame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let r = (size / 2.0) * 0.82;
    let rs = radius_scale(size, 0.6) * size_factor;

    let orbit_n = if size < 40.0 { 3 } else { 12 };
    let ghost_n = if size < 40.0 { 10 } else { 40 };
    let particles = if size < 40.0 { 1 } else { 3 };

    let mut dots = Vec::with_capacity(orbit_n * (ghost_n + particles));
    for orb in 0..orbit_n {
        let h1 = hash_d(orb as f32, 1.7);
        let h2 = hash_d(orb as f32, 5.2);
        let h3 = hash_d(orb as f32, 8.9);
        let ro = r * (0.45 + 0.52 * h1);
        let th = h1 * 2.0 * std::f32::consts::PI;
        let phi = (2.0 * h2 - 1.0).clamp(-1.0, 1.0).acos();

        let nx = phi.sin() * th.cos();
        let ny = phi.cos();
        let nz = phi.sin() * th.sin();
        let mut ux = -ny;
        let mut uy = nx;
        let uz = 0.0;
        let ul = (ux * ux + uy * uy).sqrt().max(1e-6);
        ux /= ul;
        uy /= ul;
        let vx = ny * uz - nz * uy;
        let vy = nz * ux - nx * uz;
        let vz = nx * uy - ny * ux;
        let speed = (0.25 + 0.55 * h3) * if h3 > 0.5 { 1.0 } else { -1.0 };

        // Ghost orbital ring
        for k in 0..ghost_n {
            let a = (k as f32 / ghost_n as f32) * 2.0 * std::f32::consts::PI;
            let (px, py, z) = project_3d(
                t * 0.12, 0.30, cx, cy, 1.0,
                (ux * a.cos() + vx * a.sin()) * ro,
                (uy * a.cos() + vy * a.sin()) * ro,
                (uz * a.cos() + vz * a.sin()) * ro,
            );
            let depth = ((z / ro + 1.0) / 2.0).clamp(0.0, 1.0);
            dots.push(Dot {
                x: px,
                y: py,
                z,
                r: 0.9 * rs,
                white: 0.72,
                a: 0.5 * (0.4 + 0.6 * depth),
            });
        }

        // Active particles
        for m in 0..particles {
            let a = t * speed + (m as f32 / particles as f32) * 2.0 * std::f32::consts::PI + h2 * 6.0;
            let (px, py, z) = project_3d(
                t * 0.12, 0.30, cx, cy, 1.0,
                (ux * a.cos() + vx * a.sin()) * ro,
                (uy * a.cos() + vy * a.sin()) * ro,
                (uz * a.cos() + vz * a.sin()) * ro,
            );
            let depth = ((z / ro + 1.0) / 2.0).clamp(0.0, 1.0);
            dots.push(Dot {
                x: px,
                y: py,
                z,
                r: (1.2 + 1.6 * depth) * rs,
                white: 0.3 - 0.22 * depth,
                a: 1.0,
            });
        }
    }
    finalize_frame(dots, Vec::new(), 0.3)
}

/// 2. Searching (lattice.ts - frameGlobe)
pub fn frame_globe(size: f32, t: f32, size_factor: f32) -> OrbFrame {
    let spin = 0.5;
    let cx = size / 2.0;
    let cy = size / 2.0;
    let radius = (size / 2.0) * 0.82;
    let tilt = 0.4 + 0.06 * (t * 0.35).sin();
    let scan = t * (spin + (1.7 - spin) * 4.08);
    let rs = radius_scale(size, 0.6) * size_factor;
    let dim_base = 0.45;

    let lat_rings = if size < 40.0 { 6 } else { 12 };
    let lon_density = if size < 40.0 { 15 } else { 30 };

    let mut dots = Vec::with_capacity((lat_rings + 1) * lon_density);
    for li in 0..=lat_rings {
        let lat = -std::f32::consts::FRAC_PI_2 + (li as f32 / lat_rings as f32) * std::f32::consts::PI;
        let cos_lat = lat.cos();
        let sin_lat = lat.sin();
        let lon_count = (cos_lat.abs() * lon_density as f32).round().max(1.0) as usize;

        for lj in 0..lon_count {
            let lon = (lj as f32 / lon_count as f32) * 2.0 * std::f32::consts::PI;
            let (px, py, z) = project_3d(
                t * spin, tilt, cx, cy, radius,
                cos_lat * lon.cos(), sin_lat, cos_lat * lon.sin(),
            );
            let depth = (z + 1.0) / 2.0;
            let d = angle_delta(lon + t * spin, scan);
            let boost = (-(d * d) / 0.18).exp() * z.max(0.0);
            dots.push(Dot {
                x: px,
                y: py,
                z,
                r: (0.6 + 1.7 * depth + 1.0 * boost) * rs,
                white: 0.62 - 0.54 * depth,
                a: dim_base + (1.0 - dim_base) * boost.min(1.0),
            });
        }
    }
    finalize_frame(dots, Vec::new(), 0.3)
}

/// 3. Solving (lattice.ts - frameRubik)
#[derive(Debug, Clone, Copy)]
struct Move {
    axis: usize, // 0, 1, or 2
    lo: f32,
    hi: f32,
    ang: f32,
}

static MOVES: std::sync::OnceLock<[Move; 14]> = std::sync::OnceLock::new();

fn get_moves() -> &'static [Move; 14] {
    MOVES.get_or_init(|| {
        let mut moves = [Move { axis: 0, lo: 0.0, hi: 0.0, ang: 0.0 }; 14];
        for i in 0..14 {
            let axis = ((hash_d(i as f32, 2.3) * 3.0).floor() as usize).min(2);
            let lo = -1.0 + 0.5 * ((hash_d(i as f32, 5.9) * 4.0).floor() as f32).min(3.0);
            let dir = if hash_d(i as f32, 7.7) < 0.5 { 1.0 } else { -1.0 };
            moves[i] = Move {
                axis,
                lo,
                hi: lo + 0.5,
                ang: dir * std::f32::consts::FRAC_PI_2,
            };
        }
        moves
    })
}

fn solve_cycle(time: f32, count: usize, slot_dur: f32, rest: f32) -> ([f32; 14], i32) {
    let cyc = 2.0 * count as f32 * slot_dur + rest;
    let tc = time % cyc;
    let mut amount = [0.0f32; 14];
    let mut active = -1;
    let anim_dur = 2.0 * count as f32 * slot_dur;
    if tc < anim_dur {
        let slot = ((tc / slot_dur).floor() as usize).min(2 * count - 1);
        let p = ((tc - slot as f32 * slot_dur) / slot_dur).clamp(0.0, 1.0);
        // Silky smooth sine ease-in-out without abrupt stop halts
        let ep = (1.0 - (p * std::f32::consts::PI).cos()) * 0.5;
        if slot < count {
            for i in 0..slot {
                amount[i] = 1.0;
            }
            amount[slot] = ep;
            active = slot as i32;
        } else {
            let u = 2 * count - 1 - slot;
            for i in 0..u {
                amount[i] = 1.0;
            }
            amount[u] = 1.0 - ep;
            active = u as i32;
        }
    }
    (amount, active)
}

fn apply_moves(
    mut pt: [f32; 3],
    moves: &[Move; 14],
    amount: &[f32; 14],
    active: i32,
) -> ([f32; 3], bool) {
    let mut in_active = false;
    let max_idx = if active >= 0 { (active as usize + 1).min(14) } else { 14 };
    for i in 0..max_idx {
        if amount[i] <= 0.0001 {
            continue;
        }
        let mv = &moves[i];
        let coord = match mv.axis {
            0 => pt[0],
            1 => pt[1],
            _ => pt[2],
        };
        if coord < mv.lo || coord >= mv.hi {
            continue;
        }
        if i as i32 == active {
            in_active = true;
        }
        let a = mv.ang * amount[i];
        let ca = a.cos();
        let sa = a.sin();
        if mv.axis == 0 {
            let y2 = pt[1] * ca - pt[2] * sa;
            pt[2] = pt[1] * sa + pt[2] * ca;
            pt[1] = y2;
        } else if mv.axis == 1 {
            let x2 = pt[0] * ca + pt[2] * sa;
            pt[2] = -pt[0] * sa + pt[2] * ca;
            pt[0] = x2;
        } else {
            let x2 = pt[0] * ca - pt[1] * sa;
            pt[1] = pt[0] * sa + pt[1] * ca;
            pt[0] = x2;
        }
    }
    (pt, in_active)
}

pub fn frame_rubik(size: f32, t: f32, size_factor: f32) -> OrbFrame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let r = (size / 2.0) * 0.82;
    let tilt = 0.35 + 0.1 * (t * 0.9).sin();
    let rs = radius_scale(size, 0.6) * size_factor;
    let move_count = 14;
    let moves = get_moves();
    let (amount, active) = solve_cycle(t, move_count, 0.42, 1.2);

    let lat_rings = if size < 40.0 { 6 } else { 12 };
    let lon_density = if size < 40.0 { 14 } else { 28 };

    let mut dots = Vec::with_capacity((lat_rings + 1) * lon_density);
    for li in 0..=lat_rings {
        let lat = -std::f32::consts::FRAC_PI_2 + (li as f32 / lat_rings as f32) * std::f32::consts::PI;
        let cos_lat = lat.cos();
        let sin_lat = lat.sin();
        let lon_count = (cos_lat.abs() * lon_density as f32).round().max(1.0) as usize;

        for lj in 0..lon_count {
            let lon = (lj as f32 / lon_count as f32) * 2.0 * std::f32::consts::PI;
            let (m_pt, in_active) = apply_moves(
                [cos_lat * lon.cos(), sin_lat, cos_lat * lon.sin()],
                moves,
                &amount,
                active,
            );
            let (px, py, zr) = project_3d(t * 0.55, tilt, cx, cy, r, m_pt[0], m_pt[1], m_pt[2]);
            let depth = (zr + 1.0) / 2.0;
            dots.push(Dot {
                x: px,
                y: py,
                z: zr,
                r: (0.6 + 1.7 * depth + if in_active { 0.3 } else { 0.0 }) * rs,
                white: 0.62 - 0.54 * depth - if in_active { 0.14 } else { 0.0 },
                a: 1.0,
            });
        }
    }
    finalize_frame(dots, Vec::new(), 0.3)
}

/// 4. Listening (lattice.ts - frameWave)
pub fn frame_wave(size: f32, t: f32, size_factor: f32) -> OrbFrame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let r = (size / 2.0) * 0.874;
    let rs = radius_scale(size, 0.6) * size_factor;
    let rings = if size < 40.0 { 6 } else { 12 };
    let lon_density = if size < 40.0 { 14 } else { 28 };

    let mut dots = Vec::with_capacity((rings + 1) * lon_density);
    for ri in 0..=rings {
        let lat = -std::f32::consts::FRAC_PI_2 + (ri as f32 / rings as f32) * std::f32::consts::PI;
        let cos_lat = lat.cos();
        let sin_lat = lat.sin();
        let w = 0.62 * (t * 2.1 - ri as f32 * 0.52).sin() + 0.38 * (t * 1.27 + ri as f32 * 0.83).sin();
        let rr = r * (0.88 + 0.105 * w);
        let lon_count = (cos_lat.abs() * lon_density as f32).round().max(1.0) as usize;

        for lj in 0..lon_count {
            let lon = (lj as f32 / lon_count as f32) * 2.0 * std::f32::consts::PI;
            let (px, py, z) = project_3d(
                t * 0.18, 0.38, cx, cy, 1.0,
                cos_lat * lon.cos() * rr,
                sin_lat * rr,
                cos_lat * lon.sin() * rr,
            );
            let depth = (z / r + 1.0) / 2.0;
            let crest = w.max(0.0);
            dots.push(Dot {
                x: px,
                y: py,
                z,
                r: (0.6 + 1.7 * depth) * (1.0 + 0.4 * crest) * rs,
                white: 0.66 - 0.56 * depth - 0.1 * crest,
                a: 1.0,
            });
        }
    }
    finalize_frame(dots, Vec::new(), 0.3)
}

/// 5. Connecting (web.ts - frameWeb)
pub fn frame_web(size: f32, t: f32, size_factor: f32) -> OrbFrame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let r = (size / 2.0) * 0.80;
    let rs = radius_scale(size, 0.6) * size_factor;
    let node_n = if size < 40.0 { 12 } else { 30 };
    let thr = 0.72;
    let node_r = 1.4;
    let node_r_depth = 1.8;

    let mut nodes = Vec::with_capacity(node_n);
    for i in 0..node_n {
        let d = fib_dir(i, node_n);
        let x = d[0] + 0.3 * (vnoise(i as f32 * 0.31 + 9.0, t * 0.24) - 0.5) * 2.0;
        let y = d[1] + 0.3 * (vnoise(i as f32 * 0.53 + 27.0, t * 0.21) - 0.5) * 2.0;
        let z = d[2] + 0.3 * (vnoise(i as f32 * 0.77 + 55.0, t * 0.27) - 0.5) * 2.0;
        let l = (x * x + y * y + z * z).sqrt().max(1e-6);
        nodes.push([x / l, y / l, z / l]);
    }

    let mut lines = Vec::new();
    let mut dots = Vec::with_capacity(node_n + 20);

    for i in 0..node_n {
        for j in (i + 1)..node_n {
            let dx = nodes[i][0] - nodes[j][0];
            let dy = nodes[i][1] - nodes[j][1];
            let dz = nodes[i][2] - nodes[j][2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            if dist >= thr {
                continue;
            }
            let (x1, y1, z1) = project_3d(t * 0.12, 0.32, cx, cy, r, nodes[i][0], nodes[i][1], nodes[i][2]);
            let (x2, y2, z2) = project_3d(t * 0.12, 0.32, cx, cy, r, nodes[j][0], nodes[j][1], nodes[j][2]);
            let depth = ((z1 + z2) / 2.0 + 1.0) / 2.0;
            lines.push(Line {
                x1,
                y1,
                x2,
                y2,
                white: 0.42,
                a: (1.0 - dist / thr) * (0.3 + 0.55 * depth),
                w: (0.8 * rs).max(0.6),
            });
        }
    }

    for i in 0..node_n {
        let (px, py, z) = project_3d(t * 0.12, 0.32, cx, cy, r, nodes[i][0], nodes[i][1], nodes[i][2]);
        let depth = (z + 1.0) / 2.0;
        let pulse = 1.0 + 0.25 * (t * 1.4 + i as f32 * 2.7).sin();
        dots.push(Dot {
            x: px,
            y: py,
            z,
            r: (node_r + node_r_depth * depth) * pulse * rs,
            white: 0.55 - 0.45 * depth,
            a: 1.0,
        });
    }

    // Signals: bright packets running between paired nodes
    let signals = if size < 40.0 { 2 } else { 5 };
    for s in 0..signals {
        let phase = t * 0.55 + s as f32 * 7.31;
        let seg = phase.floor();
        let a_idx = (hash_d(seg, s as f32 * 3.1 + 1.7) * node_n as f32).floor() as usize % node_n;
        let b_idx = (hash_d(seg, s as f32 * 5.7 + 4.2) * node_n as f32).floor() as usize % node_n;
        if a_idx == b_idx {
            continue;
        }
        let f = frac(phase);
        let x = lerp(nodes[a_idx][0], nodes[b_idx][0], f);
        let y = lerp(nodes[a_idx][1], nodes[b_idx][1], f);
        let z = lerp(nodes[a_idx][2], nodes[b_idx][2], f);
        let l = (x * x + y * y + z * z).sqrt().max(1e-6);
        let (px, py, zr) = project_3d(t * 0.12, 0.32, cx, cy, r, x / l, y / l, z / l);
        let depth = (zr + 1.0) / 2.0;
        dots.push(Dot {
            x: px,
            y: py,
            z: zr,
            r: (node_r * 1.5 + node_r_depth * depth) * rs,
            white: 0.05, // very bright
            a: 0.5 + 0.5 * depth,
        });
    }

    finalize_frame(dots, lines, 0.3)
}

/// 6. Weaving (braid.ts - frameBraid)
pub fn frame_braid(size: f32, t: f32, size_factor: f32) -> OrbFrame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let r = (size / 2.0) * 0.76;
    let rs = radius_scale(size, 0.6) * size_factor;
    let ghost_n = if size < 40.0 { 30 } else { 80 };
    let strand_n = if size < 40.0 { 18 } else { 42 };
    let turns = 3.0;

    let mut dots = Vec::with_capacity(ghost_n + strand_n * 3);
    for i in 0..ghost_n {
        let d = fib_dir(i, ghost_n);
        let (px, py, z) = project_3d(t * 0.4, 0.3, cx, cy, 1.0, d[0] * r, d[1] * r, d[2] * r);
        let depth = (z / r + 1.0) / 2.0;
        dots.push(Dot {
            x: px,
            y: py,
            z,
            r: 0.8 * rs,
            white: 0.78,
            a: 0.1 + 0.22 * depth,
        });
    }

    for s in 0..3 {
        let phase = (s as f32 / 3.0) * 2.0 * std::f32::consts::PI;
        for i in 0..strand_n {
            let u = (frac(i as f32 / strand_n as f32 + t * 0.045) * 2.0 - 1.0) * 0.96;
            let surf = (1.0 - u * u).max(0.0).sqrt();
            let end_fade = ((1.0 - u.abs()) / 0.1).min(1.0);
            let a = u * std::f32::consts::PI * turns + phase;
            let weave = 1.0 + 0.075 * (u * std::f32::consts::PI * turns * 2.0 + phase * 2.0 + t * 0.8).sin();
            let rr = surf * r * weave;
            let (px, py, zr) = project_3d(
                t * 0.4, 0.3, cx, cy, 1.0,
                a.cos() * rr, u * r * weave, a.sin() * rr,
            );
            let depth = (zr / r + 1.0) / 2.0;
            dots.push(Dot {
                x: px,
                y: py,
                z: zr,
                r: (1.2 + 1.8 * depth) * rs,
                white: 0.55 - 0.45 * depth,
                a: end_fade * (0.45 + 0.55 * depth),
            });
        }
    }
    finalize_frame(dots, Vec::new(), 0.3)
}

/// 7. Composing (ribbon.ts) & 8. Breathing (ribbon.ts with face_on=true)
pub fn frame_ribbon_or_ring(size: f32, t: f32, face_on: bool, size_factor: f32) -> OrbFrame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let r = (size / 2.0) * 0.78;
    let spin = if face_on { 0.0 } else { 0.0 };
    let cam_tilt = 0.3;
    let rs = radius_scale(size, 0.6) * size_factor;
    let ghost_n = if face_on { 0 } else if size < 40.0 { 20 } else { 70 };

    let mut dots = Vec::with_capacity(ghost_n + 300);
    for i in 0..ghost_n {
        let d = fib_dir(i, ghost_n);
        let (px, py, z) = project_3d(t * 0.1 * spin, cam_tilt, cx, cy, 1.0, d[0] * r, d[1] * r, d[2] * r);
        let depth = (z / r + 1.0) / 2.0;
        dots.push(Dot {
            x: px,
            y: py,
            z,
            r: 0.8 * rs,
            white: 0.78,
            a: 0.1 + 0.22 * depth,
        });
    }

    let ya = t * 0.24 * spin;
    let ta = if face_on { -cam_tilt } else { 0.55 + 0.3 * (t * 0.18).sin() * spin };
    let ux = ya.cos();
    let uy = 0.0;
    let uz = ya.sin();
    let vx = -uz * ta.sin();
    let vy = ta.cos();
    let vz = ux * ta.sin();
    let nx = uy * vz - uz * vy;
    let ny = uz * vx - ux * vz;
    let nz = ux * vy - uy * vx;

    let wob_mul = if face_on { 0.368 } else { 1.0 };
    let wob_amp = 0.23 * wob_mul;
    let base_r = if face_on { r / (1.0 + 0.85 * wob_amp) } else { r };
    let lanes = if face_on { 5 } else { 5 };
    let segs = if size < 40.0 { 32 } else { 64 };

    for w in 0..lanes {
        let lane_off = (w as f32 - (lanes - 1) as f32 / 2.0) * 0.075;
        let edge = (w as f32 - (lanes - 1) as f32 / 2.0).abs() / ((lanes - 1) as f32 / 2.0).max(1.0);
        for k in 0..segs {
            let a = (k as f32 / segs as f32) * 2.0 * std::f32::consts::PI;
            let wob = (0.16 * (a * 3.0 - t * 1.7 + w as f32 * 0.22).sin() + 0.07 * (a * 5.0 + t * 1.1).sin()) * wob_mul;
            let radial = if face_on { 1.0 + wob } else { 1.0 };
            let off = if face_on { lane_off } else { lane_off + wob };

            let x = ux * a.cos() + vx * a.sin() + nx * off;
            let y = uy * a.cos() + vy * a.sin() + ny * off;
            let z = uz * a.cos() + vz * a.sin() + nz * off;
            let l = (x * x + y * y + z * z).sqrt().max(1e-6);
            let rr = base_r * radial;
            let (px, py, zr) = project_3d(t * 0.1 * spin, cam_tilt, cx, cy, 1.0, (x / l) * rr, (y / l) * rr, (z / l) * rr);
            let depth = (zr / r + 1.0) / 2.0;
            dots.push(Dot {
                x: px,
                y: py,
                z: zr,
                r: (1.1 + 1.7 * depth) * (1.0 - 0.25 * edge) * rs,
                white: 0.52 - 0.44 * depth + 0.18 * edge,
                a: 0.4 + 0.6 * depth,
            });
        }
    }
    finalize_frame(dots, Vec::new(), 0.3)
}

/// 9. Shaping (morph.ts - frameMorph)
fn smooth_e(x: f32) -> f32 {
    x * x * (3.0 - 2.0 * x)
}

fn path_circle(f: f32) -> [f32; 2] {
    let a = -std::f32::consts::FRAC_PI_2 + f * 2.0 * std::f32::consts::PI;
    [a.cos() * 0.24, a.sin() * 0.24]
}

fn path_poly(verts: &[[f32; 2]], f: f32) -> [f32; 2] {
    let v = verts.len();
    let mut lengths = Vec::with_capacity(v);
    let mut total = 0.0;
    for i in 0..v {
        let a = verts[i];
        let b = verts[(i + 1) % v];
        let l = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        lengths.push(l);
        total += l;
    }
    let mut target = f * total;
    let mut i = 0;
    while target > lengths[i] && i < v - 1 {
        target -= lengths[i];
        i += 1;
    }
    let a = verts[i];
    let b = verts[(i + 1) % v];
    let ff = if lengths[i] > 1e-6 { (target / lengths[i]).min(1.0) } else { 0.0 };
    [a[0] + (b[0] - a[0]) * ff, a[1] + (b[1] - a[1]) * ff]
}

const TRIANGLE_VERTS: [[f32; 2]; 3] = [
    [0.0, -0.26],
    [0.24, 0.16],
    [-0.24, 0.16],
];

const SQUARE_VERTS: [[f32; 2]; 5] = [
    [0.0, -0.2],
    [0.2, -0.2],
    [0.2, 0.2],
    [-0.2, 0.2],
    [-0.2, -0.2],
];

pub fn frame_morph(size: f32, t: f32, size_factor: f32) -> OrbFrame {
    const HOLD: f32 = 1.4;
    const MORPH: f32 = 0.9;
    const SEG: f32 = HOLD + MORPH;
    const K: f32 = 3.0;

    let tc = t % (SEG * K);
    let k = (tc / SEG).floor() as usize % 3;
    let local = tc - k as f32 * SEG;
    let m = if local > HOLD { smooth_e((local - HOLD) / MORPH) } else { 0.0 };
    let sprd = 1.45;

    let sample_path = |k_idx: usize, f: f32| -> [f32; 2] {
        match k_idx {
            0 => path_circle(f),
            1 => path_poly(&TRIANGLE_VERTS, f),
            _ => path_poly(&SQUARE_VERTS, f),
        }
    };

    let m_res = 120;
    let mut pts = Vec::with_capacity(m_res);
    for i in 0..m_res {
        let f = i as f32 / m_res as f32;
        let a = sample_path(k, f);
        let b = sample_path((k + 1) % 3, f);
        pts.push([
            (a[0] + (b[0] - a[0]) * m) * sprd,
            (a[1] + (b[1] - a[1]) * m) * sprd,
        ]);
    }

    let mut lengths = Vec::with_capacity(m_res);
    let mut total = 0.0;
    for i in 0..m_res {
        let a = pts[i];
        let b = pts[(i + 1) % m_res];
        let l = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        lengths.push(l);
        total += l;
    }

    let n = if size < 40.0 { 18 } else { 38 };
    let re = 0.021 * 1.35 * sprd * size_factor;
    let pulse = 1.0 + 0.02 * (local * 3.1).sin();
    let c2 = size / 2.0;

    let mut dots = Vec::with_capacity(n);
    let mut seg_idx = 0;
    let mut acc = 0.0;

    for k2 in 0..n {
        let target = (k2 as f32 / n as f32) * total;
        while acc + lengths[seg_idx] < target && seg_idx < m_res - 1 {
            acc += lengths[seg_idx];
            seg_idx += 1;
        }
        let a = pts[seg_idx];
        let b = pts[(seg_idx + 1) % m_res];
        let f = if lengths[seg_idx] > 1e-6 { ((target - acc) / lengths[seg_idx]).min(1.0) } else { 0.0 };
        let x = (a[0] + (b[0] - a[0]) * f) * pulse;
        let y = (a[1] + (b[1] - a[1]) * f) * pulse;
        dots.push(Dot {
            x: c2 + x * size,
            y: c2 + y * size,
            z: 0.0,
            r: (re * size).max(0.7),
            white: 0.1,
            a: 1.0,
        });
    }

    finalize_frame(dots, Vec::new(), 0.3)
}

/// Dispatches frame generation for any of the 9 states
pub fn generate_state_frame(state: OrbStateKind, size: f32, t: f32, size_factor: f32) -> OrbFrame {
    match state {
        OrbStateKind::Working => frame_orbits(size, t * 1.885, size_factor),
        OrbStateKind::Searching => frame_globe(size, t * 2.015, size_factor),
        OrbStateKind::Solving => frame_rubik(size, t * 1.820, size_factor),
        OrbStateKind::Listening => frame_wave(size, t * 4.388, size_factor),
        OrbStateKind::Connecting => frame_web(size, t * 3.315, size_factor),
        OrbStateKind::Weaving => frame_braid(size, t * 1.625, size_factor),
        OrbStateKind::Composing => frame_ribbon_or_ring(size, t * 2.340, false, size_factor),
        OrbStateKind::Breathing => frame_ribbon_or_ring(size, t * 3.240, true, size_factor),
        OrbStateKind::Shaping => frame_morph(size, t * 2.405, size_factor),
    }
}

pub fn draw_inline_thinking_orb(
    ui: &mut Ui,
    kind: OrbStateKind,
    x: f32,
    y: f32,
    size: f32,
    time_sec: f32,
    colors: &ThemeColors,
) {
    let size_factor = if size <= 36.0 { 1.8 } else { 1.15 };
    let frame = generate_state_frame(kind, size, time_sec, size_factor);
    let clip = [x, y, size, size];
    let is_light = colors.bg_base.r > 0.5;
    let theme_color = if is_light {
        [colors.accent.r * 0.68, colors.accent.g * 0.68, colors.accent.b * 0.68]
    } else {
        [
            colors.accent.r * 0.80 + 0.20,
            colors.accent.g * 0.80 + 0.20,
            colors.accent.b * 0.80 + 0.20,
        ]
    };
    ui.draws.reserve(frame.lines.len() * 15 + frame.dots.len());

    for line in &frame.lines {
        let dx = line.x2 - line.x1;
        let dy = line.y2 - line.y1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let steps = ((len / 3.8).ceil() as usize).max(1);
        let line_w = (line.w * 1.15).max(0.85);
        let shade = 0.55 + (1.0 - line.white.clamp(0.0, 1.0)) * 0.45;
        let color = [
            theme_color[0] * shade,
            theme_color[1] * shade,
            theme_color[2] * shade,
            (line.a * 0.85).clamp(0.0, 1.0),
        ];

        for step in 0..=steps {
            let fraction = step as f32 / steps as f32;
            let px = x + line.x1 + dx * fraction - line_w * 0.5;
            let py = y + line.y1 + dy * fraction - line_w * 0.5;
            ui.draws.push(DrawCommand::Rect(RectDraw {
                instance: RectInstance {
                    pos: [px, py],
                    size: [line_w, line_w],
                    color,
                    radius: [line_w * 0.5; 4],
                    clip_rect: clip,
                    ..Default::default()
                },
            }));
        }
    }

    for dot in &frame.dots {
        let dot_size = (dot.r * 2.0).max(1.2);
        let brightness = (1.0 - dot.white).clamp(0.05, 1.0);
        let shade = 0.45 + brightness * 0.55;
        let color = [
            theme_color[0] * shade,
            theme_color[1] * shade,
            theme_color[2] * shade,
            dot.a.clamp(0.1, 1.0),
        ];
        let px = x + dot.x - dot.r;
        let py = y + dot.y - dot.r;
        ui.draws.push(DrawCommand::Rect(RectDraw {
            instance: RectInstance {
                pos: [px, py],
                size: [dot_size, dot_size],
                color,
                radius: [dot.r; 4],
                clip_rect: clip,
                ..Default::default()
            },
        }));
    }
}

// ============================================================================
// 4. FLOATING WINDOW UI RENDERER
// ============================================================================

pub fn draw_thinking_orbs_demo_window(ui: &mut Ui, state: &mut ZenthreeState) {
    if !state.thinking_orbs_demo_open {
        return;
    }

    let colors = use_context::<ThemeColors>()
        .map(|c| *c)
        .unwrap_or_else(|| state.colors());

    let win_w = 460.0;
    let win_h = 460.0;
    let win_radius = 12.0;

    // Default position centered/top-right
    if state.thinking_orbs_demo_pos.0 <= 1.0 && state.thinking_orbs_demo_pos.1 <= 1.0 {
        state.thinking_orbs_demo_pos = (
            (ui.width - win_w - 40.0).max(20.0),
            70.0,
        );
    }

    let mut pos_x = state.thinking_orbs_demo_pos.0.round();
    let mut pos_y = state.thinking_orbs_demo_pos.1.round();
    const HEADER_H: f32 = 38.0;

    // Header dragging
    let in_header = ui.mouse_in_rect(pos_x, pos_y, win_w - 38.0, HEADER_H);
    if in_header && !state.thinking_orbs_demo_dragging {
        ui.cursor_icon = CursorIcon::Grab;
        if ui.mouse_down {
            state.thinking_orbs_demo_dragging = true;
            state.thinking_orbs_demo_drag_offset = (ui.mouse_x - pos_x, ui.mouse_y - pos_y);
        }
    }

    if state.thinking_orbs_demo_dragging {
        if ui.mouse_down {
            ui.cursor_icon = CursorIcon::Grabbing;
            ui.consume_pointer();
            pos_x = (ui.mouse_x - state.thinking_orbs_demo_drag_offset.0)
                .clamp(10.0, (ui.width - win_w - 10.0).max(10.0))
                .round();
            pos_y = (ui.mouse_y - state.thinking_orbs_demo_drag_offset.1)
                .clamp(10.0, (ui.height - win_h - 10.0).max(10.0))
                .round();
            state.thinking_orbs_demo_pos = (pos_x, pos_y);
            ui.needs_redraw = true;
        } else {
            state.thinking_orbs_demo_dragging = false;
        }
    }

    // Modal / overlay registration
    let card_id = Id::from_u64(999999980);
    let prev_win_id = ui.current_window_id;
    ui.current_window_id = Some(card_id);
    ui.record_layout(card_id, zenthra_core::Rect::new(pos_x, pos_y, win_w, win_h));
    let overlay_key = Id::from_u64((card_id.raw() << 8) | 99);
    ui.interaction_state.insert(overlay_key, 1.0);
    ui.active_overlays.push(card_id);
    ui.active_overlay_stack.push(card_id);

    let card_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha(0.94)
    } else {
        colors.bg_sidebar
    };

    let mut do_close = false;
    let current_kind = OrbStateKind::ALL[state.thinking_orbs_demo_mode % 9];

    let render_mode = if state.thinking_orbs_demo_paused {
        zenthra::RenderMode::Static
    } else {
        zenthra::RenderMode::Continuous
    };

    let mut card = ui.container()
        .id(card_id)
        .absolute(pos_x, pos_y)
        .overlay()
        .render_mode(render_mode)
        .width(win_w)
        .height(win_h)
        .bg(card_bg)
        .border(colors.border.with_alpha(0.85), 1.0)
        .radius_all(win_radius)
        .shadow(Color::rgba(0.0, 0.0, 0.0, 0.50), 0.0, 12.0, 32.0);

    if state.glassmorphism {
        card = card.backdrop_blur(state.blur_strength.max(8.0));
    }

    card.column().show(|ui| {
        // ── Window Title Bar ──────────────────────────────────────────────
        ui.container()
            .full_width()
            .height(HEADER_H)
            .radius_top(win_radius)
            .bg(colors.highlight.with_alpha(0.04))
            .padding_left(14.0)
            .padding_right(10.0)
            .row()
            .valign(Align::Center)
            .halign(Align::SpaceBetween)
            .show(|ui| {
                ui.container()
                    .row()
                    .valign(Align::Center)
                    .gap(8.0)
                    .show(|ui| {
                        ui.container()
                            .width(8.0)
                            .height(8.0)
                            .radius_all(4.0)
                            .bg(colors.accent)
                            .show(|_| {});

                        ui.text("Thinking Orbs")
                            .size(13.0)
                            .weight(FontWeight::Bold)
                            .color(colors.text_primary)
                            .show();

                        ui.container()
                            .padding_x(6.0)
                            .padding_y(1.5)
                            .radius_all(4.0)
                            .bg(colors.accent.with_alpha(0.18))
                            .show(|ui| {
                                ui.text("9 Modes · Pure Math")
                                    .size(9.5)
                                    .weight(FontWeight::Medium)
                                    .color(colors.accent)
                                    .show();
                            });
                    });

                let close_btn = ui.button("✕")
                    .id(Id::from_u64(999999982))
                    .width(22.0)
                    .height(22.0)
                    .padding_all(0.0)
                    .radius_all(4.0)
                    .bg(Color::TRANSPARENT)
                    .hover_bg(colors.highlight)
                    .text_color(colors.text_muted)
                    .size(11.5)
                    .show();

                if close_btn.hovered {
                    ui.cursor_icon = CursorIcon::Pointer;
                }
                if close_btn.clicked {
                    do_close = true;
                }
            });

        // Thin divider
        ui.container()
            .full_width()
            .height(1.0)
            .bg(colors.border.with_alpha(0.40))
            .show(|_| {});

        // ── 9 Mode Selector Pill Tabs (two rows: 5 + 4) ───────────────────
        ui.container()
            .full_width()
            .padding_left(10.0)
            .padding_right(10.0)
            .padding_top(7.0)
            .padding_bottom(5.0)
            .column()
            .gap(4.0)
            .show(|ui| {
                // Row 1: first 5 modes
                ui.container()
                    .full_width()
                    .row()
                    .halign(Align::Center)
                    .valign(Align::Center)
                    .gap(4.0)
                    .show(|ui| {
                        for (idx, kind) in OrbStateKind::ALL.iter().enumerate().take(5) {
                            let is_active = state.thinking_orbs_demo_mode == idx;
                            let pill_bg = if is_active {
                                colors.accent.with_alpha(0.25)
                            } else {
                                colors.highlight.with_alpha(0.06)
                            };
                            let pill_text_col = if is_active { colors.accent } else { colors.text_muted };
                            let pill_border = if is_active {
                                colors.accent.with_alpha(0.75)
                            } else {
                                colors.border.with_alpha(0.25)
                            };
                            let btn = ui.button(kind.name())
                                .id(Id::from_u64(999999920 + idx as u64))
                                .height(22.0)
                                .padding_x(8.0)
                                .padding_y(2.0)
                                .radius_all(5.0)
                                .bg(pill_bg)
                                .hover_bg(if is_active { pill_bg } else { colors.highlight.with_alpha(0.14) })
                                .text_color(pill_text_col)
                                .border(pill_border, 1.0)
                                .size(9.5)
                                .show();
                            if btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                            if btn.clicked {
                                state.thinking_orbs_demo_mode = idx;
                                ui.needs_redraw = true;
                            }
                        }
                    });

                // Row 2: last 4 modes
                ui.container()
                    .full_width()
                    .row()
                    .halign(Align::Center)
                    .valign(Align::Center)
                    .gap(4.0)
                    .show(|ui| {
                        for (idx, kind) in OrbStateKind::ALL.iter().enumerate().skip(5) {
                            let is_active = state.thinking_orbs_demo_mode == idx;
                            let pill_bg = if is_active {
                                colors.accent.with_alpha(0.25)
                            } else {
                                colors.highlight.with_alpha(0.06)
                            };
                            let pill_text_col = if is_active { colors.accent } else { colors.text_muted };
                            let pill_border = if is_active {
                                colors.accent.with_alpha(0.75)
                            } else {
                                colors.border.with_alpha(0.25)
                            };
                            let btn = ui.button(kind.name())
                                .id(Id::from_u64(999999920 + idx as u64))
                                .height(22.0)
                                .padding_x(8.0)
                                .padding_y(2.0)
                                .radius_all(5.0)
                                .bg(pill_bg)
                                .hover_bg(if is_active { pill_bg } else { colors.highlight.with_alpha(0.14) })
                                .text_color(pill_text_col)
                                .border(pill_border, 1.0)
                                .size(9.5)
                                .show();
                            if btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                            if btn.clicked {
                                state.thinking_orbs_demo_mode = idx;
                                ui.needs_redraw = true;
                            }
                        }
                    });
            });


        // ── Orb Display Stage (Dedicated High-Contrast Dark Box) ──────────
        const STAGE_W: f32 = 436.0;
        const STAGE_H: f32 = 210.0;
        let stage_x = pos_x + 12.0;
        let stage_y = pos_y + HEADER_H + 70.0;

        // Stage Container: Dark contrast backdrop so dots are crystal clear and vibrant
        ui.container()
            .full_width()
            .halign(Align::Center)
            .show(|ui| {
                ui.container()
                    .id(Id::from_u64(999999983))
                    .render_mode(render_mode)
                    .width(STAGE_W)
                    .height(STAGE_H)
                    .radius_all(10.0)
                    .bg(Color::rgb(8.0 / 255.0, 9.0 / 255.0, 14.0 / 255.0)) // Deep velvety dark contrast
                    .border(colors.border.with_alpha(0.50), 1.0)
                    .shadow(Color::rgba(0.0, 0.0, 0.0, 0.60), 0.0, 4.0, 16.0)
                    .show(|_| {});
            });

        // ── Direct Mathematical Engine Frame Computation ──────────────────
        let time_sec = if state.thinking_orbs_demo_paused {
            state.thinking_orbs_demo_pause_time
        } else {
            state.orb_animation_start.elapsed().as_secs_f32() * state.thinking_orbs_demo_speed
        };

        let orb_render_size = match state.thinking_orbs_demo_size_preset {
            1 => 32.0,  // Inline size
            2 => 140.0, // Large showcase
            _ => 90.0,  // Standard avatar size
        };
        let size_multiplier = match state.thinking_orbs_demo_size_preset {
            1 => 1.8,
            2 => 1.0,
            _ => 1.15,
        };

        let frame = generate_state_frame(current_kind, orb_render_size, time_sec, size_multiplier);

        // Center orb within the stage
        let orb_ox = stage_x + (STAGE_W - orb_render_size) / 2.0;
        let orb_oy = stage_y + (STAGE_H - orb_render_size) / 2.0;
        let stage_clip = [stage_x, stage_y, STAGE_W, STAGE_H];

        // ── Fast Direct Instanced GPU Drawing of Lines and Dots ───────────

        // Pre-reserve capacity to avoid any dynamic allocations during drawing
        ui.draws.reserve(frame.lines.len() * 15 + frame.dots.len());

        // 1. Draw edge lines (connecting state)
        for line in &frame.lines {
            let dx = line.x2 - line.x1;
            let dy = line.y2 - line.y1;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            let steps = ((len / 3.8).ceil() as usize).max(1);
            let line_w = (line.w * 1.15).max(0.85);
            let alpha = (line.a * 0.85).clamp(0.0, 1.0);

            let col = [
                0.35 + 0.65 * (1.0 - line.white),
                0.50 + 0.50 * (1.0 - line.white),
                0.90,
                alpha,
            ];

            for s in 0..=steps {
                let frac = s as f32 / steps as f32;
                let px = orb_ox + line.x1 + dx * frac - line_w * 0.5;
                let py = orb_oy + line.y1 + dy * frac - line_w * 0.5;

                ui.draws.push(DrawCommand::Rect(RectDraw {
                    instance: RectInstance {
                        pos: [px, py],
                        size: [line_w, line_w],
                        color: col,
                        radius: [line_w * 0.5; 4],
                        clip_rect: stage_clip,
                        border_width: 0.0,
                        border_color: [0.0; 4],
                        shadow_color: [0.0; 4],
                        shadow_offset: [0.0; 2],
                        shadow_blur: 0.0,
                        grayscale: 0.0,
                        brightness: 1.0,
                        opacity: 1.0,
                        border_alignment: 0.0,
                    },
                }));
            }
        }

        // 2. Draw dots (z-sorted from back to front)
        for dot in &frame.dots {
            let dot_d = (dot.r * 2.0).max(1.2);
            let px = orb_ox + dot.x - dot.r;
            let py = orb_oy + dot.y - dot.r;

            // Inverted grayscale substrate logic from thinking-orbs:
            // white in [0..1], 0 is nearest / brightest ink on dark substrate
            let w = dot.white.clamp(0.0, 1.0);
            let bright = (1.0 - w).clamp(0.05, 1.0);

            // Give subtle aesthetic tint based on current state
            let dot_col = match current_kind {
                OrbStateKind::Connecting => [bright * 0.85 + 0.15, bright * 0.90 + 0.10, 1.0, dot.a.clamp(0.1, 1.0)],
                OrbStateKind::Working => [1.0, bright * 0.85 + 0.15, bright * 0.50, dot.a.clamp(0.1, 1.0)],
                OrbStateKind::Searching => [bright * 0.40 + 0.10, bright * 0.95 + 0.05, 1.0, dot.a.clamp(0.1, 1.0)],
                OrbStateKind::Solving => [bright * 0.70 + 0.30, bright * 0.80 + 0.20, 1.0, dot.a.clamp(0.1, 1.0)],
                OrbStateKind::Listening => [bright * 0.50 + 0.50, 1.0, bright * 0.80 + 0.20, dot.a.clamp(0.1, 1.0)],
                OrbStateKind::Weaving => [1.0, bright * 0.60 + 0.20, bright * 0.85 + 0.15, dot.a.clamp(0.1, 1.0)],
                OrbStateKind::Composing => [bright * 0.85 + 0.15, bright * 0.70 + 0.10, 1.0, dot.a.clamp(0.1, 1.0)],
                OrbStateKind::Breathing => [bright * 0.75 + 0.25, bright * 0.90 + 0.10, 1.0, dot.a.clamp(0.1, 1.0)],
                OrbStateKind::Shaping => [1.0, 1.0, 1.0, dot.a.clamp(0.1, 1.0)],
            };

            ui.draws.push(DrawCommand::Rect(RectDraw {
                instance: RectInstance {
                    pos: [px, py],
                    size: [dot_d, dot_d],
                    color: dot_col,
                    radius: [dot.r; 4], // Exact circle SDF
                    clip_rect: stage_clip,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                    shadow_color: [0.0; 4],
                    shadow_offset: [0.0; 2],
                    shadow_blur: 0.0,
                    grayscale: 0.0,
                    brightness: 1.0,
                    opacity: 1.0,
                    border_alignment: 0.0,
                },
            }));
        }

        // ── Controls & State Metadata Readout ─────────────────────────────
        ui.container()
            .full_width()
            .padding_left(14.0)
            .padding_right(14.0)
            .padding_top(10.0)
            .column()
            .gap(8.0)
            .show(|ui| {
                // Row: Info Subtitle + Size Presets + Speed Multipliers
                ui.container()
                    .full_width()
                    .row()
                    .valign(Align::Center)
                    .halign(Align::SpaceBetween)
                    .show(|ui| {
                        // Left: Current Mode Name + truncated description
                        ui.container()
                            .row()
                            .valign(Align::Center)
                            .gap(5.0)
                            .show(|ui| {
                                ui.text(current_kind.name())
                                    .size(12.0)
                                    .weight(FontWeight::Bold)
                                    .color(colors.text_primary)
                                    .show();

                                ui.text("—")
                                    .size(10.0)
                                    .color(colors.text_muted)
                                    .show();

                                // Short version of subtitle to avoid overflow
                                let short_subtitle = current_kind.subtitle()
                                    .split('·')
                                    .next()
                                    .unwrap_or("")
                                    .trim();
                                ui.text(short_subtitle)
                                    .size(10.0)
                                    .color(colors.text_muted)
                                    .show();
                            });


                        // Right: Interactive Controls (Size / Speed / Play-Pause)
                        ui.container()
                            .row()
                            .valign(Align::Center)
                            .gap(6.0)
                            .show(|ui| {
                                // Play / Pause Button
                                let play_pause_label = if state.thinking_orbs_demo_paused { "▶ Play" } else { "⏸ Pause" };
                                let pp_btn = ui.button(play_pause_label)
                                    .id(Id::from_u64(999999960))
                                    .height(20.0)
                                    .padding_x(6.0)
                                    .padding_y(1.5)
                                    .radius_all(4.0)
                                    .bg(colors.highlight.with_alpha(0.08))
                                    .hover_bg(colors.highlight.with_alpha(0.20))
                                    .text_color(colors.text_primary)
                                    .size(9.5)
                                    .show();

                                if pp_btn.hovered {
                                    ui.cursor_icon = CursorIcon::Pointer;
                                }
                                if pp_btn.clicked {
                                    state.thinking_orbs_demo_paused = !state.thinking_orbs_demo_paused;
                                    if state.thinking_orbs_demo_paused {
                                        state.thinking_orbs_demo_pause_time = state.orb_animation_start.elapsed().as_secs_f32() * state.thinking_orbs_demo_speed;
                                    }
                                    ui.needs_redraw = true;
                                }

                                // Size Preset Buttons: 64px / 20px / 120px
                                for (s_idx, s_label) in [("64px", 0), ("20px", 1), ("120px", 2)] {
                                    let is_cur_size = state.thinking_orbs_demo_size_preset == s_label;
                                    let s_btn = ui.button(s_idx)
                                        .id(Id::from_u64(999999965 + s_label as u64))
                                        .height(20.0)
                                        .padding_x(5.0)
                                        .padding_y(1.5)
                                        .radius_all(4.0)
                                        .bg(if is_cur_size { colors.accent.with_alpha(0.25) } else { colors.highlight.with_alpha(0.06) })
                                        .hover_bg(colors.highlight.with_alpha(0.18))
                                        .text_color(if is_cur_size { colors.accent } else { colors.text_muted })
                                        .border(if is_cur_size { colors.accent.with_alpha(0.60) } else { colors.border.with_alpha(0.20) }, 1.0)
                                        .size(9.0)
                                        .show();

                                    if s_btn.hovered {
                                        ui.cursor_icon = CursorIcon::Pointer;
                                    }
                                    if s_btn.clicked {
                                        state.thinking_orbs_demo_size_preset = s_label;
                                        ui.needs_redraw = true;
                                    }
                                }

                                // Speed Buttons: 1x / 1.5x / 2x
                                for (sp_label, sp_val) in [("1x", 1.0_f32), ("1.5x", 1.5_f32), ("2x", 2.0_f32)] {
                                    let is_cur_sp = (state.thinking_orbs_demo_speed - sp_val).abs() < 0.05;
                                    let sp_btn = ui.button(sp_label)
                                        .id(Id::from_u64(999999970 + (sp_val * 10.0) as u64))
                                        .height(20.0)
                                        .padding_x(5.0)
                                        .padding_y(1.5)
                                        .radius_all(4.0)
                                        .bg(if is_cur_sp { colors.accent.with_alpha(0.25) } else { colors.highlight.with_alpha(0.06) })
                                        .hover_bg(colors.highlight.with_alpha(0.18))
                                        .text_color(if is_cur_sp { colors.accent } else { colors.text_muted })
                                        .border(if is_cur_sp { colors.accent.with_alpha(0.60) } else { colors.border.with_alpha(0.20) }, 1.0)
                                        .size(9.0)
                                        .show();

                                    if sp_btn.hovered {
                                        ui.cursor_icon = CursorIcon::Pointer;
                                    }
                                    if sp_btn.clicked {
                                        state.thinking_orbs_demo_speed = sp_val;
                                        ui.needs_redraw = true;
                                    }
                                }
                            });
                    });
            });
    });

    if ui.clicked && ui.mouse_in_rect(pos_x, pos_y, win_w, win_h) {
        ui.consume_click();
    }

    if do_close {
        state.thinking_orbs_demo_open = false;
        state.thinking_orbs_demo_dragging = false;
        ui.needs_redraw = true;
    }

    // Cleanup overlays
    ui.active_overlay_stack.retain(|&id| id != card_id);
    ui.current_window_id = prev_win_id;

    // Hardware V-Sync synchronized animation loop (60 / 120 / 144 Hz)
    if !state.thinking_orbs_demo_paused {
        ui.request_redraw();
    }
}
