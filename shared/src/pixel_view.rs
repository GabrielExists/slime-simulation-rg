pub struct PixelView<'storage> {
    storage_one: &'storage mut u32,
    storage_two: &'storage mut u32,
}

impl<'storage> PixelView<'storage> {
    pub fn new(storage_one: &'storage mut u32, storage_two: &'storage mut u32) -> Self {
        PixelView {
            storage_one,
            storage_two,
        }
    }
    pub fn x(&self) -> u32 { *self.storage_one >> 0 & 0xFFFF }
    pub fn y(&self) -> u32 { *self.storage_one >> 16 & 0xFFFF }
    pub fn z(&self) -> u32 { *self.storage_two >> 0 & 0xFFFF }
    pub fn w(&self) -> u32 { *self.storage_two >> 16 & 0xFFFF }
    pub fn get(&self, index: usize) -> u32 {
        match index {
            0 => self.x(),
            1 => self.y(),
            2 => self.z(),
            _ => self.w(),
        }
    }
    pub fn x_frac(&self) -> f32 { frac_from_int(self.x()) }
    pub fn y_frac(&self) -> f32 { frac_from_int(self.y()) }
    pub fn z_frac(&self) -> f32 { frac_from_int(self.z()) }
    pub fn w_frac(&self) -> f32 { frac_from_int(self.w()) }
    pub fn get_frac(&self, index: usize) -> f32 {
        frac_from_int(self.get(index))
    }
    pub fn set_x(&mut self, value: u32) {
        *self.storage_one = *self.storage_one & 0xFFFF0000 | (value & 0xFFFF) << 0;
    }
    pub fn set_y(&mut self, value: u32) {
        *self.storage_one = *self.storage_one & 0x0000FFFF | ((value & 0xFFFF) << 16);
    }
    pub fn set_z(&mut self, value: u32) {
        *self.storage_two = *self.storage_two & 0xFFFF0000 | (value & 0xFFFF) << 0;
    }
    pub fn set_w(&mut self, value: u32) {
        *self.storage_two = *self.storage_two & 0x0000FFFF | ((value & 0xFFFF) << 16);
    }
    pub fn set(&mut self, index: usize, value: u32) {
        match index {
            0 => self.set_x(value),
            1 => self.set_y(value),
            2 => self.set_z(value),
            _ => self.set_w(value),
        }
    }
    pub fn set_x_frac(&mut self, value: f32) {
        self.set_x(int_from_frac(value))
    }
    pub fn set_y_frac(&mut self, value: f32) {
        self.set_y(int_from_frac(value))
    }
    pub fn set_z_frac(&mut self, value: f32) {
        self.set_z(int_from_frac(value))
    }
    pub fn set_w_frac(&mut self, value: f32) {
        self.set_w(int_from_frac(value))
    }
    pub fn set_frac(&mut self, index: usize, value: f32) {
        match index {
            0 => self.set_x_frac(value),
            1 => self.set_y_frac(value),
            2 => self.set_z_frac(value),
            _ => self.set_w_frac(value),
        }
    }
}



pub const PIXEL_MAX: u32 = 2u32.pow(15) - 1;

pub fn frac_from_int(value: u32) -> f32 {
    value as f32 / PIXEL_MAX as f32
}

pub fn int_from_frac(value: f32) -> u32 {
    if value > 0.999 {
        PIXEL_MAX
    } else {
        (value * PIXEL_MAX as f32) as u32
    }
}

#[cfg(test)]
mod test {
    use crate::pixel_view::*;

    #[test]
    fn test_set() {
        let mut a = 0x12345678;
        let mut b = 0x98765432;
        let mut pixel = PixelView::new(&mut a, &mut b);
        pixel.set_x(0x2BCD);
        pixel.set_y(0x4DEF);
        pixel.set_z(0xFEDC);
        pixel.set_w(0xBA87);
        assert_eq!(pixel.x(), 0x2BCD);
        assert_eq!(pixel.y(), 0x4DEF);
        assert_eq!(pixel.z(), 0xFEDC);
        assert_eq!(pixel.w(), 0xBA87);
    }

    #[test]
    fn test_set_frac() {
        let mut a = 0x12345678;
        let mut b = 0x98765432;
        let mut pixel = PixelView::new(&mut a, &mut b);
        pixel.set_x_frac(0.5);
        pixel.set_y_frac(0.25);
        pixel.set_z_frac(0.75);
        pixel.set_w_frac(0.125);
        assert!(f32::abs(pixel.x_frac() - 0.5) < 0.01);
        assert!(f32::abs(pixel.y_frac() - 0.25) < 0.01);
        assert!(f32::abs(pixel.z_frac() - 0.75) < 0.01);
        assert!(f32::abs(pixel.w_frac() - 0.125) < 0.01);
    }
}
