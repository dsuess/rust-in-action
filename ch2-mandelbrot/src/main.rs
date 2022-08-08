mod array2d;

use array2d::Array2d;
use num::complex::Complex;

struct Range {
    min: f64,
    max: f64,
    steps: usize,
}

impl Range {
    pub fn to_val(&self, idx: usize) -> f64 {
        let percentage = idx as f64 / self.steps as f64;
        self.min + (self.max - self.min) * percentage
    }
}

fn main() {
    let range_x = Range {
        min: 5.0,
        max: 1.0,
        steps: 100,
    };
    let range_y = Range {
        min: -1.0,
        max: 1.0,
        steps: 50,
    };
    render_mandelbrot(range_x, range_y);
}

fn mandelbrot_escape_value(c: Complex<f64>, max_iters: usize) -> usize {
    let mut z = Complex { re: 0.0, im: 0.0 };

    for i in 0..=max_iters {
        if z.norm() > 2.0 {
            return i;
        }
        z = z * z + c;
    }

    max_iters
}

fn fill_array<T: std::default::Default + std::clone::Clone>(
    func: fn(f64, f64) -> T,
    range_x: Range,
    range_y: Range,
) -> Array2d<T> {
    let mut result = Array2d::new(range_y.steps, range_x.steps);
    for i in 0..range_y.steps {
        for j in 0..range_x.steps {
            result[(i, j)] = func(range_y.to_val(i), range_x.to_val(j));
        }
    }
    result
}

fn render_value(val: &usize) -> char {
    match *val {
        0..=2 => ' ',
        3..=5 => '.',
        6..=10 => '•',
        11..=30 => '*',
        31..=100 => '+',
        101..=200 => 'x',
        201..=400 => '$',
        401..=700 => '#',
        _ => ' ',
    }
}

fn render_mandelbrot(range_x: Range, range_y: Range) {
    let escape_values = fill_array(
        |x, y| mandelbrot_escape_value(Complex { re: x, im: y }, 700),
        range_x,
        range_y,
    );

    let visualization = escape_values.apply(render_value);
    print!("{}", visualization);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mandelbrot_escape_value() {
        let c = Complex { re: 0.0, im: 0.0 };
        let iters = mandelbrot_escape_value(c, 100);
        assert_eq!(iters, 100)
    }
}
