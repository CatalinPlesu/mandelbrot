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
            name: "Inferno",
            colors: vec![
                Color::from_rgba(0, 0, 4, 255),
                Color::from_rgba(27, 12, 65, 255),
                Color::from_rgba(74, 12, 107, 255),
                Color::from_rgba(120, 28, 109, 255),
                Color::from_rgba(165, 44, 96, 255),
                Color::from_rgba(207, 68, 70, 255),
                Color::from_rgba(237, 105, 37, 255),
                Color::from_rgba(251, 155, 6, 255),
                Color::from_rgba(247, 209, 61, 255),
                Color::from_rgba(252, 255, 164, 255),
            ],
        },
        ColorScheme {
            name: "Redis",
            colors: vec![
                Color::from_rgba(22, 22, 22, 255),
                Color::from_rgba(64, 64, 64, 255),
                Color::from_rgba(151, 44, 31, 255),
                Color::from_rgba(213, 45, 33, 255),
                Color::from_rgba(240, 79, 61, 255),
                Color::from_rgba(248, 178, 165, 255),
                Color::from_rgba(252, 236, 232, 255),
            ],
        },
    ]
}
