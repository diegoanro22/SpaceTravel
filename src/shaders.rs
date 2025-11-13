use nalgebra_glm as glm;
use raylib::prelude::Color;

use crate::geom::Vec3;

#[derive(Copy, Clone)]
pub enum BodyShader {
    Star,
    Rocky,
    GasGiant,
    Moon,
    Mercury,
    Venus,
    Mars,
}

fn spherical_coords(local: Vec3) -> (f32, f32, f32) {
    let r = local.magnitude().max(1e-5);
    let nx = local.x / r;
    let ny = local.y / r;
    let nz = local.z / r;

    let lat = ny.asin();
    let lon = nz.atan2(nx);
    (lat, lon, r)
}

fn saturate(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

fn lerp(a: glm::Vec3, b: glm::Vec3, t: f32) -> glm::Vec3 {
    a * (1.0 - t) + b * t
}

fn cheap_noise(lat: f32, lon: f32, time: f32, scale: f32) -> f32 {
    let s = (lat * scale + time * 0.5).sin();
    let c = (lon * scale * 1.3 - time * 0.7).cos();
    let s2 = ((lat + lon) * scale * 0.7 + time * 0.31).sin();
    (0.5 * s + 0.35 * c + 0.15 * s2) * 0.8
}

//
// ----- SOL -----
//

fn shade_star(p: Vec3, n_view: Vec3, center: Vec3, radius: f32, time: f32) -> Color {
    let local = p - center;
    let (lat, lon, r_len) = spherical_coords(local);
    let r_norm = saturate(r_len / radius);

    let core = glm::vec3(1.05, 1.00, 0.94);
    let mid = glm::vec3(1.00, 0.82, 0.35);
    let corona = glm::vec3(1.00, 0.48, 0.08);
    let halo = glm::vec3(1.00, 0.90, 0.60);

    let mut t = r_norm * 1.2;
    t = saturate(t);
    let mut col = lerp(core, mid, t);

    let t2 = saturate((r_norm - 0.65) / 0.4);
    col = lerp(col, corona, t2 * 0.8);

    let gran_scale = 30.0;
    let g1 = (lat * gran_scale + time * 3.0).sin();
    let g2 = (lon * gran_scale * 1.2 - time * 2.1).cos();
    let g3 = ((lat * 0.7 + lon * 1.3) * gran_scale * 0.25 + time * 1.7).sin();
    let gran = (g1 * 0.45 + g2 * 0.35 + g3 * 0.20).clamp(-1.0, 1.0);

    let bright_cell = glm::vec3(1.3, 1.15, 0.7);
    let dark_cell = glm::vec3(0.7, 0.35, 0.1);
    let gran_t = 0.5 + 0.5 * gran;
    let gran_color = lerp(dark_cell, bright_cell, gran_t);

    col = col * 0.65 + gran_color * 0.35;

    let fil_scale = 6.0;
    let f1 = (lat * fil_scale + time * 0.7).sin();
    let f2 = (lon * fil_scale * 1.4 - time * 0.9).cos();
    let fil = (f1 * 0.6 + f2 * 0.4).clamp(-1.0, 1.0);
    let fil_intensity = saturate((fil - 0.1) * 2.0);
    let dark_fil = glm::vec3(0.35, 0.15, 0.06);
    col = col * (1.0 - fil_intensity * 0.55) + dark_fil * (fil_intensity * 0.55);

    let edge = saturate((r_norm - 0.8) / 0.25);
    let halo_factor = edge * edge;
    col = col * (1.0 + halo_factor * 0.4) + halo * (halo_factor * 0.3);

    let facing = (-n_view.z).max(0.0);
    let view_brightness = 0.5 + 0.6 * facing;
    col *= view_brightness;

    let pulse = 0.93 + 0.07 * (time * 2.3).sin();
    col *= pulse;

    let r = (col.x.clamp(0.0, 1.4) * 255.0) as u8;
    let g = (col.y.clamp(0.0, 1.4) * 255.0) as u8;
    let b = (col.z.clamp(0.0, 1.0) * 255.0) as u8;
    Color::new(r, g, b, 255)
}

//
// ----- TIERRA (ROCKY) -----
//

fn shade_rocky_earth(p: Vec3, _n_view: Vec3, center: Vec3, radius: f32, time: f32) -> Color {
    let local = p - center;
    let (lat, lon, r_len) = spherical_coords(local);
    let r_norm = (r_len / radius).min(1.0);

    let ocean = glm::vec3(0.02, 0.12, 0.40);
    let coast = glm::vec3(0.18, 0.42, 0.27);
    let land = glm::vec3(0.32, 0.55, 0.22);
    let desert = glm::vec3(0.74, 0.60, 0.38);
    let mountain = glm::vec3(0.78, 0.78, 0.82);
    let ice = glm::vec3(0.93, 0.97, 1.0);

    let n1 = (lon * 3.0 + (lat * 2.0).sin() * 1.3).sin();
    let n2 = (lon * 7.0 + lat * 5.0).cos();
    let n3 = ((lon * 11.0 + time * 0.25).sin() * (lat * 6.0).cos()) * 0.4;

    let mut h = 0.55 * n1 + 0.35 * n2 + 0.25 * n3;
    h = h.clamp(-1.0, 1.0);

    let mut col: glm::Vec3;
    if h < -0.15 {
        col = ocean;
    } else if h < 0.20 {
        let t = (h + 0.15) / 0.35;
        let t = t.clamp(0.0, 1.0);
        col = ocean * (1.0 - t) + coast * t;
    } else if h < 0.45 {
        let t = (h - 0.20) / 0.25;
        let t = t.clamp(0.0, 1.0);
        col = coast * (1.0 - t) + land * t;
    } else if h < 0.75 {
        let t = (h - 0.45) / 0.30;
        let t = t.clamp(0.0, 1.0);
        col = land * (1.0 - t) + desert * t;
    } else {
        let t = (h - 0.75) / 0.25;
        let t = t.clamp(0.0, 1.0);
        col = desert * (1.0 - t) + mountain * t;
    }

    let poles = (lat.abs() / (std::f32::consts::FRAC_PI_2)).powf(3.0);
    let ice_factor = poles.clamp(0.0, 1.0);
    col = col * (1.0 - ice_factor) + ice * ice_factor;

    let limb = 0.55 + 0.45 * (1.0 - r_norm * r_norm).max(0.0);
    col *= limb;

    let r = (col.x.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (col.y.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (col.z.clamp(0.0, 1.0) * 255.0) as u8;
    Color::new(r, g, b, 255)
}

//
// ----- MERCURIO -----
//

fn shade_mercury(p: Vec3, _n_view: Vec3, center: Vec3, radius: f32, time: f32) -> Color {
    let local = p - center;
    let (lat, lon, r_len) = spherical_coords(local);
    let r_norm = (r_len / radius).min(1.0);

    let base_dark = glm::vec3(0.25, 0.20, 0.18);
    let base_mid = glm::vec3(0.40, 0.32, 0.26);
    let base_bright = glm::vec3(0.65, 0.55, 0.42);

    let sun_dir = glm::vec3(1.0, 0.2, 0.0);
    let n = glm::vec3(local.x, local.y, local.z).normalize();
    let heat = saturate(glm::dot(&n, &sun_dir) * 0.6 + 0.4);

    let mut col = lerp(base_dark, base_mid, heat);
    col = lerp(col, base_bright, heat * 0.5);

    let n1 = cheap_noise(lat * 7.0, lon * 9.0, time * 0.15, 12.0);
    let n2 = cheap_noise(lat * 15.0, lon * 18.0, time * 0.07, 25.0);
    let grain = (n1 * 0.7 + n2 * 0.3).clamp(-1.0, 1.0);
    col += grain * glm::vec3(0.10, 0.08, 0.06);

    let mut crater_dark = glm::vec3(0.18, 0.16, 0.16);
    let craters = [
        (0.25_f32, 0.8_f32),
        (-0.10, -0.7),
        (0.05, 2.4),
        (-0.35, 1.5),
    ];
    for (c_lat, c_lon) in craters {
        let d_lat = lat - c_lat;
        let d_lon = (lon - c_lon + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI)
            - std::f32::consts::PI;
        let d2 = d_lat * d_lat + d_lon * d_lon;
        let crater = (-d2 * 130.0).exp();
        crater_dark = glm::vec3(0.15, 0.14, 0.14);
        col = col * (1.0 - crater * 0.7) + crater_dark * (crater * 0.7);
    }

    let limb = 0.55 + 0.45 * (1.0 - r_norm * r_norm).max(0.0);
    col *= limb;

    let r = (col.x.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (col.y.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (col.z.clamp(0.0, 1.0) * 255.0) as u8;
    Color::new(r, g, b, 255)
}

//
// ----- VENUS -----
//

fn shade_venus(p: Vec3, _n_view: Vec3, center: Vec3, radius: f32, time: f32) -> Color {
    let local = p - center;
    let (lat, lon, r_len) = spherical_coords(local);
    let r_norm = (r_len / radius).min(1.0);
    let y = lat / std::f32::consts::FRAC_PI_2;

    let deep_cloud = glm::vec3(0.80, 0.65, 0.30);
    let mid_cloud = glm::vec3(0.95, 0.80, 0.45);
    let high_cloud = glm::vec3(1.00, 0.92, 0.70);

    let t = (y * 0.4 + 0.5).clamp(0.0, 1.0);
    let mut col = lerp(deep_cloud, mid_cloud, t);
    col = lerp(col, high_cloud, 0.35);

    let band1 = (y * 6.0 + time * 0.4).sin();
    let band2 = (lon * 4.0 + y * 3.0 - time * 0.3).cos();
    let swirl = (band1 * 0.7 + band2 * 0.3).clamp(-1.0, 1.0);
    let swirl_color = glm::vec3(1.05, 0.90, 0.55);
    let swirl_t = 0.5 + 0.5 * swirl;
    col = col * (1.0 - swirl_t * 0.35) + swirl_color * (swirl_t * 0.35);

    let long_wave = (lon * 2.0 - time * 0.2).sin();
    let highlight = saturate((long_wave * 0.5 + 0.5) * (1.0 - y.abs()));
    let high_light_color = glm::vec3(1.15, 1.00, 0.80);
    col = col * (1.0 - highlight * 0.25) + high_light_color * (highlight * 0.4);

    let limb = 0.70 + 0.30 * (1.0 - r_norm * r_norm).max(0.0);
    col *= limb;

    let r = (col.x.clamp(0.0, 1.3) * 255.0) as u8;
    let g = (col.y.clamp(0.0, 1.2) * 255.0) as u8;
    let b = (col.z.clamp(0.0, 1.1) * 255.0) as u8;
    Color::new(r, g, b, 255)
}

//
// ----- MARTE -----
//

fn shade_mars(p: Vec3, _n_view: Vec3, center: Vec3, radius: f32, time: f32) -> Color {
    let local = p - center;
    let (lat, lon, r_len) = spherical_coords(local);
    let r_norm = (r_len / radius).min(1.0);
    let y = lat / std::f32::consts::FRAC_PI_2;

    let low_plain = glm::vec3(0.45, 0.18, 0.10);
    let mid_plain = glm::vec3(0.66, 0.26, 0.14);
    let high_plain = glm::vec3(0.78, 0.38, 0.20);
    let dust_storm = glm::vec3(0.90, 0.55, 0.30);
    let polar_ice = glm::vec3(0.96, 0.95, 0.92);

    let n1 = cheap_noise(lat * 3.0, lon * 4.0, time * 0.1, 10.0);
    let n2 = cheap_noise(lat * 6.0, lon * 7.0, time * 0.2, 18.0);
    let h = (0.6 * n1 + 0.4 * n2).clamp(-1.0, 1.0);

    let mut col = if h < -0.2 {
        low_plain
    } else if h < 0.3 {
        let t = (h + 0.2) / 0.5;
        lerp(low_plain, mid_plain, t.clamp(0.0, 1.0))
    } else {
        let t = (h - 0.3) / 0.7;
        lerp(mid_plain, high_plain, t.clamp(0.0, 1.0))
    };

    let dark_map = (lon * 5.0 - time * 0.15).sin() * (y * 3.0).cos();
    let dark_t = saturate((dark_map - 0.2) * 2.5);
    let dark_color = glm::vec3(0.22, 0.10, 0.08);
    col = col * (1.0 - dark_t * 0.5) + dark_color * (dark_t * 0.5);

    let storm_noise = cheap_noise(lat * 9.0, lon * 10.0, time * 0.5, 20.0);
    let storm_mask = saturate((storm_noise - 0.4) * 2.0);
    col = col * (1.0 - storm_mask * 0.4) + dust_storm * (storm_mask * 0.6);

    let polar = y.abs().powf(3.0);
    let polar_mask = saturate(polar * 1.8);
    col = col * (1.0 - polar_mask) + polar_ice * polar_mask;

    let limb = 0.55 + 0.45 * (1.0 - r_norm * r_norm).max(0.0);
    col *= limb;

    let r = (col.x.clamp(0.0, 1.1) * 255.0) as u8;
    let g = (col.y.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (col.z.clamp(0.0, 1.0) * 255.0) as u8;
    Color::new(r, g, b, 255)
}

//
// ----- JÚPITER (GAS GIANT) -----
//

fn shade_gas_giant_jupiter(p: Vec3, _n_view: Vec3, center: Vec3, radius: f32, time: f32) -> Color {
    let local = p - center;
    let (lat, lon, r_len) = spherical_coords(local);
    let r_norm = saturate(r_len / radius);

    let base_eq_light = glm::vec3(0.94, 0.86, 0.68);
    let base_eq_dark = glm::vec3(0.83, 0.64, 0.44);
    let base_mid1 = glm::vec3(0.78, 0.58, 0.40);
    let base_mid2 = glm::vec3(0.72, 0.50, 0.34);
    let base_high = glm::vec3(0.80, 0.72, 0.60);

    let y = lat / std::f32::consts::FRAC_PI_2;

    let band_pattern = (y * 5.0 + 0.4 * (lon * 1.5).sin() + time * 0.25).sin();
    let band_mix = 0.5 + 0.5 * band_pattern;

    let mut col = if band_mix < 0.25 {
        let t = band_mix / 0.25;
        lerp(base_high, base_mid2, t)
    } else if band_mix < 0.5 {
        let t = (band_mix - 0.25) / 0.25;
        lerp(base_mid2, base_mid1, t)
    } else if band_mix < 0.75 {
        let t = (band_mix - 0.5) / 0.25;
        lerp(base_mid1, base_eq_dark, t)
    } else {
        let t = (band_mix - 0.75) / 0.25;
        lerp(base_eq_dark, base_eq_light, t)
    };

    let stripe_noise = cheap_noise(lat * 4.0, lon * 8.0 + time * 1.2, time * 0.5, 15.0);
    let stripe_intensity = 0.15 * stripe_noise;
    let high_stripe_color = glm::vec3(1.05, 0.90, 0.70);
    col = col * (1.0 - stripe_intensity) + high_stripe_color * stripe_intensity;

    let spot_lat = -0.22;
    let spot_lon = time * 0.45;
    let d_lat = lat - spot_lat;
    let d_lon = (lon - spot_lon + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI)
        - std::f32::consts::PI;
    let d2 = d_lat * d_lat + d_lon * d_lon;
    let big_spot_strength = (-d2 * 7.0).exp();
    let big_spot_color = glm::vec3(1.05, 0.58, 0.32);
    col = col * (1.0 - big_spot_strength) + big_spot_color * big_spot_strength;

    let mut vortex_color = glm::vec3(0.95, 0.70, 0.45);
    let vortices = [(0.35, 1.2), (-0.05, 2.7), (0.10, -1.5), (-0.30, -2.2)];
    for (v_lat, v_lon_offset) in vortices {
        let d_lat_v = lat - v_lat;
        let v_time_lon = v_lon_offset + time * 0.7;
        let d_lon_v = (lon - v_time_lon + std::f32::consts::PI)
            .rem_euclid(2.0 * std::f32::consts::PI)
            - std::f32::consts::PI;
        let d2_v = d_lat_v * d_lat_v + d_lon_v * d_lon_v;
        let vortex_strength = (-d2_v * 18.0).exp();
        vortex_color = glm::vec3(0.98, 0.78, 0.52);
        col = col * (1.0 - vortex_strength * 0.7) + vortex_color * (vortex_strength * 0.7);
    }

    let haze = (1.0 - y.abs()).powf(2.0);
    let haze_color = glm::vec3(0.98, 0.92, 0.80);
    col = col * (1.0 - haze * 0.25) + haze_color * (haze * 0.25);

    let cyan = glm::vec3(0.40, 0.85, 1.15);
    let line_pattern =
        (lon * 20.0 + y * 6.0 - time * 2.0).sin() + (lon * 7.0 - y * 10.0 + time * 1.3).cos() * 0.5;

    let line_mask_raw = 0.5 + 0.5 * line_pattern;
    let line_mask = saturate((line_mask_raw - 0.7) * 4.0);
    let equator_boost = (1.0 - y.abs()).powf(2.0);
    let line_strength = line_mask * equator_boost;

    col = col * (1.0 - line_strength * 0.5) + cyan * (line_strength * 0.9);

    let limb = 0.55 + 0.45 * (1.0 - r_norm * r_norm).max(0.0);
    col *= limb;

    let breathe = 0.96 + 0.04 * (time * 0.8).sin();
    col *= breathe;

    let r = (col.x.clamp(0.0, 1.4) * 255.0) as u8;
    let g = (col.y.clamp(0.0, 1.3) * 255.0) as u8;
    let b = (col.z.clamp(0.0, 1.4) * 255.0) as u8;
    Color::new(r, g, b, 255)
}

//
// ----- LUNA -----
//

fn shade_moon(p: Vec3, _n_view: Vec3, center: Vec3, radius: f32, _time: f32) -> Color {
    let local = p - center;
    let (lat, lon, r_len) = spherical_coords(local);
    let r_norm = (r_len / radius).min(1.0);

    let base = glm::vec3(0.42, 0.42, 0.45);

    let n1 = cheap_noise(lat * 6.0, lon * 6.0, 0.0, 12.0);
    let n2 = cheap_noise(lat * 11.0, lon * 13.0, 0.0, 22.0);
    let grit = (n1 * 0.6 + n2 * 0.4).clamp(-1.0, 1.0);
    let mut col = base + grit * glm::vec3(0.10, 0.10, 0.12);

    let craters = [
        (0.10_f32, 0.20_f32, 0.22_f32),
        (-0.30, -1.0, 0.28),
        (0.35, 1.7, 0.20),
        (-0.10, 2.5, 0.18),
    ];

    for (c_lat, c_lon, r0) in craters {
        let d_lat = lat - c_lat;
        let d_lon = (lon - c_lon + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI)
            - std::f32::consts::PI;
        let d = (d_lat * d_lat + d_lon * d_lon).sqrt();

        let inner = (-d * 45.0).exp();
        let rim = (-(d - r0).powi(2) * 220.0).exp();

        let dark = glm::vec3(0.22, 0.22, 0.24);
        let light_rim = glm::vec3(0.85, 0.85, 0.90);

        col = col * (1.0 - inner * 0.6) + dark * (inner * 0.6);
        col = col * (1.0 - rim * 0.7) + light_rim * (rim * 0.7);
    }

    let limb = 0.50 + 0.50 * (1.0 - r_norm * r_norm).max(0.0);
    col *= limb;

    let r = (col.x.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (col.y.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (col.z.clamp(0.0, 1.0) * 255.0) as u8;
    Color::new(r, g, b, 255)
}

//
// ----- DISPATCH -----
//

pub fn shade_body(
    kind: BodyShader,
    p_model: Vec3,
    n_view: Vec3,
    center: Vec3,
    radius: f32,
    time: f32,
) -> Color {
    match kind {
        BodyShader::Star => shade_star(p_model, n_view, center, radius, time),
        BodyShader::Rocky => shade_rocky_earth(p_model, n_view, center, radius, time),
        BodyShader::GasGiant => shade_gas_giant_jupiter(p_model, n_view, center, radius, time),
        BodyShader::Moon => shade_moon(p_model, n_view, center, radius, time),
        BodyShader::Mercury => shade_mercury(p_model, n_view, center, radius, time),
        BodyShader::Venus => shade_venus(p_model, n_view, center, radius, time),
        BodyShader::Mars => shade_mars(p_model, n_view, center, radius, time),
    }
}
