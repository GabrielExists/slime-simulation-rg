pub struct PixelView<'storage> {
    storage_one: &'storage mut u32,
    storage_two: &'storage mut u32,
    storage_three: &'storage mut u32,
    storage_four: &'storage mut u32,
}

impl<'storage> PixelView<'storage> {
    pub fn new(
        storage_one: &'storage mut u32,
        storage_two: &'storage mut u32,
        storage_three: &'storage mut u32,
        storage_four: &'storage mut u32,
    ) -> Self {
        PixelView {
            storage_one,
            storage_two,
            storage_three,
            storage_four,
        }
    }
    pub fn get(&self, index: usize) -> u32 {
        match index {
            0 => (*self.storage_one >> 0) & 0xFFFF,
            1 => (*self.storage_one >> 16) & 0xFFFF,
            2 => (*self.storage_two >> 0) & 0xFFFF,
            3 => (*self.storage_two >> 16) & 0xFFFF,
            4 => (*self.storage_three >> 0) & 0xFFFF,
            5 => (*self.storage_three >> 16) & 0xFFFF,
            6 => (*self.storage_four >> 0) & 0xFFFF,
            _ => (*self.storage_four >> 16) & 0xFFFF,
        }
    }
    pub fn get_frac(&self, index: usize) -> f32 {
        frac_from_int(self.get(index))
    }
    pub fn set(&mut self, index: usize, value: u32) {
        match index {
            0 => *self.storage_one = *self.storage_one & 0xFFFF0000 | (value & 0xFFFF) << 0,
            1 => *self.storage_one = *self.storage_one & 0x0000FFFF | ((value & 0xFFFF) << 16),
            2 => *self.storage_two = *self.storage_two & 0xFFFF0000 | (value & 0xFFFF) << 0,
            3 => *self.storage_two = *self.storage_two & 0x0000FFFF | ((value & 0xFFFF) << 16),
            4 => *self.storage_three = *self.storage_three & 0xFFFF0000 | (value & 0xFFFF) << 0,
            5 => *self.storage_three = *self.storage_three & 0x0000FFFF | ((value & 0xFFFF) << 16),
            6 => *self.storage_four = *self.storage_four & 0xFFFF0000 | (value & 0xFFFF) << 0,
            _ => *self.storage_four = *self.storage_four & 0x0000FFFF | ((value & 0xFFFF) << 16),
        }
    }
    pub fn set_frac(&mut self, index: usize, value: f32) {
        self.set(index, int_from_frac(value));
    }
}

pub const NUM_CHANNELS: u32 = 8;
pub const INTS_PER_PIXEL: u32 = NUM_CHANNELS.div_ceil(2);
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
        let mut c = 0x12345678;
        let mut d = 0x98765432;
        let mut pixel = PixelView::new([&mut a, &mut b, &mut c, &mut d]);
        pixel.set(0, 0x2BCD);
        pixel.set(1, 0x4DEF);
        pixel.set(2, 0xFEDC);
        pixel.set(3, 0xBA87);
        pixel.set(4, 0x2BCD);
        pixel.set(5, 0x4DEF);
        pixel.set(6, 0xFEDC);
        pixel.set(7, 0xBA87);
        assert_eq!(pixel.get(0), 0x2BCD);
        assert_eq!(pixel.get(1), 0x4DEF);
        assert_eq!(pixel.get(2), 0xFEDC);
        assert_eq!(pixel.get(3), 0xBA87);
        assert_eq!(pixel.get(4), 0x2BCD);
        assert_eq!(pixel.get(5), 0x4DEF);
        assert_eq!(pixel.get(6), 0xFEDC);
        assert_eq!(pixel.get(7), 0xBA87);
    }

    #[test]
    fn test_set_frac() {
        let mut a = 0x12345678;
        let mut b = 0x98765432;
        let mut c = 0x12345678;
        let mut d = 0x98765432;
        let mut pixel = PixelView::new([&mut a, &mut b, &mut c, &mut d]);
        pixel.set_frac(0, 0.5);
        pixel.set_frac(1, 0.25);
        pixel.set_frac(2, 0.75);
        pixel.set_frac(3, 0.125);
        pixel.set_frac(4, 0.5);
        pixel.set_frac(5, 0.25);
        pixel.set_frac(6, 0.75);
        pixel.set_frac(7, 0.125);
        assert!(f32::abs(pixel.get_frac(0) - 0.5) < 0.01);
        assert!(f32::abs(pixel.get_frac(1) - 0.25) < 0.01);
        assert!(f32::abs(pixel.get_frac(2) - 0.75) < 0.01);
        assert!(f32::abs(pixel.get_frac(3) - 0.125) < 0.01);
        assert!(f32::abs(pixel.get_frac(4) - 0.5) < 0.01);
        assert!(f32::abs(pixel.get_frac(5) - 0.25) < 0.01);
        assert!(f32::abs(pixel.get_frac(6) - 0.75) < 0.01);
        assert!(f32::abs(pixel.get_frac(7) - 0.125) < 0.01);
    }
}
