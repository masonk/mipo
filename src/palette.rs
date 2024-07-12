use bevy::prelude::*;
pub enum Palette {
    Red,
    Yellow,
    Blue,
    HudBackground,
}
use Palette::*;

impl Palette {
    pub fn to_color(&self) -> Color {
        match self {
            Red => Color::srgba(255. / 255., 98. / 255., 81. / 255., 1.0),
            Yellow => Color::linear_rgba(252. / 255., 226. / 255., 8. / 255., 1.0),
            Blue => Color::srgba(8. / 255., 226. / 255., 252. / 255., 1.0),
            HudBackground => Color::srgba(1., 1., 1., 1.),
        }
    }
}

impl From<Palette> for Color {
    fn from(pal: Palette) -> Color {
        pal.to_color()
    }
}
