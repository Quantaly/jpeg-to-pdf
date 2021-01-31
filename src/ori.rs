#[derive(Debug, Clone, Copy)]
pub struct Orientation {
    pub value: u16,
    pub width: u16,
    pub height: u16,
}

impl Orientation {
    pub fn display_width(self) -> u16 {
        if self.value > 4 {
            self.height
        } else {
            self.width
        }
    }

    pub fn display_height(self) -> u16 {
        if self.value > 4 {
            self.width
        } else {
            self.height
        }
    }

    pub fn translate_x(self) -> u16 {
        match self.value {
            3..=4 => self.width,
            7..=8 => self.height,
            _ => 0,
        }
    }

    pub fn translate_y(self) -> u16 {
        match self.value {
            3..=4 => self.height,
            5..=6 => self.width,
            _ => 0,
        }
    }

    // based on testing, it seems like this is actually counterclockwise
    // but printpdf still calls it `rotate_cw`
    pub fn rotate_cw(self) -> f64 {
        match self.value {
            5..=6 => 270.0,
            3..=4 => 180.0,
            7..=8 => 90.0,
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

    // TODO add tests for the other seven orientations

    #[test]
    fn orientation_6() {
        let (width, height) = (4032, 3024);
        let ori = Orientation {
            value: 6,
            width,
            height,
        };

        assert_eq!(ori.display_width(), height);
        assert_eq!(ori.display_height(), width);
        assert_eq!(ori.translate_x(), 0);
        assert_eq!(ori.translate_y(), width);
        assert_eq!(ori.rotate_cw(), 270.0);
        assert_eq!(ori.scale_x(), 1.0);
    }
}
