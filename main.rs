use num::complex::Complex;
use num::Zero;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::{Write, BufWriter};
use std::ops::MulAssign;
use std::vec;

const PIXELS_PER_UNIT: i32 = 100;
const PX_WIDTH: i32        = 8 * PIXELS_PER_UNIT;
const PX_HEIGHT: i32       = 6 * PIXELS_PER_UNIT;
const STEPS: i32           = 20;

const MAX_X: i32  = PX_WIDTH / 2;
const MAX_Y: i32  = PX_HEIGHT / 2;
const SQRT_3: f32 = 1.732050;

const ROOTS: &[Complex<f32>] =
    &[ Complex::new(-1.0, 0.0),
       Complex::new((SQRT_3)/2.0, 1.0/2.0),
       Complex::new((SQRT_3)/2.0, -1.0/2.0),
       Complex::new(0.0, 1.0),
       Complex::new(0.0, -1.0),
    ];

const COLORS: &[Pixel] =
    &[ 0x4a0b58,
       0x39538e,
       0x1fa0cf,
       0x56b861,
       0x19858f,
    ];

type Pixel = u32;

fn to_rgb(p: &Pixel) -> (u8, u8, u8) {
    let r: u8 = ((p >> 16) & 0xff) as u8;
    let g: u8 = ((p >> 8) & 0xff) as u8;
    let b: u8 = ((p >> 0) & 0xff) as u8;
    (r, g, b)
}

#[derive(Clone)]
struct Polynom {
    cs: Vec<Complex<f32>>,
}

impl MulAssign<Polynom> for Polynom {
    fn mul_assign(&mut self, rhs: Polynom) {
        let len = self.cs.len() - 1 + rhs.cs.len() - 1 + 1;
        let mut res = Polynom{cs:vec![Complex::zero(); len]};
        for i in 0..self.cs.len() {
            for j in 0..rhs.cs.len() {
                res.cs[i+j] += self.cs[i] * rhs.cs[j];
            }
        }
        *self = res;
    }
}

impl fmt::Display for Polynom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in (0..self.cs.len()).rev() {
            write!(f, "{}: ({}) ", i, self.cs[i])?;
        }
        Ok(())
    }
}

impl Polynom {

    fn at(&self, coord: Complex<f32>) -> Complex<f32> {
        let mut res: Complex<f32> = Complex::zero();
        for i in 0..self.cs.len() {
            res += self.cs[i] * coord.powu(i as u32);
        }
        res
    }

    fn from_roots() -> Polynom {
        let mut pol = Polynom{
            cs:vec![Complex::new(1.0, 0.0)]
        };
        for root in ROOTS.iter() {
            pol *= Polynom{cs:vec![Complex::new(1.0, 0.0), -root]}
        }
        pol
    }

    fn derivative(&self) -> Polynom {
        let mut res = self.clone();
        for i in 0..res.cs.len()-1 {
            res.cs[i] = Complex::new((i + 1) as f32, 0.0) * res.cs[i+1];
        }
        res.cs.truncate(res.cs.len()-1);
        res
    }
}

fn between(x: f32, a: f32, b: f32) -> bool {
    x >= a && x <= b
}

fn get_color(pol: &Polynom, der: &Polynom, ic: Complex<f32>) -> Pixel {
    let mut c = ic;
    for _ in 0..STEPS {
        let (yp, yd) = (pol.at(c), der.at(c));
        if yd == Complex::zero() || c.is_nan() {
            break;
        }
        c = c - yp / yd;
        for i in 0..ROOTS.len() {
            if ROOTS[i] == c {
                return COLORS[i]
            }
        }
    }

    let dists: Vec<f32> =
            ROOTS.iter()
            .map(|r| {(c - r).norm()})
            .collect();
    let mut index = 0;
    let mut min = dists.first().unwrap();
    for i in 1..dists.len() {
        if &dists[i] < min {
            min = &dists[i];
            index = i;
        }
    }
    COLORS[index]
}

fn write_ppm(s: &mut impl Write, canv: &Vec<Pixel>) -> io::Result<()> {
    write!(s, "P6\n")?;
    write!(s, "{} {}\n", PX_WIDTH, PX_HEIGHT)?;
    write!(s, "255\n")?;
    for y in 0..PX_HEIGHT {
        for x in 0..PX_WIDTH {
            let (r, g, b) = to_rgb(&canv[(y * PX_WIDTH + x) as usize]);
            s.write(&vec![r, g, b])?;
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    assert!(ROOTS.len() != 0, "No roots specified");
    assert!(ROOTS.len() <= COLORS.len(), "Not enough colors to mark all roots");
    for root in ROOTS.iter() {
        let mx = MAX_X as f32 / PIXELS_PER_UNIT as f32;
        let my = MAX_Y as f32 / PIXELS_PER_UNIT as f32;
        assert!(between(root.re, -mx, mx),
                "Root {} is out of image bounds", root);
        assert!(between(root.im, -my, my),
                "Root {} is out of image bounds", root);
    }
    let pol = Polynom::from_roots();
    let der = pol.derivative();
    println!("Pol: {}", pol);
    println!("Der: {}", der);

    let mut canvas = vec![0 as Pixel; (PX_WIDTH * PX_HEIGHT) as usize];
    for y in 0..PX_HEIGHT {
        for x in 0..PX_WIDTH {
            let cx = (x - MAX_X) as f32 / PIXELS_PER_UNIT as f32;
            let cy = (y - MAX_Y) as f32 / PIXELS_PER_UNIT as f32;
            let col = get_color(&pol, &der, Complex::new(cx as f32, cy as f32));
            canvas[(y * PX_WIDTH + x) as usize] = col
        }
    }

    let mut of = BufWriter::new(File::create("img.ppm")?);
    write_ppm(&mut of, &canvas)?;
    Ok(())
}
