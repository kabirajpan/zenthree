//! Port of Jakubantalik/thinking-orbs to native Rust for Zenthree IDE.
//! Generates 3D-projected dotted thought-orbs for AI states (Working, Searching, Connecting, Breathing).

#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrbMode {
    Default,
    Thinking,
    Searching,
    Analyzing,
}

impl OrbMode {
    pub const ALL: [OrbMode; 4] = [
        OrbMode::Default,
        OrbMode::Thinking,
        OrbMode::Searching,
        OrbMode::Analyzing,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            OrbMode::Default => "Default",
            OrbMode::Thinking => "Thinking",
            OrbMode::Searching => "Searching",
            OrbMode::Analyzing => "Analyzing",
        }
    }

    pub fn mode_title(&self) -> &'static str {
        match self {
            OrbMode::Default => "Default",
            OrbMode::Thinking => "Thinking",
            OrbMode::Searching => "Searching",
            OrbMode::Analyzing => "Analyzing",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OrbDot {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
    pub white: f32, // 0 = bright on dark, 1 = dark on dark
    pub a: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct OrbLine {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub white: f32,
    pub a: f32,
    pub w: f32,
}

#[derive(Debug, Clone)]
pub struct OrbFrame {
    pub dots: Vec<OrbDot>,
    #[allow(dead_code)]
    pub lines: Vec<OrbLine>,
}

/// Deterministic hash in [0, 1)
pub fn hash_d(a: f32, b: f32) -> f32 {
    let h = (a * 12.9898 + b * 78.233).sin() * 43758.5453;
    h - h.floor()
}

/// Value noise on a 2D lattice — smooth, deterministic, cheap
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

/// Stable directions on a unit sphere (Fibonacci lattice)
pub fn fib_dir(i: usize, n: usize) -> [f32; 3] {
    let golden = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
    let y = 1.0 - (2.0 * (i as f32 + 0.5)) / n.max(1) as f32;
    let rad = (1.0 - y * y).max(0.0).sqrt();
    let a = i as f32 * golden;
    [rad * a.cos(), y, rad * a.sin()]
}

/// Shortest signed angular distance wrapped to (-pi, pi]
pub fn angle_delta(a: f32, b: f32) -> f32 {
    (a - b).sin().atan2((a - b).cos())
}

/// Dot radius scaling based on frame size
pub fn radius_scale(size: f32, pow: f32) -> f32 {
    (size / 300.0).powf(pow)
}

/// Shared spin + tilt + orthographic 3D projection
pub fn project_3d(x: f32, y: f32, z: f32, yaw: f32, tilt: f32, cx: f32, cy: f32, scale: f32) -> (f32, f32, f32) {
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

/// 1. Orbits (working): particles on tilted 3D orbits around a sphere
pub fn frame_orbits(size: f32, t: f32) -> OrbFrame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let r = (size / 2.0) * 0.82;
    let rs = radius_scale(size, 0.6);
    let mut dots = Vec::with_capacity(160);

    let orbit_n = 6;
    let ghost_n = 20;
    let particles = 2;

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
        let speed = (0.28 + 0.62 * h3) * if h3 > 0.5 { 1.0 } else { -1.0 };

        // Ghost orbital ring
        for k in 0..ghost_n {
            let a = (k as f32 / ghost_n as f32) * 2.0 * std::f32::consts::PI;
            let (px, py, z) = project_3d(
                (ux * a.cos() + vx * a.sin()) * ro,
                (uy * a.cos() + vy * a.sin()) * ro,
                (uz * a.cos() + vz * a.sin()) * ro,
                t * 0.18,
                0.32,
                cx, cy, 1.0,
            );
            let depth = ((z / ro + 1.0) / 2.0).clamp(0.0, 1.0);
            dots.push(OrbDot {
                x: px,
                y: py,
                z,
                r: (0.95 * rs).max(0.75),
                white: 0.70,
                a: 0.28 * (0.4 + 0.6 * depth),
            });
        }

        // Active working particles
        for m in 0..particles {
            let a = t * speed * 2.2 + (m as f32 / particles as f32) * 2.0 * std::f32::consts::PI + h2 * 6.0;
            let (px, py, z) = project_3d(
                (ux * a.cos() + vx * a.sin()) * ro,
                (uy * a.cos() + vy * a.sin()) * ro,
                (uz * a.cos() + vz * a.sin()) * ro,
                t * 0.18,
                0.32,
                cx, cy, 1.0,
            );
            let depth = ((z / ro + 1.0) / 2.0).clamp(0.0, 1.0);
            dots.push(OrbDot {
                x: px,
                y: py,
                z,
                r: ((1.4 + 2.0 * depth) * rs).max(1.8),
                white: 0.08, // bright
                a: 0.88 + 0.12 * depth,
            });
        }
    }

    dots.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal));
    OrbFrame { dots, lines: Vec::new() }
}

/// 2. Globe (searching): a radar scan sweeps across a dotted globe
pub fn frame_globe(size: f32, t: f32) -> OrbFrame {
    let spin = 0.55;
    let cx = size / 2.0;
    let cy = size / 2.0;
    let radius = (size / 2.0) * 0.82;
    let tilt = 0.38 + 0.08 * (t * 0.35).sin();
    let scan = t * (spin + (1.7 - spin) * 2.2);
    let rs = radius_scale(size, 0.6);
    let mut dots = Vec::with_capacity(160);

    let lat_rings = 11;
    let lon_density = 26;

    for li in 0..=lat_rings {
        let lat = -std::f32::consts::FRAC_PI_2 + (li as f32 / lat_rings as f32) * std::f32::consts::PI;
        let cos_lat = lat.cos();
        let sin_lat = lat.sin();
        let lon_count = (cos_lat.abs() * lon_density as f32).round().max(1.0) as usize;

        for lj in 0..lon_count {
            let lon = (lj as f32 / lon_count as f32) * 2.0 * std::f32::consts::PI;
            let (px, py, z) = project_3d(
                cos_lat * lon.cos() * radius,
                sin_lat * radius,
                cos_lat * lon.sin() * radius,
                t * spin,
                tilt,
                cx, cy, 1.0,
            );
            let depth = ((z / radius + 1.0) / 2.0).clamp(0.0, 1.0);
            let d = angle_delta(lon + t * spin, scan);
            let boost = (-(d * d) / 0.20).exp() * (z / radius).max(0.0);

            dots.push(OrbDot {
                x: px,
                y: py,
                z,
                r: ((0.75 + 1.2 * depth + 1.8 * boost) * rs).max(0.9),
                white: 0.65 - 0.55 * depth,
                a: (0.35 + 0.65 * boost.min(1.0)).clamp(0.12, 1.0),
            });
        }
    }

    dots.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal));
    OrbFrame { dots, lines: Vec::new() }
}

/// 3. Web (connecting): constellation network linking wandering nodes
pub fn frame_web(size: f32, t: f32) -> OrbFrame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let radius = (size / 2.0) * 0.78;
    let rs = radius_scale(size, 0.6);
    let node_n = 24;
    let thr = 0.82;

    let mut nodes = Vec::with_capacity(node_n);
    for i in 0..node_n {
        let d = fib_dir(i, node_n);
        let x = d[0] + 0.25 * (vnoise(i as f32 * 0.31 + 9.0, t * 0.28) - 0.5) * 2.0;
        let y = d[1] + 0.25 * (vnoise(i as f32 * 0.53 + 27.0, t * 0.24) - 0.5) * 2.0;
        let z = d[2] + 0.25 * (vnoise(i as f32 * 0.77 + 55.0, t * 0.26) - 0.5) * 2.0;
        let len = (x * x + y * y + z * z).sqrt().max(1e-6);
        nodes.push([x / len, y / len, z / len]);
    }

    let mut lines = Vec::new();
    let mut dots = Vec::with_capacity(node_n + 80);

    // Dynamic edges
    for i in 0..node_n {
        for j in (i + 1)..node_n {
            let dx = nodes[i][0] - nodes[j][0];
            let dy = nodes[i][1] - nodes[j][1];
            let dz = nodes[i][2] - nodes[j][2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            if dist < thr {
                let (x1, y1, z1) = project_3d(nodes[i][0] * radius, nodes[i][1] * radius, nodes[i][2] * radius, t * 0.16, 0.32, cx, cy, 1.0);
                let (x2, y2, z2) = project_3d(nodes[j][0] * radius, nodes[j][1] * radius, nodes[j][2] * radius, t * 0.16, 0.32, cx, cy, 1.0);
                let depth = (((z1 + z2) / (2.0 * radius) + 1.0) / 2.0).clamp(0.0, 1.0);
                
                // Add points along the line for dotted constellation effect
                let steps = 4;
                for step in 1..steps {
                    let frac = step as f32 / steps as f32;
                    let px = x1 + (x2 - x1) * frac;
                    let py = y1 + (y2 - y1) * frac;
                    let pz = z1 + (z2 - z1) * frac;
                    dots.push(OrbDot {
                        x: px,
                        y: py,
                        z: pz,
                        r: (0.75 * rs).max(0.65),
                        white: 0.60,
                        a: (1.0 - dist / thr) * (0.25 + 0.45 * depth),
                    });
                }

                lines.push(OrbLine {
                    x1, y1, x2, y2,
                    white: 0.45,
                    a: (1.0 - dist / thr) * (0.3 + 0.50 * depth),
                    w: (0.75 * rs).max(0.6),
                });
            }
        }
    }

    // Constellation nodes
    for i in 0..node_n {
        let (px, py, z) = project_3d(nodes[i][0] * radius, nodes[i][1] * radius, nodes[i][2] * radius, t * 0.16, 0.32, cx, cy, 1.0);
        let depth = ((z / radius + 1.0) / 2.0).clamp(0.0, 1.0);
        dots.push(OrbDot {
            x: px,
            y: py,
            z,
            r: ((1.3 + 1.5 * depth) * rs).max(1.4),
            white: 0.15,
            a: 0.75 + 0.25 * depth,
        });
    }

    dots.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal));
    OrbFrame { dots, lines }
}

/// 4. Ring (breathing): a calm dotted harmonic circle expanding & morphing
pub fn frame_ring(size: f32, t: f32) -> OrbFrame {
    let cx = size / 2.0;
    let cy = size / 2.0;
    let base_r = (size / 2.0) * 0.72;
    let rs = radius_scale(size, 0.6);
    let mut dots = Vec::with_capacity(90);

    let count = 48;
    let pulse = (t * 1.6).sin() * 0.5 + 0.5;

    for i in 0..count {
        let frac = i as f32 / count as f32;
        let angle = frac * 2.0 * std::f32::consts::PI;
        let wave = (angle * 3.0 + t * 2.2).sin() * 0.08 + (angle * 5.0 - t * 1.4).cos() * 0.04;
        let r = base_r * (0.86 + 0.14 * pulse + wave);

        let px = cx + angle.cos() * r;
        let py = cy + angle.sin() * r;
        let depth = (py - cy) / base_r;

        dots.push(OrbDot {
            x: px,
            y: py,
            z: depth,
            r: ((1.1 + 0.9 * pulse + wave.abs() * 3.0) * rs).max(1.0),
            white: 0.35 - 0.22 * pulse,
            a: 0.60 + 0.40 * pulse,
        });
    }

    dots.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal));
    OrbFrame { dots, lines: Vec::new() }
}

/// Generate a frame for the specified mode
pub fn render_orb_frame(mode: OrbMode, size: f32, t: f32) -> OrbFrame {
    match mode {
        OrbMode::Default => frame_globe(size, t),
        OrbMode::Thinking => frame_orbits(size, t),
        OrbMode::Searching => frame_globe(size, t),
        OrbMode::Analyzing => frame_web(size, t),
    }
}

/// Render the thinking orb directly to the UI using native 2D circle & line primitives
/// Exactly matching the Jakub Antalik thinking-orbs GitHub repository technique!
pub fn render_orb_canvas(
    ui: &mut zenthra_widgets::Ui,
    mode: OrbMode,
    stage_w: f32,
    stage_h: f32,
    t: f32,
    tint_rgb: [f32; 3],
) {
    let size = stage_w.min(stage_h) * 0.88;
    let ox = (stage_w - size) / 2.0;
    let oy = (stage_h - size) / 2.0;
    let frame = render_orb_frame(mode, size, t);

    // 1. Draw connecting lines
    for line in &frame.lines {
        let dx = line.x2 - line.x1;
        let dy = line.y2 - line.y1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let _angle = dy.atan2(dx);
        let _mid_x = ox + (line.x1 + line.x2) / 2.0;
        let _mid_y = oy + (line.y1 + line.y2) / 2.0;
        let line_w = line.w.max(0.75);

        // Discard or draw via rotated small segment steps or bounding rect
        let steps = (len / 3.0).ceil().max(1.0) as usize;
        for s in 0..=steps {
            let frac = s as f32 / steps as f32;
            let px = ox + line.x1 + dx * frac - line_w * 0.5;
            let py = oy + line.y1 + dy * frac - line_w * 0.5;
            let alpha = line.a * 0.65;
            let col = zenthra_core::Color::rgba(
                tint_rgb[0] * 0.9 + 0.1,
                tint_rgb[1] * 0.9 + 0.1,
                tint_rgb[2] * 0.9 + 0.1,
                alpha,
            );
            ui.container()
                .absolute(px, py)
                .width(line_w)
                .height(line_w)
                .radius_all(line_w * 0.5)
                .bg(col)
                .show(|_| {});
        }
    }

    // 2. Draw dots (nodes and intermediate constellation points)
    for dot in &frame.dots {
        let px = ox + dot.x - dot.r;
        let py = oy + dot.y - dot.r;
        let d = dot.r * 2.0;

        let wh = dot.white.clamp(0.0, 1.0);
        let col = zenthra_core::Color::rgba(
            tint_rgb[0] * (1.0 - wh * 0.3) + wh * 0.3,
            tint_rgb[1] * (1.0 - wh * 0.3) + wh * 0.3,
            tint_rgb[2] * (1.0 - wh * 0.3) + wh * 0.3,
            dot.a.clamp(0.05, 1.0),
        );

        ui.container()
            .absolute(px, py)
            .width(d)
            .height(d)
            .radius_all(dot.r)
            .bg(col)
            .show(|_| {});
    }
}

