#[derive(Debug, Clone, Copy)]
pub struct Orientation {
    pub value: u32,
    pub width: u16,
    pub height: u16,
}

impl Orientation {
    pub fn display_width(self) -> u16 {
        match self.value {
            5..=8 => self.height,
            _ => self.width,
        }
    }

    pub fn display_height(self) -> u16 {
        match self.value {
            5..=8 => self.width,
            _ => self.height,
        }
    }

    pub fn translate_x(self) -> u16 {
        match self.value {
            2 | 3 => self.width,
            5 | 8 => self.height,
            _ => 0,
        }
    }

    pub fn translate_y(self) -> u16 {
        match self.value {
            3 | 4 => self.height,
            5 | 6 => self.width,
            _ => 0,
        }
    }

    // based on testing, it seems like this is actually counterclockwise
    // but printpdf still calls it `rotate_cw`
    pub fn rotate_cw(self) -> f64 {
        match self.value {
            3 | 4 => 180.0,
            5 | 8 => 90.0,
            6 | 7 => 270.0,
            _ => 0.0,
        }
    }

    pub fn scale_x(self) -> f64 {
        match self.value {
            2 | 4 | 5 | 7 => -1.0,
            _ => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct OrientationData {
        display_width: u16,
        display_height: u16,
        translate_x: u16,
        translate_y: u16,
        rotate_cw: f64,
        scale_x: f64,
    }

    fn test_orientation(value: u32, expected: fn(width: u16, height: u16) -> OrientationData) {
        let (width, height) = (4032, 3024);

        let ori = Orientation {
            value,
            width,
            height,
        };

        let actual = OrientationData {
            display_width: ori.display_width(),
            display_height: ori.display_height(),
            translate_x: ori.translate_x(),
            translate_y: ori.translate_y(),
            rotate_cw: ori.rotate_cw(),
            scale_x: ori.scale_x(),
        };
        let expected = expected(width, height);

        assert_eq!(actual, expected);
    }

    #[test]
    fn orientation_1() {
        test_orientation(1, |width, height| OrientationData {
            display_width: width,
            display_height: height,
            translate_x: 0,
            translate_y: 0,
            rotate_cw: 0.0,
            scale_x: 1.0,
        });
    }

    #[test]
    fn orientation_2() {
        test_orientation(2, |width, height| OrientationData {
            display_width: width,
            display_height: height,
            translate_x: width,
            translate_y: 0,
            rotate_cw: 0.0,
            scale_x: -1.0,
        });
    }

    #[test]
    fn orientation_3() {
        test_orientation(3, |width, height| OrientationData {
            display_width: width,
            display_height: height,
            translate_x: width,
            translate_y: height,
            rotate_cw: 180.0,
            scale_x: 1.0,
        });
    }

    #[test]
    fn orientation_4() {
        test_orientation(4, |width, height| OrientationData {
            display_width: width,
            display_height: height,
            translate_x: 0,
            translate_y: height,
            rotate_cw: 180.0,
            scale_x: -1.0,
        });
    }

    #[test]
    fn orientation_5() {
        test_orientation(5, |width, height| OrientationData {
            display_width: height,
            display_height: width,
            translate_x: height,
            translate_y: width,
            rotate_cw: 90.0,
            scale_x: -1.0,
        });
    }

    #[test]
    fn orientation_6() {
        test_orientation(6, |width, height| OrientationData {
            display_width: height,
            display_height: width,
            translate_x: 0,
            translate_y: width,
            rotate_cw: 270.0,
            scale_x: 1.0,
        });
    }

    #[test]
    fn orientation_7() {
        test_orientation(7, |width, height| OrientationData {
            display_width: height,
            display_height: width,
            translate_x: 0,
            translate_y: 0,
            rotate_cw: 270.0,
            scale_x: -1.0,
        });
    }

    #[test]
    fn orientation_8() {
        test_orientation(8, |width, height| OrientationData {
            display_width: height,
            display_height: width,
            translate_x: height,
            translate_y: 0,
            rotate_cw: 90.0,
            scale_x: 1.0,
        });
    }
}
