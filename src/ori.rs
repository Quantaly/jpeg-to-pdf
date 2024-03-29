#[derive(Debug, Clone, Copy)]
pub struct Orientation {
    pub value: u32,
    pub width: usize,
    pub height: usize,
}

impl Orientation {
    pub fn display_width(self) -> usize {
        match self.value {
            5..=8 => self.height,
            _ => self.width,
        }
    }

    pub fn display_height(self) -> usize {
        match self.value {
            5..=8 => self.width,
            _ => self.height,
        }
    }

    pub fn translate_x(self) -> Option<usize> {
        match self.value {
            2 | 3 => Some(self.width),
            5 | 8 => Some(self.height),
            _ => None,
        }
    }

    pub fn translate_y(self) -> Option<usize> {
        match self.value {
            3 | 4 => Some(self.height),
            5 | 6 => Some(self.width),
            _ => None,
        }
    }

    // based on testing, it seems like this is actually counterclockwise
    // but printpdf still calls it `rotate_cw`
    pub fn rotate_cw(self) -> Option<f64> {
        match self.value {
            3 | 4 => Some(180.0),
            5 | 8 => Some(90.0),
            6 | 7 => Some(270.0),
            _ => None,
        }
    }

    pub fn scale_x(self) -> Option<f64> {
        match self.value {
            2 | 4 | 5 | 7 => Some(-1.0),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct OrientationData {
        display_width: usize,
        display_height: usize,
        translate_x: Option<usize>,
        translate_y: Option<usize>,
        rotate_cw: Option<f64>,
        scale_x: Option<f64>,
    }

    fn test_orientation(value: u32, expected: fn(width: usize, height: usize) -> OrientationData) {
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
            translate_x: None,
            translate_y: None,
            rotate_cw: None,
            scale_x: None,
        });
    }

    #[test]
    fn orientation_2() {
        test_orientation(2, |width, height| OrientationData {
            display_width: width,
            display_height: height,
            translate_x: Some(width),
            translate_y: None,
            rotate_cw: None,
            scale_x: Some(-1.0),
        });
    }

    #[test]
    fn orientation_3() {
        test_orientation(3, |width, height| OrientationData {
            display_width: width,
            display_height: height,
            translate_x: Some(width),
            translate_y: Some(height),
            rotate_cw: Some(180.0),
            scale_x: None,
        });
    }

    #[test]
    fn orientation_4() {
        test_orientation(4, |width, height| OrientationData {
            display_width: width,
            display_height: height,
            translate_x: None,
            translate_y: Some(height),
            rotate_cw: Some(180.0),
            scale_x: Some(-1.0),
        });
    }

    #[test]
    fn orientation_5() {
        test_orientation(5, |width, height| OrientationData {
            display_width: height,
            display_height: width,
            translate_x: Some(height),
            translate_y: Some(width),
            rotate_cw: Some(90.0),
            scale_x: Some(-1.0),
        });
    }

    #[test]
    fn orientation_6() {
        test_orientation(6, |width, height| OrientationData {
            display_width: height,
            display_height: width,
            translate_x: None,
            translate_y: Some(width),
            rotate_cw: Some(270.0),
            scale_x: None,
        });
    }

    #[test]
    fn orientation_7() {
        test_orientation(7, |width, height| OrientationData {
            display_width: height,
            display_height: width,
            translate_x: None,
            translate_y: None,
            rotate_cw: Some(270.0),
            scale_x: Some(-1.0),
        });
    }

    #[test]
    fn orientation_8() {
        test_orientation(8, |width, height| OrientationData {
            display_width: height,
            display_height: width,
            translate_x: Some(height),
            translate_y: None,
            rotate_cw: Some(90.0),
            scale_x: None,
        });
    }
}
