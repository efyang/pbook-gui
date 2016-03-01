use std::iter;

pub fn make_string_if_nonzero(n: i64, id: &'static str) -> String {
    if n != 0 {
        return format!("{}{} ", n, id);
    } else {
        return "".to_owned();
    }
}

pub trait Ignore {
    fn ignore(&self);
}

impl<T, U> Ignore for Result<T, U> {
    fn ignore(&self) {}
}

const BYTE_UNITS: [(f32, &'static str); 4] = [(0.0, "B"),
                                              (1024.0, "KiB"),
                                              (1048576.0, "MiB"),
                                              (1073741800.0, "GiB")];

pub trait ToByteUnits {
    fn convert_to_byte_units(&self, decimal_places: usize) -> String;
}

impl ToByteUnits for f32 {
    fn convert_to_byte_units(&self, decimal_places: usize) -> String {
        let mut bunit = (0.0f32, "B");
        for bidx in 0..BYTE_UNITS.len() - 1 {
            let min = BYTE_UNITS[bidx];
            let max = BYTE_UNITS[bidx + 1];
            if (self >= &min.0) && (self < &max.0) {
                bunit = min;
            }
        }
        let last = BYTE_UNITS.last().unwrap().clone();
        if self >= &last.0 {
            bunit = last;
        }
        let divided = self / maximum(bunit.0, 1.0) as f32;
        format!("{} {}", round_to_places(divided, decimal_places), bunit.1)
    }
}

pub trait Repetition {
    fn repeat(&self, times: usize) -> String;
}

impl Repetition for str {
    fn repeat(&self, times: usize) -> String {
        iter::repeat(self).take(times).map(|s| s.clone()).collect::<String>()
    }
}

pub trait Shorten {
    fn shorten(&mut self, maxchars: isize) -> String;
}

impl Shorten for String {
    fn shorten(&mut self, maxchars: isize) -> String {
        let length = self.len() as isize;
        if length > maxchars {
            self.truncate(minimum(maximum(maxchars - 3, 0), length) as usize);
            self.clone() + "..."
        } else {
            self.clone()
        }
    }
}

pub fn maximum<T: PartialOrd>(x: T, y: T) -> T {
    if x >= y {
        x
    } else {
        y
    }
}

pub fn minimum<T: PartialOrd>(x: T, y: T) -> T {
    if x <= y {
        x
    } else {
        y
    }
}

pub fn round_to_places(n: f32, places: usize) -> f32 {
    let div = (format!("1{}", &"0".repeat(places))).parse::<f32>().unwrap();
    (n * div).round() / div
}
