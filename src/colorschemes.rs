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
        ColorScheme {
            name: "Viridis",
            colors: vec![
                Color::from_rgba(68, 1, 84, 255),
                Color::from_rgba(72, 40, 120, 255),
                Color::from_rgba(62, 73, 137, 255),
                Color::from_rgba(49, 104, 142, 255),
                Color::from_rgba(38, 131, 143, 255),
                Color::from_rgba(31, 157, 138, 255),
                Color::from_rgba(108, 206, 89, 255),
                Color::from_rgba(182, 222, 43, 255),
                Color::from_rgba(253, 231, 37, 255),
            ],
        },
        ColorScheme {
            name: "Plasma",
            colors: vec![
                Color::from_rgba(13, 8, 135, 255),
                Color::from_rgba(62, 4, 156, 255),
                Color::from_rgba(99, 0, 167, 255),
                Color::from_rgba(135, 7, 166, 255),
                Color::from_rgba(166, 32, 152, 255),
                Color::from_rgba(192, 58, 131, 255),
                Color::from_rgba(213, 84, 110, 255),
                Color::from_rgba(231, 111, 90, 255),
                Color::from_rgba(245, 140, 70, 255),
                Color::from_rgba(249, 174, 58, 255),
                Color::from_rgba(252, 210, 50, 255),
                Color::from_rgba(240, 249, 33, 255),
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
            name: "Magma",
            colors: vec![
                Color::from_rgba(0, 0, 4, 255),
                Color::from_rgba(28, 16, 68, 255),
                Color::from_rgba(79, 18, 123, 255),
                Color::from_rgba(129, 37, 129, 255),
                Color::from_rgba(181, 54, 122, 255),
                Color::from_rgba(229, 80, 99, 255),
                Color::from_rgba(251, 135, 97, 255),
                Color::from_rgba(254, 194, 135, 255),
                Color::from_rgba(253, 228, 181, 255),
                Color::from_rgba(252, 253, 191, 255),
            ],
        },
        ColorScheme {
            name: "Turbo",
            colors: vec![
                Color::from_rgba(48, 18, 59, 255),
                Color::from_rgba(65, 64, 166, 255),
                Color::from_rgba(71, 115, 235, 255),
                Color::from_rgba(62, 160, 255, 255),
                Color::from_rgba(44, 193, 232, 255),
                Color::from_rgba(31, 211, 170, 255),
                Color::from_rgba(64, 212, 126, 255),
                Color::from_rgba(127, 211, 78, 255),
                Color::from_rgba(182, 196, 67, 255),
                Color::from_rgba(230, 173, 63, 255),
                Color::from_rgba(246, 134, 47, 255),
                Color::from_rgba(239, 90, 42, 255),
                Color::from_rgba(214, 66, 45, 255),
                Color::from_rgba(177, 42, 59, 255),
                Color::from_rgba(122, 31, 92, 255),
            ],
        },
        ColorScheme {
            name: "Cividis",
            colors: vec![
                Color::from_rgba(0, 32, 76, 255),
                Color::from_rgba(0, 48, 111, 255),
                Color::from_rgba(55, 67, 143, 255),
                Color::from_rgba(87, 89, 166, 255),
                Color::from_rgba(111, 111, 167, 255),
                Color::from_rgba(135, 135, 159, 255),
                Color::from_rgba(159, 159, 146, 255),
                Color::from_rgba(184, 184, 130, 255),
                Color::from_rgba(210, 210, 111, 255),
                Color::from_rgba(236, 236, 90, 255),
            ],
        },
    ]
}
