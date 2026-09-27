// apps/zenthree/crates/zenthree/src/shaders/spherical_polyhedra.wgsl
// Unified 3D Dotted Sphere with Default, Thinking, Searching, and Analyzing states.
// 100% isotropic Fibonacci Golden Spiral point lattice.

@group(0) @binding(0) var t_src: texture_2d<f32>;
@group(0) @binding(1) var s_src: sampler;

struct BackdropUniforms {
    radius: vec4<f32>,
    rect_pos: vec2<f32>,
    rect_size: vec2<f32>,
    screen_size: vec2<f32>,
    time: f32,
    brightness: f32,
    saturation: f32,
    contrast: f32,
    blur_type: f32,
    opacity: f32,
    padding: vec2<f32>,
    _end_padding: vec2<f32>,
}

@group(1) @binding(0) var<uniform> u: BackdropUniforms;

struct BackdropVsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0)       uv:  vec2<f32>,
    @location(1)       local_pos: vec2<f32>,
    @location(2)       half_size: vec2<f32>,
    @location(3)       radius: vec4<f32>,
}

fn sdf_rounded_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    let corner = select(
        select(r.w, r.z, p.x > 0.0),
        select(r.x, r.y, p.x > 0.0),
        p.y > 0.0
    );
    let q = abs(p) - b + corner;
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - corner;
}

@fragment
fn fs_main(in: BackdropVsOut) -> @location(0) vec4<f32> {
    // 1. Container corner clipping via SDF
    let d_box = sdf_rounded_box(in.local_pos, in.half_size, in.radius);
    let aa_width = fwidth(d_box);
    let rect_alpha = 1.0 - smoothstep(-aa_width, aa_width, d_box);
    if (rect_alpha < 0.01) {
        discard;
    }

    // 2. Screen UV setup matching centered coordinates
    let uv = in.local_pos / in.half_size.y * 0.5;

    // 3. Time, State, & Orbital Movement
    let mode_idx = u.blur_type; // 0=Default, 1=Thinking, 2=Searching, 3=Analyzing
    let is_active = u.opacity;
    let speed_mult = max(u.padding.x, 0.5);
    let t = u.time * 0.42 * speed_mult;

    // 4. Perfectly Level Camera Setup (Zero tilt, pure horizontal eye-level)
    let cam_dist: f32 = 4.3;
    let ro = vec3<f32>(0.0, 0.0, cam_dist);
    let cw = vec3<f32>(0.0, 0.0, -1.0);
    let cu = vec3<f32>(1.0, 0.0, 0.0);
    let cv = vec3<f32>(0.0, 1.0, 0.0);

    let mouse_tilt_x = u.padding.y * 0.6;
    // Flipped direction: spin from Left to Right across front face
    let spin = -(t * 0.75 + mouse_tilt_x);

    let base_tint = vec3<f32>(u.brightness, u.saturation, u.contrast);

    // 5. Single Hollow 3D Sphere (Fibonacci Golden Spiral Lattice, r = 1.0)
    // No inner sphere — true 3D interior depth via transparent kinetic parallax and interior chords
    const N_DOTS: i32 = 560;
    const GOLDEN_ANGLE: f32 = 2.39996323; // π * (3 - sqrt(5))
    const FN_RECIP: f32 = 1.0 / 559.0;

    var col = vec3<f32>(0.0);

    for (var i: i32 = 0; i < N_DOTS; i = i + 1) {
        let fi = f32(i);
        // Uniform height distribution from +1.0 (north pole) to -1.0 (south pole)
        let y = 1.0 - (fi * FN_RECIP) * 2.0;
        let radius = sqrt(max(1.0 - y * y, 0.0));

        // Horizontal spin along latitude rings (left to right)
        let phi = fi * GOLDEN_ANGLE + spin;
        let sphere = vec3<f32>(
            radius * cos(phi),
            y,
            radius * sin(phi)
        );

        // In Thinking mode: organic heartbeat expansion breathing
        var pos = sphere;
        if (mode_idx > 0.5 && mode_idx < 1.5) {
            let heartbeat = sin(t * 3.8) * 0.035 * is_active;
            pos = sphere * (1.0 + heartbeat);
        } else if (mode_idx > 2.5) {
            // In Analyzing mode: organic wandering breathing displacement
            let wave_i = sin(t * 1.8 + fi * 0.45) * 0.045;
            pos = sphere * (1.0 + wave_i);
        }

        let ray = pos - ro;
        let depth = dot(ray, cw);
        if (depth <= 0.2) {
            continue;
        }

        // Kinetic Depth & Hollow Sphere Transparency:
        // sphere.z > 0 is front hemisphere; sphere.z < 0 is rear hemisphere seen through hollow interior
        let z_depth = sphere.z;
        // Softly fade extreme far-back horizon for crystal clarity
        let rear_fade = smoothstep(-0.88, -0.15, z_depth);
        if (rear_fade < 0.01) {
            continue;
        }

        let proj = vec2<f32>(dot(ray, cu), dot(ray, cv)) / depth;
        let oc = uv - proj;

        // ── Mode 3: Analyzing (Dynamic Constellation Web with Wandering Nodes & Signals) ──
        if (mode_idx > 2.5) {
            // Selected constellation hub nodes
            let is_hub = (i % 14 == 0);
            if (is_hub) {
                // Surface neighbor offsets on Fibonacci lattice
                let offset_a = 13;
                let offset_b = 21;

                // Edge 1: Neighbor (+13)
                if (i + offset_a < N_DOTS) {
                    let fj = f32(i + offset_a);
                    let y_j = 1.0 - (fj * FN_RECIP) * 2.0;
                    let r_j = sqrt(max(1.0 - y_j * y_j, 0.0));
                    let phi_j = fj * GOLDEN_ANGLE + spin;
                    
                    let wave_j = sin(t * 1.8 + fj * 0.45) * 0.045;
                    var pos_b = vec3<f32>(r_j * cos(phi_j), y_j, r_j * sin(phi_j)) * (1.0 + wave_j);

                    let ray_b = pos_b - ro;
                    let depth_b = dot(ray_b, cw);
                    let proj_b = vec2<f32>(dot(ray_b, cu), dot(ray_b, cv)) / depth_b;
                    let pa = uv - proj;
                    let ba = proj_b - proj;
                    let h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-5), 0.0, 1.0);
                    let line_d = length(pa - ba * h);

                    // Much wider hit radius for crisp, visible edges at any resolution
                    if (line_d < 0.012) {
                        let line_glow = smoothstep(0.011, 0.0008, line_d);
                        let edge_z = mix(z_depth, pos_b.z, h);
                        let depth_alpha = mix(0.35, 1.0, smoothstep(-0.4, 0.4, edge_z));

                        // Traveling signal spark shooting along the edge
                        let signal_speed = 3.2;
                        let signal_phase = fract(t * signal_speed * 0.4 + fi * 0.17);
                        let signal_dist = abs(h - signal_phase);
                        let signal_spark = exp(-signal_dist * signal_dist * 180.0);

                        let base_line_col = mix(vec3<f32>(0.52, 0.38, 0.85), vec3<f32>(0.85, 0.78, 1.0), smoothstep(-0.2, 0.5, edge_z));
                        let spark_col = vec3<f32>(1.0, 0.98, 1.0);
                        let line_col = mix(base_line_col, spark_col, signal_spark * 0.85);

                        col = col + line_col * (line_glow * depth_alpha * (1.1 + signal_spark * 3.5));
                    }
                }

                // Edge 2: Neighbor (+21)
                if (i + offset_b < N_DOTS) {
                    let fj = f32(i + offset_b);
                    let y_j = 1.0 - (fj * FN_RECIP) * 2.0;
                    let r_j = sqrt(max(1.0 - y_j * y_j, 0.0));
                    let phi_j = fj * GOLDEN_ANGLE + spin;
                    
                    let wave_j = sin(t * 1.8 + fj * 0.45) * 0.045;
                    var pos_b = vec3<f32>(r_j * cos(phi_j), y_j, r_j * sin(phi_j)) * (1.0 + wave_j);

                    let ray_b = pos_b - ro;
                    let depth_b = dot(ray_b, cw);
                    let proj_b = vec2<f32>(dot(ray_b, cu), dot(ray_b, cv)) / depth_b;
                    let pa = uv - proj;
                    let ba = proj_b - proj;
                    let h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-5), 0.0, 1.0);
                    let line_d = length(pa - ba * h);

                    if (line_d < 0.012) {
                        let line_glow = smoothstep(0.011, 0.0008, line_d);
                        let edge_z = mix(z_depth, pos_b.z, h);
                        let depth_alpha = mix(0.35, 1.0, smoothstep(-0.4, 0.4, edge_z));

                        // Traveling signal spark shooting along edge
                        let signal_speed = 3.6;
                        let signal_phase = fract(t * signal_speed * 0.35 + fi * 0.23);
                        let signal_dist = abs(h - signal_phase);
                        let signal_spark = exp(-signal_dist * signal_dist * 180.0);

                        let base_line_col = mix(vec3<f32>(0.52, 0.38, 0.85), vec3<f32>(0.85, 0.78, 1.0), smoothstep(-0.2, 0.5, edge_z));
                        let spark_col = vec3<f32>(1.0, 0.98, 1.0);
                        let line_col = mix(base_line_col, spark_col, signal_spark * 0.85);

                        col = col + line_col * (line_glow * depth_alpha * (1.1 + signal_spark * 3.5));
                    }
                }
            }
        }

        // Early exit for pixels far away from the point
        if (abs(oc.x) < 0.038 && abs(oc.y) < 0.038) {
            let d2 = dot(oc, oc);

            // Volumetric depth-of-field:
            // Front dots are crisp; rear dots seen through hollow sphere are slightly softer
            var dot_sigma = mix(24000.0, 44000.0, smoothstep(-0.4, 0.4, z_depth));
            if (mode_idx > 2.5) {
                if (i % 28 == 0) {
                    dot_sigma = mix(4500.0, 9500.0, smoothstep(-0.4, 0.4, z_depth)); // 3x larger major hub!
                } else if (i % 14 == 0) {
                    dot_sigma = mix(9000.0, 18000.0, smoothstep(-0.4, 0.4, z_depth)); // 2x larger minor hub!
                }
            }
            let dot_falloff = exp(-d2 * dot_sigma);
            if (dot_falloff < 0.003) {
                continue;
            }

            // Lighting & internal depth attenuation:
            // Front dots catch direct light; rear dots softly shine through the hollow sphere
            let front_light = 0.45 + 0.55 * max(z_depth, 0.0);
            let interior_atten = mix(0.28, 1.0, smoothstep(-0.6, 0.3, z_depth)) * rear_fade;
            var pt_intensity = front_light * 3.4 * interior_atten;

            let base_color = 0.5 + 0.5 * sphere;
            var tinted_color = mix(base_color, base_tint * (0.8 + 0.4 * base_color), 0.5);

            // Rear dots have subtle cool interior depth tint
            if (z_depth < 0.0) {
                let rear_tint = vec3<f32>(0.35, 0.50, 0.95);
                tinted_color = mix(tinted_color, rear_tint, smoothstep(0.0, -0.6, z_depth) * 0.45);
            }

            // ── Animation States ─────────────────────────────────────
            if (mode_idx > 2.5) {
                // ── Mode 3: Analyzing (Hierarchical Lavender Constellation Nodes) ──
                let is_major_hub = (i % 28 == 0);
                let is_minor_hub = (i % 14 == 0);

                if (is_major_hub) {
                    // Major focal hub: larger, brighter soft glow with breathing pulse
                    let pulse = 1.0 + 0.32 * sin(t * 2.2 + fi * 0.35);
                    pt_intensity *= mix(3.2, 5.0, smoothstep(-0.2, 0.5, z_depth)) * pulse;
                    let hub_color = mix(vec3<f32>(0.72, 0.60, 0.95), vec3<f32>(0.92, 0.86, 1.0), smoothstep(-0.1, 0.4, z_depth));
                    tinted_color = hub_color * (1.3 + 0.4 * pulse);
                } else if (is_minor_hub) {
                    // Minor hub: connector node with subtle breathing
                    let pulse = 1.0 + 0.20 * sin(t * 2.2 + fi * 0.35 + 1.57);
                    pt_intensity *= mix(2.0, 3.4, smoothstep(-0.2, 0.5, z_depth)) * pulse;
                    let node_color = mix(vec3<f32>(0.58, 0.46, 0.88), vec3<f32>(0.80, 0.72, 1.0), smoothstep(-0.1, 0.4, z_depth));
                    tinted_color = node_color * 1.25;
                } else {
                    // Subtle background sphere dots: soft lavender dust
                    pt_intensity *= 0.28;
                    let dust_color = vec3<f32>(0.45, 0.35, 0.70);
                    tinted_color = dust_color;
                }
            } else if (mode_idx > 1.5) {
                // ── Mode 2: Searching (Volumetric Radar Sweep Beam) ───────
                let dot_azimuth = atan2(sphere.z, sphere.x);
                let beam_angle = t * 3.8;
                let PI: f32 = 3.14159265;
                let diff = fract((dot_azimuth - beam_angle) / (2.0 * PI));

                let trail_len = 0.35;
                var radar_trail: f32 = 0.0;
                if (diff < trail_len) {
                    radar_trail = exp(-diff * 9.5);
                }
                let leading_spike = exp(-diff * 75.0);

                pt_intensity *= (1.0 + radar_trail * 2.6 + leading_spike * 3.2);

                let radar_cyan = vec3<f32>(0.08, 0.95, 1.0);
                let radar_core = vec3<f32>(0.85, 1.0, 1.0);
                tinted_color = mix(tinted_color, radar_cyan * 1.35, radar_trail * 0.90);
                tinted_color = mix(tinted_color, radar_core, leading_spike * 0.85);
            } else if (mode_idx > 0.5 || is_active > 0.05) {
                // ── Mode 1: Thinking (Concentric Neural Waves South to North) ──
                let wave_phase = sphere.y * 3.8 - t * 4.8;
                let wave_crest = pow(max(sin(wave_phase), 0.0), 3.0);
                pt_intensity *= (1.0 + wave_crest * 1.5 * is_active);

                let wave_tint = vec3<f32>(1.0, 0.78, 0.28);
                tinted_color = mix(tinted_color, wave_tint * 1.25, wave_crest * 0.70 * is_active);

                let dot_hash = fract(sin(dot(sphere, vec3<f32>(12.9898, 78.233, 45.164))) * 43758.5453);
                if (dot_hash > 0.82) {
                    let spark = pow(max(sin(t * 4.0 + dot_hash * 6.28), 0.0), 8.0);
                    pt_intensity += spark * 2.2 * is_active;
                    tinted_color = mix(tinted_color, vec3<f32>(1.0, 1.0, 0.95), spark * 0.60 * is_active);
                }
            }

            col = col + tinted_color * (pt_intensity * dot_falloff);
        }
    }

    // 6. Crisp Tone Mapping & Background Blending
    col = tanh(col * 1.35) * 1.15;
    let alpha = clamp(max(max(col.r, col.g), col.b) * 1.4, 0.0, 1.0);

    // Composite cleanly over container background (empty space between dots stays pure background)
    let bg_color = textureSample(t_src, s_src, in.uv);
    let final_rgb = bg_color.rgb * (1.0 - alpha * 0.92) + col * alpha;

    return vec4<f32>(final_rgb, rect_alpha);
}

