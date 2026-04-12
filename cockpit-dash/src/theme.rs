use ratatui::style::Color;

// Catppuccin Mocha palette
pub const BASE: Color = Color::Rgb(30, 30, 46); // #1e1e2e
pub const SURFACE0: Color = Color::Rgb(49, 50, 68); // #313244
pub const SURFACE1: Color = Color::Rgb(69, 71, 90); // #45475a
pub const TEXT: Color = Color::Rgb(205, 214, 244); // #cdd6f4
pub const SUBTEXT0: Color = Color::Rgb(166, 173, 200); // #a6adc8
pub const OVERLAY0: Color = Color::Rgb(108, 112, 134); // #6c7086

pub const GREEN: Color = Color::Rgb(166, 227, 161); // #a6e3a1
pub const BLUE: Color = Color::Rgb(137, 180, 250); // #89b4fa
pub const RED: Color = Color::Rgb(243, 139, 168); // #f38ba8
pub const YELLOW: Color = Color::Rgb(249, 226, 175); // #f9e2af
pub const MAUVE: Color = Color::Rgb(203, 166, 247); // #cba6f7
pub const PEACH: Color = Color::Rgb(250, 179, 135); // #fab387

pub fn status_color(status: &str) -> Color {
    match status {
        "in_progress" => GREEN,
        "open" => BLUE,
        "blocked" => RED,
        "deferred" => YELLOW,
        "closed" | "done" => OVERLAY0,
        _ => TEXT,
    }
}

pub fn status_icon(status: &str) -> &'static str {
    match status {
        "in_progress" => "●",
        "open" => "○",
        "blocked" => "◌",
        "deferred" => "◊",
        "closed" | "done" => "✓",
        _ => "·",
    }
}
