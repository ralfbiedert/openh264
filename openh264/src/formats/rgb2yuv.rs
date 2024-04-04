use crate::formats::{rgbdata::RgbData, YUVSource};

/// Converts RGB to YUV data.
pub struct YUVBuffer {
    yuv: Vec<u8>,
    width: usize,
    height: usize,
}

impl YUVBuffer {
    /// Allocates a new YUV buffer with the given width and height.
    ///
    /// Both dimensions must be even.
    pub fn new(width: usize, height: usize) -> Self {
        Self::verify(Self {
            yuv: vec![0u8; (3 * (width * height)) / 2],
            width,
            height,
        })
    }

    /// Allocates a new YUV buffer with the given width and height and data.
    ///
    /// Data `rgb` format is specified the configured `RGBData` trait.
    ///
    /// Both dimensions must be even. May panic or yield unexpected results if `rgb`
    /// does not match the formats given.
    pub fn with_rgb<T: RgbData>(width: usize, height: usize, rgb: &T) -> Self {
        let mut rval = Self::new(width, height);
        rval.read_rgb(rgb);
        rval
    }

    /// Verify priors on inputs.
    ///
    /// Image dimensions must be even.
    fn verify(self) -> Self {
        assert_eq!(self.width % 2, 0, "width needs to be multiple of 2");
        assert_eq!(self.height % 2, 0, "height needs to be a multiple of 2");
        self
    }

    /// Reads an RGB buffer, converts it to YUV and stores it.
    ///
    /// Data `rgb` format is specified the configured `RGBData` trait.
    ///
    /// May panic or yield unexpected results if `rgb` does not match the formats given.
    pub fn read_rgb<T: RgbData>(&mut self, rgb: &T) {
        let width = self.width;
        let height = self.height;

        let u_base = width * height;
        let v_base = u_base + u_base / 4;
        let half_width = width / 2;

        // y is full size, u, v is quarter size
        let write_y = |yuv: &mut [u8], x: usize, y: usize, rgb: (f32, f32, f32)| {
            yuv[x + y * width] = (0.2578125 * rgb.0 + 0.50390625 * rgb.1 + 0.09765625 * rgb.2 + 16.0) as u8;
        };

        let write_u = |yuv: &mut [u8], x: usize, y: usize, rgb: (f32, f32, f32)| {
            yuv[u_base + x + y * half_width] = (-0.1484375 * rgb.0 + -0.2890625 * rgb.1 + 0.4375 * rgb.2 + 128.0) as u8;
        };

        let write_v = |yuv: &mut [u8], x: usize, y: usize, rgb: (f32, f32, f32)| {
            yuv[v_base + x + y * half_width] = (0.4375 * rgb.0 + -0.3671875 * rgb.1 + -0.0703125 * rgb.2 + 128.0) as u8;
        };

        for i in 0..width / 2 {
            for j in 0..height / 2 {
                let px = i * 2;
                let py = j * 2;
                let pix0x0 = rgb.pixel(px, py, width, height);
                let pix0x1 = rgb.pixel(px, py + 1, width, height);
                let pix1x0 = rgb.pixel(px + 1, py, width, height);
                let pix1x1 = rgb.pixel(px + 1, py + 1, width, height);
                let avg_pix = (
                    (pix0x0.0 as u32 + pix0x1.0 as u32 + pix1x0.0 as u32 + pix1x1.0 as u32) as f32 / 4.0,
                    (pix0x0.1 as u32 + pix0x1.1 as u32 + pix1x0.1 as u32 + pix1x1.1 as u32) as f32 / 4.0,
                    (pix0x0.2 as u32 + pix0x1.2 as u32 + pix1x0.2 as u32 + pix1x1.2 as u32) as f32 / 4.0,
                );
                write_y(&mut self.yuv[..], px, py, pix0x0);
                write_y(&mut self.yuv[..], px, py + 1, pix0x1);
                write_y(&mut self.yuv[..], px + 1, py, pix1x0);
                write_y(&mut self.yuv[..], px + 1, py + 1, pix1x1);
                write_u(&mut self.yuv[..], i, j, avg_pix);
                write_v(&mut self.yuv[..], i, j, avg_pix);
            }
        }
    }
}

impl YUVSource for YUVBuffer {
    fn width(&self) -> i32 {
        self.width as i32
    }

    fn height(&self) -> i32 {
        self.height as i32
    }

    fn y(&self) -> &[u8] {
        &self.yuv[0..self.width * self.height]
    }

    fn u(&self) -> &[u8] {
        let base_u = self.width * self.height;
        &self.yuv[base_u..base_u + base_u / 4]
    }

    fn v(&self) -> &[u8] {
        let base_u = self.width * self.height;
        let base_v = base_u + base_u / 4;
        &self.yuv[base_v..]
    }

    fn y_stride(&self) -> i32 {
        self.width as i32
    }

    fn u_stride(&self) -> i32 {
        (self.width / 2) as i32
    }

    fn v_stride(&self) -> i32 {
        (self.width / 2) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::YUVBuffer;
    use crate::formats::YUVSource;

    #[test]
    fn rgb_to_yuv_conversion_black_2x2() {
        let yuv = YUVBuffer::with_rgb(2, 2, &[0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8]);
        assert_eq!(yuv.y(), [16u8, 16u8, 16u8, 16u8]);
        assert_eq!(yuv.u(), [128u8]);
        assert_eq!(yuv.v(), [128u8]);
        assert_eq!(yuv.y_stride(), 2);
        assert_eq!(yuv.u_stride(), 1);
        assert_eq!(yuv.v_stride(), 1);
    }

    #[test]
    fn rgb_to_yuv_conversion_white_4x2() {
        let yuv = YUVBuffer::with_rgb(
            4,
            2,
            &[
                255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
                255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
            ],
        );
        assert_eq!(yuv.y(), [235u8, 235u8, 235u8, 235u8, 235u8, 235u8, 235u8, 235u8]);
        assert_eq!(yuv.u(), [128u8, 128u8]);
        assert_eq!(yuv.v(), [128u8, 128u8]);
        assert_eq!(yuv.y_stride(), 4);
        assert_eq!(yuv.u_stride(), 2);
        assert_eq!(yuv.v_stride(), 2);
    }

    #[test]
    fn rgb_to_yuv_conversion_red_2x4() {
        let yuv = YUVBuffer::with_rgb(
            4,
            2,
            &[
                255u8, 0u8, 0u8, 255u8, 0u8, 0u8, 255u8, 0u8, 0u8, 255u8, 0u8, 0u8, 255u8, 0u8, 0u8, 255u8, 0u8, 0u8, 255u8, 0u8,
                0u8, 255u8, 0u8, 0u8,
            ],
        );
        assert_eq!(yuv.y(), [81u8, 81u8, 81u8, 81u8, 81u8, 81u8, 81u8, 81u8]);
        assert_eq!(yuv.u(), [90u8, 90u8]);
        assert_eq!(yuv.v(), [239u8, 239u8]);
        assert_eq!(yuv.y_stride(), 4);
        assert_eq!(yuv.u_stride(), 2);
        assert_eq!(yuv.v_stride(), 2);
    }
}
