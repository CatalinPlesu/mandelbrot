use macroquad::color::Color;

#[derive(Clone, Debug)]
pub struct ColorScheme {
    pub name: &'static str,
    pub colors: Vec<Color>,
}

pub fn colorschemes() -> Vec<ColorScheme> {
    vec![
        ColorScheme {
            name: "Midnight Garden",
            colors: vec![
                Color::from_rgba(21, 29, 59, 255),
                Color::from_rgba(216, 33, 72, 255),
                Color::from_rgba(110, 191, 139, 255),
                Color::from_rgba(218, 219, 189, 255),
            ],
        },
        ColorScheme {
            name: "Lagoon Pulse",
            colors: vec![
                Color::from_rgba(1, 21, 38, 255),
                Color::from_rgba(136, 247, 226, 255),
                Color::from_rgba(68, 212, 146, 255),
                Color::from_rgba(255, 161, 92, 255),
                Color::from_rgba(245, 235, 103, 255),
                Color::from_rgba(250, 35, 62, 255),
                Color::from_rgba(217, 7, 84, 255),
            ],
        },
        ColorScheme {
            name: "Signal",
            colors: vec![
                Color::from_rgba(17, 20, 207, 255),
                Color::from_rgba(217, 35, 22, 255),
                Color::from_rgba(217, 35, 22, 255),
            ],
        },
        ColorScheme {
            name: "Blue Ember",
            colors: vec![
                Color::from_rgba(0, 7, 100, 255),
                Color::from_rgba(32, 107, 203, 255),
                Color::from_rgba(237, 255, 255, 255),
                Color::from_rgba(255, 170, 0, 255),
                Color::from_rgba(0, 2, 0, 255),
            ],
        },
        ColorScheme {
            name: "Orchard",
            colors: vec![
                Color::from_rgba(87, 51, 145, 255),
                Color::from_rgba(255, 230, 171, 255),
                Color::from_rgba(53, 124, 60, 255),
                Color::from_rgba(239, 109, 109, 255),
            ],
        },
        ColorScheme {
            name: "Neon Rose",
            colors: vec![
                Color::from_rgba(12, 30, 127, 255),
                Color::from_rgba(210, 39, 121, 255),
                Color::from_rgba(97, 40, 151, 255),
                Color::from_rgba(255, 0, 142, 255),
            ],
        },
        ColorScheme {
            name: "Aurora",
            colors: vec![
                Color::from_rgba(17, 5, 44, 255),
                Color::from_rgba(61, 8, 123, 255),
                Color::from_rgba(244, 59, 134, 255),
                Color::from_rgba(255, 228, 89, 255),
                Color::from_rgba(81, 45, 109, 255),
                Color::from_rgba(248, 72, 94, 255),
                Color::from_rgba(238, 238, 238, 255),
                Color::from_rgba(0, 193, 212, 255),
            ],
        },
        ColorScheme {
            name: "Gruvbox",
            colors: vec![
                Color::from_rgba(40, 40, 40, 255),
                Color::from_rgba(204, 36, 29, 255),
                Color::from_rgba(104, 157, 106, 255),
                Color::from_rgba(69, 133, 136, 255),
                Color::from_rgba(177, 98, 134, 255),
                Color::from_rgba(215, 153, 33, 255),
                Color::from_rgba(152, 151, 26, 255),
                Color::from_rgba(214, 93, 14, 255),
            ],
        },
    ]
}
