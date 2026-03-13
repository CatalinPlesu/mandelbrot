use macroquad::color::colors::*;
use macroquad::color::Color;
use macroquad::input::*;
use macroquad::prelude::vec2;
use macroquad::shapes::*;
use macroquad::texture::*;
use macroquad::window;
use macroquad::window::*;

use egui;
use num;
// use rand::Rng;
use std::sync::{mpsc, Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

mod colorschemes;

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
    max_iter: usize,
    colorscheme: usize,
    pallet: Vec<Color>,
    center: Point<f64>,
    julia: Point<f64>,
    offset: (Point<f32>, Point<f32>),
    refresh: bool,
    last_refresh: Instant,
    refresh_limit: u64,
    mouse_click: bool,
    egui: bool,
    animation: bool,
    animation_unit: f64,
    threads: usize,
    bands: usize,
    recolor: bool,
}

#[derive(Clone, Debug)]
struct RenderCache {
    center: Point<f64>,
    scale: f64,
    power: f64,
    max_iter: usize,
    bands: usize,
    julia: Point<f64>,
    screen_width: f32,
    screen_height: f32,
    textures: Vec<Texture2D>,
    iter_bands: Vec<Vec<u16>>,
}

enum RenderMessage {
    Band { id: u64, index: usize, iters: Vec<u16> },
    Done { id: u64 },
}

impl Default for Singleton {
    fn default() -> Singleton {
        Singleton {
            power: 2.,
            scale: 1.,
            max_iter: 55,
            colorscheme: 0,
            pallet: Vec::new(),
            center: Point { x: 0., y: 0. },
            julia: Point { x: 0., y: 0. },
            offset: (Point { x: 0., y: 0. }, Point { x: 0., y: 0. }),
            refresh: false,
            last_refresh: Instant::now(),
            refresh_limit: 100,
            mouse_click: false,
            egui: true,
            animation: false,
            animation_unit: 0.01,
            threads: 8,
            bands: 32,
            recolor: false,
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
                        + (colorscheme.colors[i + 1].b - colorscheme.colors[i].b)
                            * (j as f32 / color as f32),
                ));
            }
        }
        while self.pallet.len() <= self.max_iter {
            self.pallet.push(BLACK);
        }
    }
}

fn mandelbrot(c: num::complex::Complex<f64>, singl: &Singleton) -> usize {
    let mut z = num::complex::Complex::<f64>::new(singl.julia.x, singl.julia.y);
    let mut i: usize = 0;
    while i < singl.max_iter && z.l1_norm() <= 4f64 {
        z = z.powf(singl.power) + c;
        i += 1;
    }
    return i;
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

fn band_height_for(singl: &Singleton, screen_height: usize) -> usize {
    let mut band_height = screen_height / singl.bands;
    band_height += band_height * singl.bands / 10;
    band_height
}

fn band_height_for_dims(bands: usize, screen_height: usize) -> usize {
    let mut band_height = screen_height / bands;
    band_height += band_height * bands / 10;
    band_height
}

fn empty_band_images(singl: &Singleton, screen_width: usize, screen_height: usize) -> Vec<Image> {
    let mut bands = Vec::new();
    for i in 0..singl.bands {
        bands.push(i);
    }

    let mut images = Vec::new();
    let band_height = band_height_for(singl, screen_height);
    for _ in 0..singl.bands {
        images.push(Image::gen_image_color(
            screen_width as u16,
            band_height as u16,
            Color::new(0., 0., 0., 0.),
        ));
    }
    images
}

fn empty_iter_bands(singl: &Singleton, screen_width: usize, screen_height: usize) -> Vec<Vec<u16>> {
    let band_height = band_height_for(singl, screen_height);
    let mut bands = Vec::new();
    for _ in 0..singl.bands {
        bands.push(vec![0u16; screen_width * band_height]);
    }
    bands
}

fn image_from_iters(iters: &[u16], width: usize, height: usize, pallet: &[Color]) -> Image {
    let mut image = Image::gen_image_color(width as u16, height as u16, WHITE);
    let max_index = pallet.len().saturating_sub(1);
    for x in 0..width as u32 {
        for y in 0..height as u32 {
            let index = (y as usize) * width + x as usize;
            let iter = iters[index] as usize;
            let color = pallet[iter.min(max_index)];
            image.set_pixel(x, y, color);
        }
    }
    image
}

fn fractal_iter_bands(singl: &Singleton, screen_width: usize, screen_height: usize) -> Vec<Vec<u16>> {
    let mut bands = Vec::new();
    for i in 0..singl.bands {
        bands.push(i);
    }

    let bands_mutex = Arc::new(Mutex::new(bands));
    let iter_mutex = Arc::new(Mutex::new(empty_iter_bands(singl, screen_width, screen_height)));
    let singl_mutex = Arc::new(singl.clone());

    let mut handles = Vec::new();
    for _ in 0..singl.threads {
        let singl_clone = Arc::clone(&singl_mutex);
        let bands_clone = Arc::clone(&bands_mutex);
        let iter_clone = Arc::clone(&iter_mutex);
        let handle = thread::spawn(move || {
            let local_singl = singl_clone;
            loop {
                let mut bands = bands_clone.lock().unwrap();
                if bands.len() == 0 {
                    break;
                }
                let index = bands.remove(0);
                drop(bands);

                let band_height = band_height_for(&local_singl, screen_height);
                let mut iters = vec![0u16; screen_width * band_height];

                for x in 0..screen_width as u32 {
                    for y in 0..band_height as u32 {
                        let point = Point::<f64> {
                            x: x as f64,
                            y: (index * band_height) as f64 + y as f64,
                        }
                        .to_world_with_dims(&local_singl, screen_width as f64, screen_height as f64);
                        let c = num::complex::Complex::<f64>::new(point.x, point.y);

                        let iter = mandelbrot(c, &local_singl);
                        let offset = (y as usize) * screen_width + x as usize;
                        iters[offset] = iter.min(u16::MAX as usize) as u16;
                    }
                }

                let mut iter_bands = iter_clone.lock().unwrap();
                iter_bands[index] = iters;
                drop(iter_bands);
                thread::sleep(Duration::from_millis(1));
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let iter_clone = Arc::clone(&iter_mutex);
    let iter_bands = iter_clone.lock().unwrap();
    let mut result = Vec::new();
    for i in 0..singl.bands {
        result.push(iter_bands[i].clone());
    }
    drop(iter_bands);
    result
}

fn textures_from_images(images: Vec<Image>) -> Vec<Texture2D> {
    let mut textures = Vec::new();
    for image in images {
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Linear);
        textures.push(texture);
    }
    textures
}

fn images_from_iter_bands(
    iter_bands: &[Vec<u16>],
    width: usize,
    height: usize,
    pallet: &[Color],
) -> Vec<Image> {
    let band_height = band_height_for_dims(iter_bands.len(), height);
    let mut images = Vec::new();
    for band in iter_bands {
        images.push(image_from_iters(band, width, band_height, pallet));
    }
    images
}

fn update_cache_textures_from_iters(cache: &mut RenderCache, pallet: &[Color]) {
    if cache.iter_bands.is_empty() {
        return;
    }
    let width = cache.screen_width as usize;
    let height = cache.screen_height as usize;
    let band_height = band_height_for_dims(cache.iter_bands.len(), height);
    for (index, band) in cache.iter_bands.iter().enumerate() {
        if index >= cache.textures.len() {
            continue;
        }
        if band.len() != width * band_height {
            continue;
        }
        let image = image_from_iters(band, width, band_height, pallet);
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Linear);
        cache.textures[index] = texture;
    }
}

fn draw_cached_textures(cache: &RenderCache, singl: &Singleton) {
    if cache.textures.is_empty() {
        return;
    }

    let screen_w = screen_width();
    let screen_h = screen_height();
    let unit_old = map_screen_to_world_with_dims_scale(
        cache.scale,
        cache.screen_width as f64,
        cache.screen_height as f64,
    );
    let unit_new = map_screen_to_world_with_dims_scale(singl.scale, screen_w as f64, screen_h as f64);
    let scale = (unit_old / unit_new) as f32;

    let offset_x = screen_w / 2.0
        - (cache.screen_width * scale) / 2.0
        + ((cache.center.x - singl.center.x) / unit_new) as f32
        + singl.offset.1.x;
    let offset_y = screen_h / 2.0
        - (cache.screen_height * scale) / 2.0
        + ((singl.center.y - cache.center.y) / unit_new) as f32
        + singl.offset.1.y;

    for i in 0..cache.textures.len() {
        let texture = &cache.textures[i];
        let dest_size = vec2(texture.width() * scale, texture.height() * scale);
        draw_texture_ex(
            texture,
            offset_x,
            offset_y + i as f32 * texture.height() * scale,
            WHITE,
            DrawTextureParams {
                dest_size: Some(dest_size),
                ..Default::default()
            },
        );
    }
}

fn start_fractal_job(
    singl: &Singleton,
    screen_width: usize,
    screen_height: usize,
    render_id: u64,
    sender: mpsc::Sender<RenderMessage>,
) {
    let singl_clone = singl.clone();
    thread::spawn(move || {
        let mut bands = Vec::new();
        for i in 0..singl_clone.bands {
            bands.push(i);
        }
        let bands_mutex = Arc::new(Mutex::new(bands));
        let singl_mutex = Arc::new(singl_clone);
        let completed = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..singl_mutex.threads {
            let singl_local = Arc::clone(&singl_mutex);
            let bands_clone = Arc::clone(&bands_mutex);
            let sender_clone = sender.clone();
            let completed_clone = Arc::clone(&completed);

            let handle = thread::spawn(move || loop {
                let mut bands = bands_clone.lock().unwrap();
                if bands.is_empty() {
                    break;
                }
                let index = bands.remove(0);
                drop(bands);

                let band_height = band_height_for(&singl_local, screen_height);
                let mut iters = vec![0u16; screen_width * band_height];

                for x in 0..screen_width as u32 {
                    for y in 0..band_height as u32 {
                        let point = Point::<f64> {
                            x: x as f64,
                            y: (index * band_height) as f64 + y as f64,
                        }
                        .to_world_with_dims(
                            &singl_local,
                            screen_width as f64,
                            screen_height as f64,
                        );
                        let c = num::complex::Complex::<f64>::new(point.x, point.y);

                        let iter = mandelbrot(c, &singl_local);
                        let offset = (y as usize) * screen_width + x as usize;
                        iters[offset] = iter.min(u16::MAX as usize) as u16;
                    }
                }

                let _ = sender_clone.send(RenderMessage::Band {
                    id: render_id,
                    index,
                    iters,
                });

                let finished = completed_clone.fetch_add(1, Ordering::SeqCst) + 1;
                if finished == singl_local.bands {
                    let _ = sender_clone.send(RenderMessage::Done { id: render_id });
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
            || cache.bands != singl.bands
            || (cache.power - singl.power).abs() > f64::EPSILON
            || (cache.julia.x - singl.julia.x).abs() > f64::EPSILON
            || (cache.julia.y - singl.julia.y).abs() > f64::EPSILON
        {
            continue;
        }
        let unit_old = map_screen_to_world_with_dims_scale(
            cache.scale,
            cache.screen_width as f64,
            cache.screen_height as f64,
        );
        let scale_score = (unit_old / unit_new).ln().abs();
        let dx = (cache.center.x - singl.center.x).abs() / unit_new;
        let dy = (cache.center.y - singl.center.y).abs() / unit_new;
        let score = scale_score * 2.0 + (dx + dy) / 1000.0;
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
                    .add(egui::Slider::new(&mut singl.scale, 1f64..=1_000_000f64).text("Zoom"))
                    .changed();
                needs_refresh |= ui
                    .add(egui::Slider::new(&mut singl.max_iter, 0..=1_000).text("Max iterations"))
                    .changed();
                needs_refresh |= ui
                    .add(egui::Slider::new(&mut singl.power, 0.0..=100.0).text("Power"))
                    .changed();
                needs_refresh |= ui
                    .add(egui::Slider::new(&mut singl.threads, 1..=16).text("Threads"))
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

                if needs_refresh {
                    singl.refresh = true;
                    singl.last_refresh = Instant::now();
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
                }
                if ui.button("Animation on/off").clicked() {
                    singl.animation = !singl.animation;
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
                }
                if ui.button("Center").clicked() {
                    singl.center = Point::<f64> { x: 0., y: 0. };
                    singl.offset = (Point { x: 0., y: 0. }, Point { x: 0., y: 0. });
                    singl.mouse_click = false;
                    singl.refresh = true;
                    singl.last_refresh =
                        Instant::now() - Duration::from_millis(singl.refresh_limit + 1);
                }
            });
        });
        egui_macroquad::draw();
    }
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
        }

        if is_mouse_button_down(MouseButton::Left) && singl.mouse_click {
            let mouse = mouse_position();
            singl.offset.1.x = mouse.0 - singl.offset.0.x;
            singl.offset.1.y = mouse.1 - singl.offset.0.y;
        }

        if is_mouse_button_released(MouseButton::Left) && singl.mouse_click {
            let unit = map_screen_to_world(singl);
            singl.center.x -= singl.offset.1.x as f64 * unit;
            singl.center.y += singl.offset.1.y as f64 * unit;
            singl.offset = (Point { x: 0., y: 0. }, Point { x: 0., y: 0. });
            singl.mouse_click = false;
            singl.refresh = true;
            singl.last_refresh = Instant::now();
        }

        if mouse_wheel().1 != 0. {
            let mouse = mouse_position();
            let before = Point::<f64> {
                x: mouse.0 as f64,
                y: mouse.1 as f64,
            }
            .to_world(&singl);
            singl.scale += singl.scale * (mouse_wheel().1 / 10.) as f64;
            if singl.scale < 1f64 {
                singl.scale = 1f64;
            }
            let after = Point::<f64> {
                x: mouse.0 as f64,
                y: mouse.1 as f64,
            }
            .to_world(&singl);
            singl.center.x += before.x - after.x;
            singl.center.y += before.y - after.y;
            singl.refresh = true;
            singl.last_refresh = Instant::now();
        }
    }

    if is_key_pressed(KeyCode::Enter) {
        singl.refresh = true;
        singl.last_refresh = Instant::now() - Duration::from_millis(singl.refresh_limit + 1);
    }

    if is_key_pressed(KeyCode::Escape) {
        singl.egui = !singl.egui;
    }

    if is_key_pressed(KeyCode::Space) {
        singl.animation = !singl.animation;
    }

    if is_key_pressed(KeyCode::Tab) {
        singl.colorscheme += 1usize;
        if singl.colorscheme >= colorschemes::colorschemes().len() {
            singl.colorscheme = 0usize;
        }
        singl.generate_colors();
        singl.recolor = true;
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

    let screen_w = screen_width() as usize;
    let screen_h = screen_height() as usize;
    let iter_bands = fractal_iter_bands(&singl, screen_w, screen_h);
    let images = images_from_iter_bands(&iter_bands, screen_w, screen_h, &singl.pallet);
    let textures = textures_from_images(images);
    let mut caches = vec![RenderCache {
        center: singl.center.clone(),
        scale: singl.scale,
        power: singl.power,
        max_iter: singl.max_iter,
        bands: singl.bands,
        julia: singl.julia.clone(),
        screen_width: screen_w as f32,
        screen_height: screen_h as f32,
        textures,
        iter_bands,
    }];

    let (sender, receiver) = mpsc::channel::<RenderMessage>();
    let mut compute_in_flight = false;
    let mut render_id: u64 = 0;
    let mut inflight_cache: Option<RenderCache> = None;

    loop {
        clear_background(LIGHTGRAY);

        while let Ok(message) = receiver.try_recv() {
            match message {
                RenderMessage::Band { id, index, iters } => {
                    if id != render_id {
                        continue;
                    }
                    if let Some(cache) = inflight_cache.as_mut() {
                        if index < cache.iter_bands.len() {
                            cache.iter_bands[index] = iters;
                        }
                        if index < cache.textures.len() {
                            let width = cache.screen_width as usize;
                            let height = cache.screen_height as usize;
                            let band_height = band_height_for_dims(cache.iter_bands.len(), height);
                            if cache.iter_bands[index].len() == width * band_height {
                                let image = image_from_iters(
                                    &cache.iter_bands[index],
                                    width,
                                    band_height,
                                    &singl.pallet,
                                );
                                let texture = Texture2D::from_image(&image);
                                texture.set_filter(FilterMode::Linear);
                                cache.textures[index] = texture;
                            }
                        }
                    }
                }
                RenderMessage::Done { id } => {
                    if id != render_id {
                        continue;
                    }
                    if let Some(cache) = inflight_cache.take() {
                        caches.retain(|existing| {
                            (existing.scale - cache.scale).abs() > f64::EPSILON
                                || (existing.center.x - cache.center.x).abs() > f64::EPSILON
                                || (existing.center.y - cache.center.y).abs() > f64::EPSILON
                                || (existing.power - cache.power).abs() > f64::EPSILON
                                || existing.max_iter != cache.max_iter
                                || existing.bands != cache.bands
                                || (existing.julia.x - cache.julia.x).abs() > f64::EPSILON
                                || (existing.julia.y - cache.julia.y).abs() > f64::EPSILON
                        });
                        caches.insert(0, cache);
                        if caches.len() > 6 {
                            caches.truncate(6);
                        }
                    }
                    compute_in_flight = false;
                }
            }
        }

        if singl.recolor {
            for cache in caches.iter_mut() {
                if cache.max_iter == singl.max_iter
                    && cache.bands == singl.bands
                    && (cache.power - singl.power).abs() <= f64::EPSILON
                    && (cache.julia.x - singl.julia.x).abs() <= f64::EPSILON
                    && (cache.julia.y - singl.julia.y).abs() <= f64::EPSILON
                {
                    update_cache_textures_from_iters(cache, &singl.pallet);
                }
            }
            if let Some(cache) = inflight_cache.as_mut() {
                if cache.max_iter == singl.max_iter
                    && cache.bands == singl.bands
                    && (cache.power - singl.power).abs() <= f64::EPSILON
                    && (cache.julia.x - singl.julia.x).abs() <= f64::EPSILON
                    && (cache.julia.y - singl.julia.y).abs() <= f64::EPSILON
                {
                    update_cache_textures_from_iters(cache, &singl.pallet);
                }
            }
            singl.recolor = false;
        }

        if singl.refresh && singl.last_refresh.elapsed().as_millis() > singl.refresh_limit as u128 {
            if !compute_in_flight {
                singl.generate_colors();
                let screen_w = screen_width() as usize;
                let screen_h = screen_height() as usize;
                render_id = render_id.wrapping_add(1);
                inflight_cache = Some(RenderCache {
                    center: singl.center.clone(),
                    scale: singl.scale,
                    power: singl.power,
                    max_iter: singl.max_iter,
                    bands: singl.bands,
                    julia: singl.julia.clone(),
                    screen_width: screen_w as f32,
                    screen_height: screen_h as f32,
                    textures: textures_from_images(empty_band_images(&singl, screen_w, screen_h)),
                    iter_bands: empty_iter_bands(&singl, screen_w, screen_h),
                });
                start_fractal_job(&singl, screen_w, screen_h, render_id, sender.clone());
                singl.refresh = false;
                compute_in_flight = true;
            }
        }

        if singl.animation {
            singl.power += singl.animation_unit;
        }

        if let Some(index) = select_cache_index(&caches, &singl) {
            draw_cached_textures(&caches[index], &singl);
        }
        if let Some(cache) = inflight_cache.as_ref() {
            draw_cached_textures(cache, &singl);
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
