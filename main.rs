use core::f32::consts::PI;
use num::complex::Complex;
use num::Zero;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::{Write, BufWriter};
use std::ops::MulAssign;
use std::vec;

const PPX: i32 = 100;
const WIDTH: i32 = 8 * PPX;
const HEIGHT: i32 = 6 * PPX;
const STEPS: i32 = 20;

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
        let mut res = Polynom{cs:Vec::with_capacity(len)};
        res.cs.resize(len, Complex::zero());
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

    fn from_root(root: Complex<f32>) -> Polynom {
        Polynom{cs:vec![Complex::new(1.0, 0.0), -root]}
    }

    fn from_roots(roots: &Vec<Complex<f32>>) -> Polynom {
        let mut pol = Polynom{
            cs:vec![Complex::new(1.0, 0.0)]
        };
        for i in 0..roots.len() {
            pol *= Polynom::from_root(roots[i]);
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

fn distance(a: &Complex<f32>, b: &Complex<f32>) -> f32 {
    let d = b-a;
    (d.re.powi(2) + d.im.powi(2)).sqrt()
}

fn try_point(roots: &Vec<Complex<f32>>, pol: &Polynom, der: &Polynom, c: Complex<f32>) -> usize {
    let mut cur = c;
    for _ in 0..STEPS {
        let (yp, yd) = (pol.at(cur), der.at(cur));
        if yd == Complex::zero() || cur.is_nan() {
            break;
        }
        let nc = cur - yp / yd;
        for i in 0..roots.len() {
            if roots[i] == nc {
                return i as usize
            }
        }
        cur = nc;
    }
    let dists: Vec<f32> = roots.iter().map(|r| {distance(&cur, &r)}).collect();
    let mut index = 0;
    let mut min = dists.first().unwrap();
    for i in 1..dists.len() {
        if &dists[i] < min {
            min = &dists[i];
            index = i;
        }
    }
    index
}

fn write_ppm(s: &mut impl Write, canv: &Vec<Pixel>) -> io::Result<()> {
    write!(s, "P6\n")?;
    write!(s, "{} {}\n", WIDTH, HEIGHT)?;
    write!(s, "255\n")?;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let (r, g, b) = to_rgb(&canv[(y * WIDTH + x) as usize]);
            s.write(&vec![r, g, b])?;
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let colors: Vec<Pixel> = vec![0x4a0b58,
                                  0x39538e,
                                  0x1fa0cf,
                                  0x56b861,
                                  0x19858f,
                                 ];
    let roots: Vec<Complex<f32>> =
        vec![ Complex::new(-1.0, 0.0),
              Complex::from_polar(1.0f32, PI / 6.0),
              Complex::from_polar(1.0f32, PI / 6.0).conj(),
              Complex::new(0.0, 1.0),
              Complex::new(0.0, -1.0),
            ];
    assert!(roots.len() != 0);
    assert!(roots.len() <= colors.len());
    let pol = Polynom::from_roots(&roots);
    let der = pol.derivative();
    println!("Pol: {}", pol);
    println!("Der: {}", der);

    let mut canvas: Vec<Pixel> = Vec::with_capacity((WIDTH * HEIGHT) as usize);
    canvas.resize((WIDTH*HEIGHT) as usize, 0);
    let (mx, my) = (WIDTH/2, HEIGHT/2);
    // TODO: Parallelize rendering
    for y in -my..my {
        for x in -mx..mx {
            let px = x as f32 / PPX as f32;
            let py = y as f32 / PPX as f32;
            let c = try_point(&roots, &pol, &der,
                              Complex::new(px as f32, py as f32)
                             );
            canvas[((y + my) * WIDTH + (mx + x)) as usize] = colors[c]
        }
    }
    let mut of = BufWriter::new(File::create("img.ppm")?);
    write_ppm(&mut of, &canvas)?;
    Ok(())
}
