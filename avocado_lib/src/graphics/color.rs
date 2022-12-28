#[derive(Default, Copy, Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    #[inline]
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1., }.clamp()
    }

    #[inline]
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a, }.clamp()
    }

    #[inline]
    pub fn clamp(mut self) -> Self {
        self.r = self.r.clamp(0., 1.);
        self.g = self.g.clamp(0., 1.);
        self.b = self.b.clamp(0., 1.);
        self.a = self.a.clamp(0., 1.);
        self
    }

    #[inline]
    pub fn to_vertex(self) -> [f32; 4] {
        [ self.r, self.g, self.b, self.a, ]
    }

    #[inline]
    pub fn to_vertex_u8(self) -> [u8; 4] {
        [
            (self.r * 255.) as u8,
            (self.g * 255.) as u8,
            (self.b * 255.) as u8,
            (self.a * 255.) as u8
        ]
    }
}
