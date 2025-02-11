use glam::Vec2;
use uiua::Array;

pub struct Boxed(pub Value);

pub enum Value {
    Point(Array<Vec2>),
    Number(Array<f64>),
    Char(Array<char>),
    Box(Array<Boxed>),
}
