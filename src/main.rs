use macroquad::color::colors::*;
use macroquad::color::Color;
use macroquad::input::*;
use macroquad::prelude::vec2;
use macroquad::shapes::*;
use macroquad::texture::*;
use macroquad::time::get_frame_time;
use macroquad::window;
use macroquad::window::*;

use egui;
use image as image_rs;
use num;
use serde::{Deserialize, Serialize};
// use rand::Rng;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::sync::{mpsc, Arc, Mutex};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;

mod colorschemes;

fn log_event(event: &str, details: &str) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));
    let ts_ms = now.as_secs() * 1000 + now.subsec_millis() as u64;
    println!("{}ms [{}] {}", ts_ms, event, details);
}

#[derive(Clone, Debug)]
struct Point<T> {
    x: T,
    y: T,
}

impl Point<f64> {
    fn to_world(&self, singl: &Singleton) -> Point<f64> {
        let unit = map_screen_to_world(&singl);
        Point::<f64> {
            x: (self.x - singl.offset.1.x as f64 - screen_width() as f64 / 2f64) * unit
                + singl.center.x,
            y: -(self.y - singl.offset.1.y as f64 - screen_height() as f64 / 2f64) * unit
                + singl.center.y,
        }
    }

    fn to_world_with_dims(
        &self,
        singl: &Singleton,
        screen_width: f64,
        screen_height: f64,
    ) -> Point<f64> {
        let unit = map_screen_to_world_with_dims(singl, screen_width, screen_height);
        Point::<f64> {
            x: (self.x - singl.offset.1.x as f64 - screen_width / 2f64) * unit + singl.center.x,
            y: -(self.y - singl.offset.1.y as f64 - screen_height / 2f64) * unit + singl.center.y,
        }
    }
}

#[derive(Clone, Debug)]
struct Singleton {
    power: f64,
    scale: f64,
    target_scale: f64,
    max_iter: usize,
    colorscheme: usize,
    pallet: Vec<Color>,
    center: Point<f64>,
    target_center: Point<f64>,
    julia: Point<f64>,
    offset: (Point<f32>, Point<f32>),
    refresh: bool,
    last_refresh: Instant,
    refresh_limit: u64,
    last_zoom_input: Instant,
    zoom_cooldown_ms: u64,
    zoom_pending: bool,
    zoom_lerp: f64,
    render_debounce_ms: u64,
    preview_debounce_ms: u64,
    input_idle_ms: u64,
    last_input: Instant,
    preview_scale: f32,
    preview_while_interacting: bool,
    tile_size: usize,
    mouse_click: bool,
    egui: bool,
    animation: bool,
    animation_unit: f64,
    threads: usize,
    recolor: bool,
    snapshot_files: Vec<String>,
    snapshot_selected: usize,
    snapshot_last_scan: Instant,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SnapshotConfig {
    center_x: f64,
    center_y: f64,
    scale: f64,
    power: f64,
    max_iter: usize,
    colorscheme: usize,
    julia_x: f64,
    julia_y: f64,
}

#[derive(Clone, Debug)]
struct RenderCache {
    center: Point<f64>,
    scale: f64,
    power: f64,
    max_iter: usize,
    julia: Point<f64>,
    render_width: f32,
    render_height: f32,
    viewport_width: f32,
    viewport_height: f32,
    render_scale: f32,
    texture: Texture2D,
    iters: Vec<u16>,
    complete: bool,
}

struct InflightRender {
    cache: RenderCache,
    image: Image,
    tile_done: Vec<bool>,
    tile_cols: usize,
    tile_rows: usize,
    tile_size: usize,
    pending_tiles: usize,
    last_texture_update: Instant,
    texture_update_interval_ms: u64,
    texture_update_stride: usize,
}

#[derive(Clone, Debug)]
struct Tile {
    index: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

enum RenderMessage {
    Tile {
        id: u64,
        index: usize,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        iters: Vec<u16>,
    },
    Done { id: u64 },
}

impl Default for Singleton {
    fn default() -> Singleton {
        Singleton {
            power: 2.,
            scale: 1.,
            target_scale: 1.,
            max_iter: 55,
            colorscheme: 0,
            pallet: Vec::new(),
            center: Point { x: 0., y: 0. },
            target_center: Point { x: 0., y: 0. },
            julia: Point { x: 0., y: 0. },
            offset: (Point { x: 0., y: 0. }, Point { x: 0., y: 0. }),
            refresh: false,
            last_refresh: Instant::now(),
            refresh_limit: 100,
            last_zoom_input: Instant::now() - Duration::from_secs(10),
            zoom_cooldown_ms: 2000,
            zoom_pending: false,
            zoom_lerp: 12.0,
            render_debounce_ms: 1200,
            preview_debounce_ms: 120,
            input_idle_ms: 150,
            last_input: Instant::now() - Duration::from_secs(10),
            preview_scale: 0.35,
            preview_while_interacting: true,
            tile_size: 64,
            mouse_click: false,
            egui: true,
            animation: false,
            animation_unit: 0.01,
            threads: 8,
            recolor: false,
            snapshot_files: Vec::new(),
            snapshot_selected: 0,
            snapshot_last_scan: Instant::now() - Duration::from_secs(10),
        }
    }
}

impl Singleton {
    fn generate_colors(&mut self) {
        let colorscheme = colorschemes::colorschemes()[self.colorscheme].clone();
        self.pallet = Vec::new();
        let color = self.max_iter / (colorscheme.colors.len() - 1);
        for i in 0..(colorscheme.colors.len() - 1) {
            for j in 0..color {
                self.pallet.push(Color::new(
                    colorscheme.colors[i].r
                        + (colorscheme.colors[i + 1].r - colorscheme.colors[i].r)
                            * (j as f32 / color as f32),
                    colorscheme.colors[i].g
                        + (colorscheme.colors[i + 1].g - colorscheme.colors[i].g)
                            * (j as f32 / color as f32),
                    colorscheme.colors[i].b
                        + (colorscheme.colors[i + 1].b - colorscheme.colors[i].b)
                            * (j as f32 / color as f32),
                    colorscheme.colors[i].a
                        + (colorscheme.colors[i + 1].a - colorscheme.colors[i].a)
                            * (j as f32 / color as f32),
                ));
            }
        }
        while self.pallet.len() <= self.max_iter {
            self.pallet.push(BLACK);
        }
    }
}

fn mandelbrot_scalar(c_re: f64, c_im: f64, singl: &Singleton) -> u16 {
    if (singl.power - 2.0).abs() <= f64::EPSILON {
        let mut z_re = singl.julia.x;
        let mut z_im = singl.julia.y;
        let mut i = 0usize;
        while i < singl.max_iter && (z_re * z_re + z_im * z_im) <= 4.0 {
            let new_re = z_re * z_re - z_im * z_im + c_re;
            let new_im = 2.0 * z_re * z_im + c_im;
            z_re = new_re;
            z_im = new_im;
            i += 1;
        }
        return i.min(u16::MAX as usize) as u16;
    }

    let mut z = num::complex::Complex::<f64>::new(singl.julia.x, singl.julia.y);
    let c = num::complex::Complex::<f64>::new(c_re, c_im);
    let mut i: usize = 0;
    while i < singl.max_iter && z.l1_norm() <= 4f64 {
        z = z.powf(singl.power) + c;
        i += 1;
    }
    i.min(u16::MAX as usize) as u16
}

fn should_use_simd(singl: &Singleton) -> bool {
    if (singl.power - 2.0).abs() > f64::EPSILON {
        return false;
    }
    #[cfg(target_arch = "x86_64")]
    {
        return is_x86_feature_detected!("sse2");
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        return false;
    }
}

fn mandelbrot_pair(c0_re: f64, c0_im: f64, c1_re: f64, c1_im: f64, singl: &Singleton) -> (u16, u16) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        if !is_x86_feature_detected!("sse2") || (singl.power - 2.0).abs() > f64::EPSILON {
            let a = mandelbrot_scalar(c0_re, c0_im, singl);
            let b = mandelbrot_scalar(c1_re, c1_im, singl);
            return (a, b);
        }

        let max_iter = singl.max_iter as i32;
        let mut z_re = _mm_set1_pd(singl.julia.x);
        let mut z_im = _mm_set1_pd(singl.julia.y);
        let c_re = _mm_set_pd(c1_re, c0_re);
        let c_im = _mm_set_pd(c1_im, c0_im);
        let four = _mm_set1_pd(4.0);

        let mut iter0 = 0i32;
        let mut iter1 = 0i32;

        for _ in 0..max_iter {
            let z_re2 = _mm_mul_pd(z_re, z_re);
            let z_im2 = _mm_mul_pd(z_im, z_im);
            let mag2 = _mm_add_pd(z_re2, z_im2);
            let mask = _mm_cmple_pd(mag2, four);
            let mask_bits = _mm_movemask_pd(mask);
            if mask_bits == 0 {
                break;
            }

            let z_re_im = _mm_mul_pd(z_re, z_im);
            let new_re = _mm_add_pd(_mm_sub_pd(z_re2, z_im2), c_re);
            let new_im = _mm_add_pd(_mm_add_pd(z_re_im, z_re_im), c_im);

            z_re = _mm_or_pd(_mm_and_pd(mask, new_re), _mm_andnot_pd(mask, z_re));
            z_im = _mm_or_pd(_mm_and_pd(mask, new_im), _mm_andnot_pd(mask, z_im));

            if (mask_bits & 0b01) != 0 {
                iter0 += 1;
            }
            if (mask_bits & 0b10) != 0 {
                iter1 += 1;
            }
        }

        return (
            (iter0 as usize).min(u16::MAX as usize) as u16,
            (iter1 as usize).min(u16::MAX as usize) as u16,
        );
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        let a = mandelbrot_scalar(c0_re, c0_im, singl);
        let b = mandelbrot_scalar(c1_re, c1_im, singl);
        (a, b)
    }
}
fn map_screen_to_world(singl: &Singleton) -> f64 {
    map_screen_to_world_with_dims(singl, screen_width() as f64, screen_height() as f64)
}

fn map_screen_to_world_with_dims(singl: &Singleton, screen_width: f64, screen_height: f64) -> f64 {
    map_screen_to_world_with_dims_scale(singl.scale, screen_width, screen_height)
}

fn map_screen_to_world_with_dims_scale(scale: f64, screen_width: f64, screen_height: f64) -> f64 {
    let world_unit: f64;
    if screen_width < screen_height {
        world_unit = 4f64 / (screen_width * scale);
    } else {
        world_unit = 4f64 / (screen_height * scale);
    }
    world_unit
}

fn screen_point_to_world(point: Point<f64>, center: &Point<f64>, scale: f64, offset: &Point<f32>) -> Point<f64> {
    let unit = map_screen_to_world_with_dims_scale(scale, screen_width() as f64, screen_height() as f64);
    Point::<f64> {
        x: (point.x - offset.x as f64 - screen_width() as f64 / 2f64) * unit + center.x,
        y: -(point.y - offset.y as f64 - screen_height() as f64 / 2f64) * unit + center.y,
    }
}

fn apply_zoom_lerp(singl: &mut Singleton) {
    let dt = get_frame_time() as f64;
    let t = (dt * singl.zoom_lerp).min(1.0);
    if t <= 0.0 {
        return;
    }
    let scale_delta = singl.target_scale - singl.scale;
    if scale_delta.abs() < 1e-12 {
        singl.scale = singl.target_scale;
    } else {
        singl.scale += scale_delta * t;
    }

    let center_dx = singl.target_center.x - singl.center.x;
    let center_dy = singl.target_center.y - singl.center.y;
    if center_dx.abs() < 1e-12 && center_dy.abs() < 1e-12 {
        singl.center = singl.target_center.clone();
    } else {
        singl.center.x += center_dx * t;
        singl.center.y += center_dy * t;
    }
}

fn render_dimensions(screen_width: usize, screen_height: usize, render_scale: f32) -> (usize, usize) {
    let width = ((screen_width as f32) * render_scale).round().max(1.0) as usize;
    let height = ((screen_height as f32) * render_scale).round().max(1.0) as usize;
    (width, height)
}

fn tile_layout(width: usize, height: usize, tile_size: usize) -> (usize, usize) {
    let cols = (width + tile_size - 1) / tile_size;
    let rows = (height + tile_size - 1) / tile_size;
    (cols, rows)
}

fn tiles_checkerboard(width: usize, height: usize, tile_size: usize) -> (Vec<Tile>, usize, usize) {
    let (cols, rows) = tile_layout(width, height, tile_size);
    let mut rings: Vec<(Vec<(f32, Tile)>, Vec<(f32, Tile)>)> = Vec::new();
    let center_x = (cols as f32) * 0.5;
    let center_y = (rows as f32) * 0.5;
    let mut max_ring = 0usize;

    for row in 0..rows {
        for col in 0..cols {
            let x = col * tile_size;
            let y = row * tile_size;
            let width_tile = (tile_size).min(width.saturating_sub(x));
            let height_tile = (tile_size).min(height.saturating_sub(y));
            let index = row * cols + col;
            let dx = col as f32 + 0.5 - center_x;
            let dy = row as f32 + 0.5 - center_y;
            let dist = (dx * dx + dy * dy).sqrt();
            let ring = dist.floor() as usize;
            if ring > max_ring {
                max_ring = ring;
            }
            if rings.len() <= ring {
                rings.resize_with(ring + 1, || (Vec::new(), Vec::new()));
            }
            let tile = Tile {
                index,
                x,
                y,
                width: width_tile,
                height: height_tile,
            };
            if (row + col) % 2 == 0 {
                rings[ring].0.push((dist, tile));
            } else {
                rings[ring].1.push((dist, tile));
            }
        }
    }

    let mut tiles = Vec::with_capacity(cols * rows);
    let seed_radius_tiles = 3usize.min(max_ring);
    for ring in 0..=max_ring {
        let (even, odd) = &mut rings[ring];
        even.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        odd.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        if ring <= seed_radius_tiles {
            let mut merged = Vec::with_capacity(even.len() + odd.len());
            merged.extend(even.iter().cloned());
            merged.extend(odd.iter().cloned());
            merged.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            tiles.extend(merged.into_iter().map(|entry| entry.1));
        } else {
            tiles.extend(even.iter().cloned().map(|entry| entry.1));
            tiles.extend(odd.iter().cloned().map(|entry| entry.1));
        }
    }
    (tiles, cols, rows)
}

fn empty_render_image(width: usize, height: usize) -> Image {
    Image::gen_image_color(width as u16, height as u16, Color::new(0., 0., 0., 0.))
}

fn image_from_iters(iters: &[u16], width: usize, height: usize, pallet: &[Color]) -> Image {
    let mut image = Image::gen_image_color(width as u16, height as u16, Color::new(0., 0., 0., 1.0));
    let max_index = pallet.len().saturating_sub(1);
    for y in 0..height as u32 {
        for x in 0..width as u32 {
            let index = (y as usize) * width + x as usize;
            let iter = iters[index] as usize;
            let color = pallet[iter.min(max_index)];
            image.set_pixel(x, y, color);
        }
    }
    image
}

fn image_from_iters_with_mask(
    iters: &[u16],
    width: usize,
    height: usize,
    pallet: &[Color],
    tile_done: &[bool],
    tile_cols: usize,
    tile_rows: usize,
    tile_size: usize,
) -> Image {
    let mut image = empty_render_image(width, height);
    let max_index = pallet.len().saturating_sub(1);
    for row in 0..tile_rows {
        for col in 0..tile_cols {
            let tile_index = row * tile_cols + col;
            if !tile_done.get(tile_index).copied().unwrap_or(false) {
                continue;
            }
            let x0 = col * tile_size;
            let y0 = row * tile_size;
            let tile_w = tile_size.min(width.saturating_sub(x0));
            let tile_h = tile_size.min(height.saturating_sub(y0));
            for y in 0..tile_h {
                let y_abs = y0 + y;
                let row_offset = y_abs * width + x0;
                for x in 0..tile_w {
                    let iter = iters[row_offset + x] as usize;
                    let color = pallet[iter.min(max_index)];
                    image.set_pixel((x0 + x) as u32, (y0 + y) as u32, color);
                }
            }
        }
    }
    image
}

fn update_image_tile(
    image: &mut Image,
    tile_iters: &[u16],
    tile_x: usize,
    tile_y: usize,
    tile_w: usize,
    tile_h: usize,
    pallet: &[Color],
) {
    let max_index = pallet.len().saturating_sub(1);
    for y in 0..tile_h {
        let row_offset = y * tile_w;
        for x in 0..tile_w {
            let iter = tile_iters[row_offset + x] as usize;
            let color = pallet[iter.min(max_index)];
            image.set_pixel((tile_x + x) as u32, (tile_y + y) as u32, color);
        }
    }
}

fn texture_from_image(image: &Image) -> Texture2D {
    let texture = Texture2D::from_image(image);
    texture.set_filter(FilterMode::Linear);
    texture
}

fn recolor_cache(cache: &mut RenderCache, pallet: &[Color]) {
    let width = cache.render_width as usize;
    let height = cache.render_height as usize;
    let image = image_from_iters(&cache.iters, width, height, pallet);
    cache.texture = texture_from_image(&image);
}

fn draw_cached_texture(cache: &RenderCache, singl: &Singleton) {
    let screen_w = screen_width();
    let screen_h = screen_height();
    let unit_old = map_screen_to_world_with_dims_scale(
        cache.scale,
        cache.viewport_width as f64,
        cache.viewport_height as f64,
    );
    let unit_new = map_screen_to_world_with_dims_scale(singl.scale, screen_w as f64, screen_h as f64);
    let scale_world = (unit_old / unit_new) as f32;
    let scale_render_x = cache.viewport_width / cache.render_width.max(1.0);
    let scale_render_y = cache.viewport_height / cache.render_height.max(1.0);
    let scale_x = scale_world * scale_render_x;
    let scale_y = scale_world * scale_render_y;

    let offset_x = screen_w / 2.0
        - (cache.render_width * scale_x) / 2.0
        + ((cache.center.x - singl.center.x) / unit_new) as f32
        + singl.offset.1.x;
    let offset_y = screen_h / 2.0
        - (cache.render_height * scale_y) / 2.0
        + ((singl.center.y - cache.center.y) / unit_new) as f32
        + singl.offset.1.y;

    let dest_size = vec2(cache.texture.width() * scale_x, cache.texture.height() * scale_y);
    draw_texture_ex(
        &cache.texture,
        offset_x,
        offset_y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(dest_size),
            ..Default::default()
        },
    );
}

fn start_fractal_job(
    singl: &Singleton,
    render_width: usize,
    render_height: usize,
    viewport_width: usize,
    viewport_height: usize,
    render_id: u64,
    active_render_id: Arc<AtomicU64>,
    tiles: Vec<Tile>,
    total_tiles: usize,
    sender: mpsc::Sender<RenderMessage>,
) {
    log_event(
        "render_start",
        &format!(
            "id={} size={}x{} render={}x{} tiles={} threads={} max_iter={} power={} scale={}",
            render_id,
            viewport_width,
            viewport_height,
            render_width,
            render_height,
            total_tiles,
            singl.threads,
            singl.max_iter,
            singl.power,
            singl.scale
        ),
    );
    let singl_clone = singl.clone();
    thread::spawn(move || {
        let tiles_mutex = Arc::new(Mutex::new(tiles));
        let singl_mutex = Arc::new(singl_clone);
        let completed = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..singl_mutex.threads {
            let singl_local = Arc::clone(&singl_mutex);
            let tiles_clone = Arc::clone(&tiles_mutex);
            let sender_clone = sender.clone();
            let completed_clone = Arc::clone(&completed);
            let active_id = Arc::clone(&active_render_id);

            let handle = thread::spawn(move || loop {
                if active_id.load(Ordering::SeqCst) != render_id {
                    break;
                }
                let mut tiles = tiles_clone.lock().unwrap();
                if tiles.is_empty() {
                    break;
                }
                let tile = tiles.remove(0);
                drop(tiles);

                if active_id.load(Ordering::SeqCst) != render_id {
                    break;
                }

                let unit = map_screen_to_world_with_dims_scale(
                    singl_local.scale,
                    viewport_width as f64,
                    viewport_height as f64,
                );
                let half_w = viewport_width as f64 * 0.5;
                let half_h = viewport_height as f64 * 0.5;
                let scale_x = viewport_width as f64 / render_width as f64;
                let scale_y = viewport_height as f64 / render_height as f64;
                let mut iters = vec![0u16; tile.width * tile.height];
                let use_simd = should_use_simd(&singl_local);

                for y in 0..tile.height {
                    if active_id.load(Ordering::SeqCst) != render_id {
                        return;
                    }
                    let y_abs = tile.y + y;
                    let screen_y = (y_abs as f64) * scale_y;
                    let world_y = -((screen_y) - half_h) * unit + singl_local.center.y;
                    let mut x = 0usize;
                    while x < tile.width {
                        let x_abs = tile.x + x;
                        let screen_x0 = (x_abs as f64) * scale_x;
                        let world_x0 = (screen_x0 - half_w) * unit + singl_local.center.x;
                        if use_simd && x + 1 < tile.width {
                            let screen_x1 = ((x_abs + 1) as f64) * scale_x;
                            let world_x1 = (screen_x1 - half_w) * unit + singl_local.center.x;
                            let (iter0, iter1) = mandelbrot_pair(world_x0, world_y, world_x1, world_y, &singl_local);
                            let offset = y * tile.width + x;
                            iters[offset] = iter0;
                            iters[offset + 1] = iter1;
                            x += 2;
                        } else {
                            let iter = mandelbrot_scalar(world_x0, world_y, &singl_local);
                            let offset = y * tile.width + x;
                            iters[offset] = iter;
                            x += 1;
                        }
                    }
                }

                if active_id.load(Ordering::SeqCst) != render_id {
                    break;
                }
                let _ = sender_clone.send(RenderMessage::Tile {
                    id: render_id,
                    index: tile.index,
                    x: tile.x,
                    y: tile.y,
                    width: tile.width,
                    height: tile.height,
                    iters,
                });

                let finished = completed_clone.fetch_add(1, Ordering::SeqCst) + 1;
                if finished == total_tiles {
                    log_event("render_done", &format!("id={} tiles={}", render_id, total_tiles));
                    if active_id.load(Ordering::SeqCst) == render_id {
                        let _ = sender_clone.send(RenderMessage::Done { id: render_id });
                    }
                }

                thread::sleep(Duration::from_millis(1));
            });
            handles.push(handle);
        }

        for h in handles {
            let _ = h.join();
        }
    });
}

fn adjusted_render_state(singl: &Singleton) -> Singleton {
    let mut render = singl.clone();
    if singl.mouse_click {
        let unit = map_screen_to_world(singl);
        render.center.x -= singl.offset.1.x as f64 * unit;
        render.center.y += singl.offset.1.y as f64 * unit;
    }
    render
}

fn start_render(
    singl: &Singleton,
    screen_w: usize,
    screen_h: usize,
    render_scale: f32,
    render_id: &mut u64,
    active_render_id: &Arc<AtomicU64>,
    sender: &mpsc::Sender<RenderMessage>,
) -> InflightRender {
    let render_scale = render_scale.clamp(0.1, 1.0);
    let (render_w, render_h) = render_dimensions(screen_w, screen_h, render_scale);
    let tile_size = singl.tile_size.max(8);
    let (tiles, tile_cols, tile_rows) = tiles_checkerboard(render_w, render_h, tile_size);
    let total_tiles = tiles.len();
    *render_id = render_id.wrapping_add(1);
    active_render_id.store(*render_id, Ordering::SeqCst);

    let iters = vec![0u16; render_w * render_h];
    let image = empty_render_image(render_w, render_h);
    let texture = texture_from_image(&image);
    let cache = RenderCache {
        center: singl.center.clone(),
        scale: singl.scale,
        power: singl.power,
        max_iter: singl.max_iter,
        julia: singl.julia.clone(),
        render_width: render_w as f32,
        render_height: render_h as f32,
        viewport_width: screen_w as f32,
        viewport_height: screen_h as f32,
        render_scale,
        texture,
        iters,
        complete: false,
    };

    start_fractal_job(
        singl,
        render_w,
        render_h,
        screen_w,
        screen_h,
        *render_id,
        Arc::clone(active_render_id),
        tiles,
        total_tiles,
        sender.clone(),
    );

    InflightRender {
        cache,
        image,
        tile_done: vec![false; tile_cols * tile_rows],
        tile_cols,
        tile_rows,
        tile_size,
        pending_tiles: 0,
        last_texture_update: Instant::now(),
        texture_update_interval_ms: 33,
        texture_update_stride: 4,
    }
}

fn select_cache_index(caches: &[RenderCache], singl: &Singleton) -> Option<usize> {
    if caches.is_empty() {
        return None;
    }
    let mut best_index = None;
    let mut best_score = f64::INFINITY;
    let screen_w = screen_width() as f64;
    let screen_h = screen_height() as f64;
    let unit_new = map_screen_to_world_with_dims_scale(singl.scale, screen_w, screen_h);

    for (index, cache) in caches.iter().enumerate() {
        if cache.max_iter != singl.max_iter
            || (cache.power - singl.power).abs() > f64::EPSILON
            || (cache.julia.x - singl.julia.x).abs() > f64::EPSILON
            || (cache.julia.y - singl.julia.y).abs() > f64::EPSILON
        {
            continue;
        }
        let unit_old = map_screen_to_world_with_dims_scale(
            cache.scale,
            cache.viewport_width as f64,
            cache.viewport_height as f64,
        );
        let scale_score = (unit_old / unit_new).ln().abs();
        let res_penalty = (1.0 / cache.render_scale.max(0.01) as f64).ln().max(0.0) * 0.5;
        let dx = (cache.center.x - singl.center.x).abs() / unit_new;
        let dy = (cache.center.y - singl.center.y).abs() / unit_new;
        let incomplete_penalty = if cache.complete { 0.0 } else { 1.0 };
        let score = scale_score * 2.0 + (dx + dy) / 1000.0 + res_penalty + incomplete_penalty;
        if score < best_score {
            best_score = score;
            best_index = Some(index);
        }
    }
    best_index
}

fn draw_menus(singl: &mut Singleton) {
    if singl.egui {
        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Settings").show(egui_ctx, |ui| {
                let mut needs_refresh = false;
                needs_refresh |= ui
                    .add(egui::Slider::new(&mut singl.scale, 1f64..=1e12f64).text("Zoom"))
                    .changed();
                needs_refresh |= ui
                    .add(egui::Slider::new(&mut singl.max_iter, 0..=10_000).text("Max iterations"))
                    .changed();
                needs_refresh |= ui
                    .add(egui::Slider::new(&mut singl.power, 0.0..=100.0).text("Power"))
                    .changed();
                needs_refresh |= ui
                    .add(egui::Slider::new(&mut singl.threads, 1..=16).text("Threads"))
                    .changed();
                needs_refresh |= ui
                    .add(
                        egui::Slider::new(&mut singl.tile_size, 16usize..=256usize)
                            .text("Tile size"),
                    )
                    .changed();
                needs_refresh |= ui
                    .add(
                        egui::Slider::new(&mut singl.animation_unit, 0.0001..=0.1)
                            .text("Animation unit"),
                    )
                    .changed();
                needs_refresh |= ui
                    .add(
                        egui::Slider::new(&mut singl.refresh_limit, 10..=10000).text("Redraw delay"),
                    )
                    .changed();
                needs_refresh |= ui
                    .add(
                        egui::Slider::new(&mut singl.render_debounce_ms, 0u64..=5000u64)
                            .text("Render debounce (ms)"),
                    )
                    .changed();
                needs_refresh |= ui
                    .add(
                        egui::Slider::new(&mut singl.preview_debounce_ms, 0u64..=500u64)
                            .text("Preview debounce (ms)"),
                    )
                    .changed();
                needs_refresh |= ui
                    .add(
                        egui::Slider::new(&mut singl.preview_scale, 0.1f32..=1.0f32)
                            .text("Preview scale"),
                    )
                    .changed();
                needs_refresh |= ui
                    .checkbox(&mut singl.preview_while_interacting, "Preview while moving")
                    .changed();

                if needs_refresh {
                    singl.target_scale = singl.scale;
                    singl.target_center = singl.center.clone();
                    singl.refresh = true;
                    singl.last_refresh = Instant::now();
                    singl.last_input = Instant::now();
                }

                let schemes = colorschemes::colorschemes();
                let mut selected = singl.colorscheme;
                egui::ComboBox::from_label("Colorscheme")
                    .selected_text(schemes[selected].name)
                    .show_ui(ui, |ui| {
                        for (index, scheme) in schemes.iter().enumerate() {
                            if ui
                                .selectable_label(index == selected, scheme.name)
                                .clicked()
                            {
                                selected = index;
                            }
                        }
                    });
                if selected != singl.colorscheme {
                    singl.colorscheme = selected;
                    singl.generate_colors();
                    singl.recolor = true;
                }

                if ui.button("Refresh").clicked() {
                    singl.refresh = true;
                    singl.mouse_click = false;
                    singl.offset = (Point { x: 0., y: 0. }, Point { x: 0., y: 0. });
                    singl.last_refresh =
                        Instant::now() - Duration::from_millis(singl.refresh_limit + 1);
                    singl.last_input = Instant::now();
                }
                if ui.button("Animation on/off").clicked() {
                    singl.animation = !singl.animation;
                    singl.last_input = Instant::now();
                }
                if singl.snapshot_last_scan.elapsed().as_millis() > 1000 {
                    singl.snapshot_files = list_snapshot_configs("captures");
                    if singl.snapshot_selected >= singl.snapshot_files.len() {
                        singl.snapshot_selected = 0;
                    }
                    singl.snapshot_last_scan = Instant::now();
                }
                if ui.button("Refresh snapshots").clicked() {
                    singl.snapshot_files = list_snapshot_configs("captures");
                    if singl.snapshot_selected >= singl.snapshot_files.len() {
                        singl.snapshot_selected = 0;
                    }
                    singl.snapshot_last_scan = Instant::now();
                }
                let snapshot_label = singl
                    .snapshot_files
                    .get(singl.snapshot_selected)
                    .map(|path| path.split('/').last().unwrap_or(path))
                    .unwrap_or("(none)");
                egui::ComboBox::from_label("Snapshot config")
                    .selected_text(snapshot_label)
                    .show_ui(ui, |ui| {
                        for (index, path) in singl.snapshot_files.iter().enumerate() {
                            let label = path.split('/').last().unwrap_or(path);
                            if ui.selectable_label(index == singl.snapshot_selected, label).clicked() {
                                singl.snapshot_selected = index;
                            }
                        }
                    });
                if ui.button("Load selected config").clicked() {
                    if let Some(path) = singl.snapshot_files.get(singl.snapshot_selected) {
                        if let Some(config) = load_snapshot_config(path) {
                            apply_snapshot_config(singl, config);
                        }
                    }
                }
                if ui.button("Save snapshot").clicked() {
                    save_snapshot(singl);
                }
                if ui.button("Load last config").clicked() {
                    if let Some(config) = load_latest_snapshot_config("captures") {
                        apply_snapshot_config(singl, config);
                    }
                }
            });

            egui::Window::new("Debugg info").show(egui_ctx, |ui| {
                ui.label(format!("Scale: {}", singl.scale));
                ui.label(format!("Iterations: {}", singl.max_iter));
                ui.label(format!("Refresh: {}", singl.refresh));
                ui.label(format!("Mouse click: {}", singl.mouse_click));

                ui.collapsing("Positions", |ui| {
                    ui.label(format!("Center: ({}, {})", singl.center.x, singl.center.y));
                    ui.label(format!(
                        "Offset: ({}, {}), ({}, {})",
                        singl.offset.0.x, singl.offset.0.y, singl.offset.1.x, singl.offset.1.y
                    ));
                    ui.label(format!("Mouse position: {:?}", mouse_position()));
                    ui.label(format!(
                        "World position: {:?}",
                        Point::<f64> {
                            x: mouse_position().0 as f64,
                            y: mouse_position().1 as f64
                        }
                        .to_world(&singl)
                    ));
                });

                ui.collapsing("Colors", |ui| {
                    ui.monospace(format!("{:#?}", singl.pallet.len()));
                });

                if ui.button("Reset").clicked() {
                    *singl = Singleton {
                        ..Default::default()
                    };
                    singl.refresh = true;
                    singl.last_refresh =
                        Instant::now() - Duration::from_millis(singl.refresh_limit + 1);
                    singl.last_input = Instant::now();
                }
                if ui.button("Center").clicked() {
                    singl.center = Point::<f64> { x: 0., y: 0. };
                    singl.offset = (Point { x: 0., y: 0. }, Point { x: 0., y: 0. });
                    singl.mouse_click = false;
                    singl.refresh = true;
                    singl.last_refresh =
                        Instant::now() - Duration::from_millis(singl.refresh_limit + 1);
                    singl.last_input = Instant::now();
                }
            });
        });
        egui_macroquad::draw();
    }
}

fn save_snapshot(singl: &Singleton) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    let folder = "captures";
    if fs::create_dir_all(folder).is_err() {
        return;
    }

    let image_path = format!("{}/fractal_{}.jpg", folder, timestamp);
    let meta_path = format!("{}/fractal_{}.json", folder, timestamp);

    let image = get_screen_data();
    let _ = save_jpeg(&image, &image_path);

    let config = SnapshotConfig {
        center_x: singl.center.x,
        center_y: singl.center.y,
        scale: singl.scale,
        power: singl.power,
        max_iter: singl.max_iter,
        colorscheme: singl.colorscheme,
        julia_x: singl.julia.x,
        julia_y: singl.julia.y,
    };
    if let Ok(json) = serde_json::to_string_pretty(&config) {
        let _ = fs::write(&meta_path, json);
    }
}

fn save_jpeg(image: &Image, path: &str) -> Result<(), image_rs::ImageError> {
    let width = image.width as u32;
    let height = image.height as u32;
    let mut rgb = Vec::with_capacity((width * height * 3) as usize);
    let bytes = &image.bytes;
    for chunk in bytes.chunks_exact(4) {
        rgb.push(chunk[0]);
        rgb.push(chunk[1]);
        rgb.push(chunk[2]);
    }
    let buffer = image_rs::RgbImage::from_raw(width, height, rgb)
        .ok_or_else(|| image_rs::ImageError::Parameter(image_rs::error::ParameterError::from_kind(
            image_rs::error::ParameterErrorKind::DimensionMismatch,
        )))?;
    buffer.save_with_format(path, image_rs::ImageFormat::Jpeg)
}

fn list_snapshot_configs(folder: &str) -> Vec<String> {
    let mut entries: Vec<(u64, String)> = Vec::new();
    if let Ok(dir) = fs::read_dir(folder) {
        for entry in dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
            let timestamp = name
                .strip_prefix("fractal_")
                .and_then(|rest| rest.strip_suffix(".json"))
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(0);
            entries.push((timestamp, path.to_string_lossy().to_string()));
        }
    }
    entries.sort_by(|a, b| b.0.cmp(&a.0));
    entries.into_iter().map(|(_, path)| path).collect()
}

fn latest_snapshot_path(folder: &str) -> Option<String> {
    let mut best: Option<(u64, String)> = None;
    let entries = fs::read_dir(folder).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
        let timestamp = name
            .strip_prefix("fractal_")
            .and_then(|rest| rest.strip_suffix(".json"))
            .and_then(|value| value.parse::<u64>().ok());
        if let Some(ts) = timestamp {
            let path_string = path.to_string_lossy().to_string();
            if best.as_ref().map(|(best_ts, _)| ts > *best_ts).unwrap_or(true) {
                best = Some((ts, path_string));
            }
        }
    }
    best.map(|(_, path)| path)
}

fn load_latest_snapshot_config(folder: &str) -> Option<SnapshotConfig> {
    let path = latest_snapshot_path(folder)?;
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn load_snapshot_config(path: &str) -> Option<SnapshotConfig> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn apply_snapshot_config(singl: &mut Singleton, config: SnapshotConfig) {
    let schemes = colorschemes::colorschemes();
    singl.center = Point {
        x: config.center_x,
        y: config.center_y,
    };
    singl.target_center = singl.center.clone();
    singl.scale = config.scale.max(1.0);
    singl.target_scale = singl.scale;
    singl.power = config.power;
    singl.max_iter = config.max_iter;
    singl.colorscheme = config.colorscheme.min(schemes.len().saturating_sub(1));
    singl.julia = Point {
        x: config.julia_x,
        y: config.julia_y,
    };
    singl.generate_colors();
    singl.recolor = true;
    singl.refresh = true;
    singl.last_refresh = Instant::now() - Duration::from_millis(singl.refresh_limit + 1);
    singl.last_input = Instant::now();
}

fn user_input(singl: &mut Singleton) {
    let xrest = 300.;
    let yrest = 400.;
    if singl.egui {
        draw_rectangle(0., 0., xrest, yrest, Color::new(0., 0., 0., 0.2));
    }
    if mouse_position().0 > xrest || mouse_position().1 > yrest || !singl.egui {
        if is_mouse_button_pressed(MouseButton::Right) {
            let mouse = mouse_position();
            singl.julia = Point::<f64> {
                x: mouse.0 as f64,
                y: mouse.1 as f64,
            }
            .to_world(&singl);
            singl.refresh = true;
            singl.last_refresh =
                Instant::now() - Duration::from_millis(singl.refresh_limit + 1);
            singl.last_input = Instant::now();
            log_event(
                "julia_set",
                &format!("world=({}, {})", singl.julia.x, singl.julia.y),
            );
        }
        if is_mouse_button_pressed(MouseButton::Left) && !singl.mouse_click {
            let mouse = mouse_position();
            singl.mouse_click = true;
            singl.offset = (
                Point::<f32> {
                    x: mouse.0,
                    y: mouse.1,
                },
                Point::<f32> { x: 0., y: 0. },
            );
            singl.last_input = Instant::now();
            log_event(
                "drag_start",
                &format!("screen=({}, {})", mouse.0, mouse.1),
            );
        }

        if is_mouse_button_down(MouseButton::Left) && singl.mouse_click {
            let mouse = mouse_position();
            singl.offset.1.x = mouse.0 - singl.offset.0.x;
            singl.offset.1.y = mouse.1 - singl.offset.0.y;
            singl.last_input = Instant::now();
        }

        if is_mouse_button_released(MouseButton::Left) && singl.mouse_click {
            let unit = map_screen_to_world(singl);
            singl.center.x -= singl.offset.1.x as f64 * unit;
            singl.center.y += singl.offset.1.y as f64 * unit;
            singl.offset = (Point { x: 0., y: 0. }, Point { x: 0., y: 0. });
            singl.mouse_click = false;
            singl.target_center = singl.center.clone();
            singl.refresh = true;
            singl.last_refresh = Instant::now();
            singl.last_input = Instant::now();
            log_event(
                "drag_end",
                &format!("center=({}, {})", singl.center.x, singl.center.y),
            );
        }

        if mouse_wheel().1 != 0. {
            let mouse = mouse_position();
            let point = Point::<f64> {
                x: mouse.0 as f64,
                y: mouse.1 as f64,
            };
            let before = screen_point_to_world(
                point.clone(),
                &singl.target_center,
                singl.target_scale,
                &singl.offset.1,
            );
            let mut new_scale = singl.target_scale + singl.target_scale * (mouse_wheel().1 / 10.) as f64;
            if new_scale < 1f64 {
                new_scale = 1f64;
            }
            let after = screen_point_to_world(point, &singl.target_center, new_scale, &singl.offset.1);
            singl.target_scale = new_scale;
            singl.target_center.x += before.x - after.x;
            singl.target_center.y += before.y - after.y;
            singl.zoom_pending = true;
            singl.last_zoom_input = Instant::now();
            singl.last_input = Instant::now();
            log_event(
                "zoom",
                &format!(
                    "scale={} center=({}, {})",
                    singl.target_scale, singl.target_center.x, singl.target_center.y
                ),
            );
        }
    }

    if is_key_pressed(KeyCode::Enter) {
        singl.refresh = true;
        singl.last_refresh = Instant::now() - Duration::from_millis(singl.refresh_limit + 1);
        singl.last_input = Instant::now();
        log_event("refresh", "key=enter");
    }

    if is_key_pressed(KeyCode::Escape) {
        singl.egui = !singl.egui;
        singl.last_input = Instant::now();
        log_event("toggle_ui", &format!("enabled={}", singl.egui));
    }

    if is_key_pressed(KeyCode::Space) {
        singl.animation = !singl.animation;
        singl.last_input = Instant::now();
        log_event("toggle_animation", &format!("enabled={}", singl.animation));
    }

    if is_key_pressed(KeyCode::Tab) {
        singl.colorscheme += 1usize;
        if singl.colorscheme >= colorschemes::colorschemes().len() {
            singl.colorscheme = 0usize;
        }
        singl.generate_colors();
        singl.recolor = true;
        singl.last_input = Instant::now();
        let schemes = colorschemes::colorschemes();
        let name = schemes
            .get(singl.colorscheme)
            .map(|scheme| scheme.name)
            .unwrap_or("unknown");
        log_event("colorscheme", &format!("index={} name={}", singl.colorscheme, name));
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut singl = Singleton {
        power: 2.,
        egui: false,
        ..Default::default()
    };
    singl.generate_colors();
    singl.target_scale = singl.scale;
    singl.target_center = singl.center.clone();
    singl.refresh = true;
    singl.last_refresh = Instant::now() - Duration::from_millis(singl.refresh_limit + 1);

    let screen_w = screen_width() as usize;
    let screen_h = screen_height() as usize;
    let mut caches: Vec<RenderCache> = Vec::new();

    let (sender, receiver) = mpsc::channel::<RenderMessage>();
    let mut compute_in_flight = false;
    let mut render_id: u64 = 0;
    let active_render_id = Arc::new(AtomicU64::new(0));
    let mut inflight_render: Option<InflightRender> = None;
    let mut last_screen_w = screen_w;
    let mut last_screen_h = screen_h;
    let mut last_preview_start = Instant::now() - Duration::from_secs(10);

    loop {
        clear_background(LIGHTGRAY);

        let current_w = screen_width() as usize;
        let current_h = screen_height() as usize;
        if current_w != 0
            && current_h != 0
            && (current_w != last_screen_w || current_h != last_screen_h)
        {
            last_screen_w = current_w;
            last_screen_h = current_h;
            caches.clear();
            inflight_render = None;
            compute_in_flight = false;
            singl.refresh = true;
            singl.last_refresh = Instant::now() - Duration::from_millis(singl.refresh_limit + 1);
            log_event(
                "resize",
                &format!("size={}x{}", last_screen_w, last_screen_h),
            );
        }

        while let Ok(message) = receiver.try_recv() {
            match message {
                RenderMessage::Tile {
                    id,
                    index,
                    x,
                    y,
                    width,
                    height,
                    iters,
                } => {
                    if id != render_id {
                        continue;
                    }
                    log_event(
                        "tile_complete",
                        &format!("id={} tile={} size={}x{}", id, index, width, height),
                    );
                    if let Some(inflight) = inflight_render.as_mut() {
                        let full_width = inflight.cache.render_width as usize;
                        let full_height = inflight.cache.render_height as usize;
                        if x + width <= full_width && y + height <= full_height {
                            for row in 0..height {
                                let src_offset = row * width;
                                let dst_offset = (y + row) * full_width + x;
                                inflight.cache.iters[dst_offset..dst_offset + width]
                                    .copy_from_slice(&iters[src_offset..src_offset + width]);
                            }
                            if index < inflight.tile_done.len() {
                                inflight.tile_done[index] = true;
                            }
                            update_image_tile(&mut inflight.image, &iters, x, y, width, height, &singl.pallet);
                            inflight.pending_tiles += 1;
                            let update_due = inflight.pending_tiles >= inflight.texture_update_stride
                                || inflight.last_texture_update.elapsed().as_millis()
                                    >= inflight.texture_update_interval_ms as u128;
                            if update_due {
                                inflight.cache.texture = texture_from_image(&inflight.image);
                                inflight.last_texture_update = Instant::now();
                                inflight.pending_tiles = 0;
                            }
                        }
                    }
                }
                RenderMessage::Done { id } => {
                    if id != render_id {
                        continue;
                    }
                    log_event(
                        "render_complete",
                        &format!("id={} caches_before={}", id, caches.len()),
                    );
                    if let Some(mut inflight) = inflight_render.take() {
                        inflight.cache.complete = true;
                        inflight.cache.texture = texture_from_image(&inflight.image);
                        caches.retain(|existing| {
                            (existing.scale - inflight.cache.scale).abs() > f64::EPSILON
                                || (existing.center.x - inflight.cache.center.x).abs() > f64::EPSILON
                                || (existing.center.y - inflight.cache.center.y).abs() > f64::EPSILON
                                || (existing.power - inflight.cache.power).abs() > f64::EPSILON
                                || existing.max_iter != inflight.cache.max_iter
                                || (existing.render_scale - inflight.cache.render_scale).abs() > f32::EPSILON
                                || (existing.julia.x - inflight.cache.julia.x).abs() > f64::EPSILON
                                || (existing.julia.y - inflight.cache.julia.y).abs() > f64::EPSILON
                        });
                        caches.insert(0, inflight.cache);
                        if caches.len() > 6 {
                            caches.truncate(6);
                        }
                    }
                    log_event(
                        "render_complete",
                        &format!("id={} caches_after={}", id, caches.len()),
                    );
                    compute_in_flight = false;
                }
            }
        }

        if singl.recolor {
            for cache in caches.iter_mut() {
                if cache.complete
                    && cache.max_iter == singl.max_iter
                    && (cache.power - singl.power).abs() <= f64::EPSILON
                    && (cache.julia.x - singl.julia.x).abs() <= f64::EPSILON
                    && (cache.julia.y - singl.julia.y).abs() <= f64::EPSILON
                {
                    recolor_cache(cache, &singl.pallet);
                }
            }
            if let Some(inflight) = inflight_render.as_mut() {
                if inflight.cache.max_iter == singl.max_iter
                    && (inflight.cache.power - singl.power).abs() <= f64::EPSILON
                    && (inflight.cache.julia.x - singl.julia.x).abs() <= f64::EPSILON
                    && (inflight.cache.julia.y - singl.julia.y).abs() <= f64::EPSILON
                {
                    let width = inflight.cache.render_width as usize;
                    let height = inflight.cache.render_height as usize;
                    inflight.image = image_from_iters_with_mask(
                        &inflight.cache.iters,
                        width,
                        height,
                        &singl.pallet,
                        &inflight.tile_done,
                        inflight.tile_cols,
                        inflight.tile_rows,
                        inflight.tile_size,
                    );
                    inflight.cache.texture = texture_from_image(&inflight.image);
                    inflight.last_texture_update = Instant::now();
                    inflight.pending_tiles = 0;
                }
            }
            singl.recolor = false;
        }

        if singl.zoom_pending
            && singl.last_zoom_input.elapsed().as_millis() >= singl.zoom_cooldown_ms as u128
        {
            singl.scale = singl.target_scale;
            singl.center = singl.target_center.clone();
            singl.refresh = true;
            singl.last_refresh = Instant::now() - Duration::from_millis(singl.refresh_limit + 1);
            singl.zoom_pending = false;
        }

        apply_zoom_lerp(&mut singl);

        let input_idle = singl.last_input.elapsed().as_millis() >= singl.input_idle_ms as u128;
        let input_active = !input_idle;
        if input_active && compute_in_flight {
            active_render_id.store(render_id.wrapping_add(1), Ordering::SeqCst);
            compute_in_flight = false;
        }

        if input_active && singl.preview_while_interacting {
            let preview_due = last_preview_start.elapsed().as_millis()
                >= singl.preview_debounce_ms as u128;
            if preview_due && !compute_in_flight {
                singl.generate_colors();
                let screen_w = screen_width() as usize;
                let screen_h = screen_height() as usize;
                let render_singl = adjusted_render_state(&singl);
                log_event(
                    "preview_start",
                    &format!(
                        "id={} size={}x{} scale={} preview_scale={}",
                        render_id.wrapping_add(1),
                        screen_w,
                        screen_h,
                        render_singl.scale,
                        singl.preview_scale
                    ),
                );
                inflight_render = Some(start_render(
                    &render_singl,
                    screen_w,
                    screen_h,
                    singl.preview_scale,
                    &mut render_id,
                    &active_render_id,
                    &sender,
                ));
                compute_in_flight = true;
                last_preview_start = Instant::now();
            }
        }

        if singl.refresh
            && input_idle
            && singl.last_refresh.elapsed().as_millis() > singl.refresh_limit as u128
            && singl.last_input.elapsed().as_millis() >= singl.render_debounce_ms as u128
        {
            if !compute_in_flight {
                singl.generate_colors();
                let screen_w = screen_width() as usize;
                let screen_h = screen_height() as usize;
                let render_singl = adjusted_render_state(&singl);
                log_event(
                    "refresh_start",
                    &format!(
                        "id={} size={}x{} scale={} center=({}, {})",
                        render_id.wrapping_add(1),
                        screen_w,
                        screen_h,
                        render_singl.scale,
                        render_singl.center.x,
                        render_singl.center.y
                    ),
                );
                inflight_render = Some(start_render(
                    &render_singl,
                    screen_w,
                    screen_h,
                    1.0,
                    &mut render_id,
                    &active_render_id,
                    &sender,
                ));
                singl.refresh = false;
                compute_in_flight = true;
            }
        }

        if singl.animation {
            singl.power += singl.animation_unit;
        }

        if let Some(index) = select_cache_index(&caches, &singl) {
            draw_cached_texture(&caches[index], &singl);
        }
        if let Some(inflight) = inflight_render.as_ref() {
            draw_cached_texture(&inflight.cache, &singl);
        }

        if is_key_pressed(KeyCode::S) {
            save_snapshot(&singl);
        }
        if is_key_pressed(KeyCode::L) {
            if let Some(config) = load_latest_snapshot_config("captures") {
                apply_snapshot_config(&mut singl, config);
            }
        }

        user_input(&mut singl);
        draw_menus(&mut singl);

        window::next_frame().await
    }
}

fn window_conf() -> window::Conf {
    window::Conf {
        window_title: "GC Lab 2".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}
