use glium::{glutin::surface::WindowSurface, Display, Vertex, VertexBuffer};

pub struct DynVB<V: Copy + Vertex> {
    buffer: VertexBuffer<V>,
    length: usize,
}

impl<V: Copy + Vertex> DynVB<V> {
    pub fn new(
        display: &Display<WindowSurface>,
    ) -> Result<Self, glium::vertex::BufferCreationError> {
        Ok(Self {
            buffer: VertexBuffer::empty_dynamic(display, 0)?,
            length: 0,
        })
    }
}
