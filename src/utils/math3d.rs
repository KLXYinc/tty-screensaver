#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn rotate_x(self, angle: f64) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x,
            y: self.y * cos_a - self.z * sin_a,
            z: self.y * sin_a + self.z * cos_a,
        }
    }

    pub fn rotate_y(self, angle: f64) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x * cos_a + self.z * sin_a,
            y: self.y,
            z: -self.x * sin_a + self.z * cos_a,
        }
    }

    pub fn rotate_z(self, angle: f64) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x * cos_a - self.y * sin_a,
            y: self.x * sin_a + self.y * cos_a,
            z: self.z,
        }
    }

    pub fn project_to_2d(
        self,
        screen_width: f64,
        screen_height: f64,
        fov: f64,
        z_offset: f64,
    ) -> Option<(f64, f64)> {
        let adjusted_z = self.z + z_offset;

        if adjusted_z <= 0.1 {
            return None;
        }

        let f = fov / adjusted_z;

        let aspect_correction_y = 0.5;

        let min_dimension = screen_width.min(screen_height / aspect_correction_y);

        let screen_x = (self.x * f * min_dimension) + (screen_width * 0.5);
        let screen_y = (self.y * f * min_dimension * aspect_correction_y) + (screen_height * 0.5);

        Some((screen_x, screen_y))
    }
}
