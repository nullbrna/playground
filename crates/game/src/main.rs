use sola_raylib::color::Color;
use sola_raylib::consts::TraceLogLevel;
use sola_raylib::drawing::RaylibDraw;

fn main() {
    let (mut ray, ray_thread) = sola_raylib::init()
        .size(640, 480)
        .title("Hello, World")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    ray.set_target_fps(60);

    while !ray.window_should_close() {
        let mut draw = ray.begin_drawing(&ray_thread);
        draw.draw_fps(550, 440);

        draw.clear_background(Color::BLACK);
        draw.draw_text("Hello, world!", 20, 20, 20, Color::WHITE);
    }
}
