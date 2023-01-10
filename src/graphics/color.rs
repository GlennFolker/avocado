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
    pub fn to_vertex(self) -> [u8; 4] {
        [
            (self.r * 255.) as u8,
            (self.g * 255.) as u8,
            (self.b * 255.) as u8,
            (self.a * 255.) as u8
        ]
    }

    pub fn to_hsv(self) -> (f32, f32, f32) {
        let (h, s, v);
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);

        let range = max - min;
        h = if range.abs() <= 0.000001 {
            0.
        } else if (max - self.r).abs() <= 0.000001 {
            (60. * (self.g - self.b) / range + 360.) % 360.
        } else if (max - self.g).abs() <= 0.000001 {
            60. * (self.b - self.r) / range + 120.
        } else {
            60. * (self.r - self.g) / range + 240.
        };

        s = if max > 0. {
            1. - min / max
        } else {
            0.
        };

        v = max;
        (h, s, v)
    }

    #[inline]
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let x = (h / 60. + 6.) % 6.;
        let i = x as u8;

        let f = x - (i as f32);
        let p = v * (1. - s);
        let q = v * (1. - s * f);
        let t = v * (1. - s * (1. - f));

        let (r, g, b) = match i {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            5 => (v, p, q),
            _ => unreachable!(),
        };

        Self::rgb(r, g, b)
    }

    #[inline]
    pub fn shift_hue(self, shift: f32) -> Self {
        let (mut h, s, v) = self.to_hsv();
        h += shift;

        Self::from_hsv(h, s, v)
    }
}
