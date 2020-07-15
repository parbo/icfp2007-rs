use std::collections::VecDeque;

type Coord = i32;

#[derive(Clone, Copy)]
struct Pos {
    x: Coord,
    y: Coord,
}

type Component = u8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rgb {
    pub r: Component,
    pub g: Component,
    pub b: Component,
}

type Transparency = Component;

const BLACK: Rgb = Rgb { r: 0, g: 0, b: 0 };
const RED: Rgb = Rgb { r: 255, g: 0, b: 0 };
const GREEN: Rgb = Rgb { r: 0, g: 255, b: 0 };
const YELLOW: Rgb = Rgb {
    r: 255,
    g: 255,
    b: 0,
};
const BLUE: Rgb = Rgb { r: 0, g: 0, b: 255 };
const MAGENTA: Rgb = Rgb {
    r: 255,
    g: 0,
    b: 255,
};
const CYAN: Rgb = Rgb {
    r: 0,
    g: 255,
    b: 255,
};
const WHITE: Rgb = Rgb {
    r: 255,
    g: 255,
    b: 255,
};
const TRANSPARENT: Component = 0;
const OPAQUE: Component = 255;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pixel {
    pub color: Rgb,
    pub alpha: Transparency,
}

impl Pixel {
    fn new(r: Component, g: Component, b: Component, a: Transparency) -> Pixel {
        Pixel {
            color: Rgb { r, g, b },
            alpha: a,
        }
    }
}

#[derive(Clone)]
pub struct Bitmap {
    pub pixels: Vec<Pixel>, // 600 x 600
}

impl Bitmap {
    fn new() -> Bitmap {
        Bitmap {
            pixels: vec![Pixel::new(0, 0, 0, TRANSPARENT); 360000],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Color {
    Rgb(Rgb),
    Transparency(Transparency),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Dir {
    N,
    E,
    S,
    W,
}

pub struct Fuun {
    rna: Vec<String>,
    bucket: Vec<Color>,
    position: Pos,
    mark: Pos,
    dir: Dir,
    bitmaps: VecDeque<Bitmap>,
    current: Option<Pixel>,
    fill_todo: Vec<Pos>,
    step: usize,
}

impl Fuun {
    pub fn new(rna_str: &str) -> Fuun {
        let mut bitmaps = VecDeque::new();
        bitmaps.push_front(Bitmap::new());
        let mut f = Fuun {
            rna: vec![],
            bucket: vec![],
            position: Pos { x: 0, y: 0 },
            mark: Pos { x: 0, y: 0 },
            dir: Dir::E,
            bitmaps,
            current: None,
            fill_todo: vec![],
            step: 0,
        };
        f.fill_todo.reserve(360000);
        f.add_rna_str(rna_str);
        f
    }

    pub fn reset(&mut self) {
        self.rna.clear();
        self.bucket.clear();
        self.position = Pos { x: 0, y: 0 };
        self.mark = self.position;
        self.dir = Dir::E;
        self.bitmaps.clear();
        self.bitmaps.push_front(Bitmap::new());
        self.current = None;
        self.fill_todo.clear();
        self.step = 0;
    }

    pub fn add_rna_command(&mut self, rna: String) {
        assert!(rna.len() == 7);
        self.rna.push(rna);
    }

    pub fn add_rna_str(&mut self, rna_str: &str) {
        for s in (0..rna_str.len()).step_by(7) {
            let e = std::cmp::min(s + 7, rna_str.len());
            let code = &rna_str[s..e];
            self.add_rna_command(code.into());
        }
    }

    pub fn remaining_steps(&self) -> usize {
        self.rna.len() - self.step
    }

    fn add_color(&mut self, color: Color) {
        self.current = None;
        self.bucket.insert(0, color);
    }

    fn current_pixel(&mut self) -> Pixel {
        if let Some(pixel) = self.current {
            return pixel;
        }
        let mut rsum = 0usize;
        let mut rcnt = 0usize;
        let mut gsum = 0usize;
        let mut gcnt = 0usize;
        let mut bsum = 0usize;
        let mut bcnt = 0usize;
        let mut asum = 0usize;
        let mut acnt = 0usize;
        for c in &self.bucket {
            match c {
                Color::Rgb(rgb) => {
                    rsum = rsum + rgb.r as usize;
                    rcnt = rcnt + 1;
                    gsum = gsum + rgb.g as usize;
                    gcnt = gcnt + 1;
                    bsum = bsum + rgb.b as usize;
                    bcnt = bcnt + 1;
                }
                Color::Transparency(alpha) => {
                    asum = asum + *alpha as usize;
                    acnt = acnt + 1;
                }
            }
        }
        let rc = if rcnt > 0 { rsum / rcnt } else { 0 };
        let gc = if gcnt > 0 { gsum / gcnt } else { 0 };
        let bc = if bcnt > 0 { bsum / bcnt } else { 0 };
        let ac = if acnt > 0 { asum / acnt } else { 255 };
        let p = Pixel::new(
            ((rc * ac) / 255) as Component,
            ((gc * ac) / 255) as Component,
            ((bc * ac) / 255) as Component,
            ac as Component,
        );
        self.current = Some(p);
        p
    }

    fn move_dir(pos: Pos, d: Dir) -> Pos {
        let mut y = pos.y;
        let mut x = pos.x;
        match d {
            Dir::N => y = y - 1,
            Dir::E => x = x + 1,
            Dir::S => y = y + 1,
            Dir::W => x = x - 1,
        }
        if y < 0 {
            y = 599;
        } else if y > 599 {
            y = 0;
        }
        if x < 0 {
            x = 599;
        } else if x > 599 {
            x = 0;
        }
        Pos { x, y }
    }

    fn turn_ccw(d: Dir) -> Dir {
        match d {
            Dir::N => Dir::W,
            Dir::E => Dir::N,
            Dir::S => Dir::E,
            Dir::W => Dir::S,
        }
    }

    fn turn_cw(d: Dir) -> Dir {
        match d {
            Dir::N => Dir::E,
            Dir::E => Dir::S,
            Dir::S => Dir::W,
            Dir::W => Dir::N,
        }
    }

    fn get_pixel(&self, p: Pos) -> Pixel {
        let ix = (p.y * 600 + p.x) as usize;
        self.bitmaps[0].pixels[ix]
    }

    fn set_pixel(&mut self, p: Pos) {
        let ix = (p.y * 600 + p.x) as usize;
        self.bitmaps[0].pixels[ix] = self.current_pixel();
    }

    fn line(&mut self, p0: Pos, p1: Pos) {
        let deltax = p1.x - p0.x;
        let deltay = p1.y - p0.y;
        let d = std::cmp::max(deltax.abs(), deltay.abs());
        let c = if deltax * deltay <= 0 { 1 } else { 0 };
        let mut x = p0.x * d + ((d - c) / 2);
        let mut y = p0.y * d + ((d - c) / 2);
        for _ in 0..d {
            let p = Pos { x: x / d, y: y / d };
            self.set_pixel(p);
            x = x + deltax;
            y = y + deltay;
        }
        self.set_pixel(p1);
    }

    fn try_fill(&mut self) {
        let new = self.current_pixel();
        let old = self.get_pixel(self.position);
        if new != old {
            self.fill(self.position, old);
        }
    }

    fn fill(&mut self, pos: Pos, initial: Pixel) {
        self.fill_todo.push(pos);
        while let Some(p) = self.fill_todo.pop() {
            if self.get_pixel(p) == initial {
                self.set_pixel(p);
                if p.x > 0 {
                    self.fill_todo.push(Pos { x: p.x - 1, y: p.y });
                }
                if p.x < 599 {
                    self.fill_todo.push(Pos { x: p.x + 1, y: p.y });
                }
                if p.y > 0 {
                    self.fill_todo.push(Pos { x: p.x, y: p.y - 1 });
                }
                if p.y < 599 {
                    self.fill_todo.push(Pos { x: p.x, y: p.y + 1 });
                }
            }
        }
    }

    fn add_bitmap(&mut self) {
        if self.bitmaps.len() < 10 {
            self.bitmaps.push_front(Bitmap::new());
        }
    }

    fn compose(&mut self) {
        if self.bitmaps.len() >= 2 {
            for y in 0..600 {
                for x in 0..600 {
                    let ix = (y * 600 + x) as usize;
                    let pixel0 = self.bitmaps[0].pixels[ix];
                    let r0 = pixel0.color.r as usize;
                    let g0 = pixel0.color.g as usize;
                    let b0 = pixel0.color.b as usize;
                    let a0 = pixel0.alpha as usize;
                    let pixel1 = self.bitmaps[1].pixels[ix];
                    let r1 = pixel1.color.r as usize;
                    let g1 = pixel1.color.g as usize;
                    let b1 = pixel1.color.b as usize;
                    let a1 = pixel1.alpha as usize;
                    let pixel = Pixel::new(
                        (r0 + r1 * (255 - a0) / 255) as Component,
                        (g0 + g1 * (255 - a0) / 255) as Component,
                        (b0 + b1 * (255 - a0) / 255) as Component,
                        (a0 + a1 * (255 - a0) / 255) as Transparency,
                    );
                    self.bitmaps[1].pixels[ix] = pixel;
                }
            }
            self.bitmaps.pop_front();
        }
    }

    fn clip(&mut self) {
        if self.bitmaps.len() >= 2 {
            for y in 0..600 {
                for x in 0..600 {
                    let ix = (y * 600 + x) as usize;
                    let pixel0 = self.bitmaps[0].pixels[ix];
                    let a0 = pixel0.alpha as usize;
                    let pixel1 = self.bitmaps[1].pixels[ix];
                    let r1 = pixel1.color.r as usize;
                    let g1 = pixel1.color.g as usize;
                    let b1 = pixel1.color.b as usize;
                    let a1 = pixel1.alpha as usize;
                    let pixel = Pixel::new(
                        (r1 * a0 / 255) as Component,
                        (g1 * a0 / 255) as Component,
                        (b1 * a0 / 255) as Component,
                        (a1 * a0 / 255) as Transparency,
                    );
                    self.bitmaps[1].pixels[ix] = pixel;
                }
            }
            self.bitmaps.pop_front();
        }
    }

    pub fn step(&mut self, steps: usize) -> (Bitmap, bool) {
        let start = self.step;
        let end = std::cmp::min(self.step + steps, self.rna.len());
        for s in start..end {
            let code = &self.rna[s];
            match code.as_str() {
                "PIPIIIC" => self.add_color(Color::Rgb(BLACK)),
                "PIPIIIP" => self.add_color(Color::Rgb(RED)),
                "PIPIICC" => self.add_color(Color::Rgb(GREEN)),
                "PIPIICF" => self.add_color(Color::Rgb(YELLOW)),
                "PIPIICP" => self.add_color(Color::Rgb(BLUE)),
                "PIPIIFC" => self.add_color(Color::Rgb(MAGENTA)),
                "PIPIIFF" => self.add_color(Color::Rgb(CYAN)),
                "PIPIIPC" => self.add_color(Color::Rgb(WHITE)),
                "PIPIIPF" => self.add_color(Color::Transparency(TRANSPARENT)),
                "PIPIIPP" => self.add_color(Color::Transparency(OPAQUE)),
                "PIIPICP" => {
                    self.current = None;
                    self.bucket.clear();
                }
                "PIIIIIP" => self.position = Fuun::move_dir(self.position, self.dir),
                "PCCCCCP" => self.dir = Fuun::turn_ccw(self.dir),
                "PFFFFFP" => self.dir = Fuun::turn_cw(self.dir),
                "PCCIFFP" => self.mark = self.position,
                "PFFICCP" => self.line(self.position, self.mark),
                "PIIPIIP" => self.try_fill(),
                "PCCPFFP" => self.add_bitmap(),
                "PFFPCCP" => self.compose(),
                "PFFICCF" => self.clip(),
                _ => {}
            }
        }
        self.step = end;
        log::info!("step: {} / {}", self.step, self.rna.len());
        (self.bitmaps[0].clone(), self.step == self.rna.len())
    }

    pub fn build(&mut self) -> Bitmap {
        let (bmp, _done) = self.step(self.rna.len() - self.step);
        bmp
    }

    pub fn is_draw_command(rna: &str) -> bool {
        match rna {
            "PFFICCP" => true,
            "PIIPIIP" => true,
            "PFFPCCP" => true,
            "PFFICCF" => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_pixel() {
        let b = Color::Rgb(BLACK);
        let r = Color::Rgb(RED);
        let m = Color::Rgb(MAGENTA);
        let w = Color::Rgb(WHITE);
        let y = Color::Rgb(YELLOW);
        let c = Color::Rgb(CYAN);
        let t = Color::Transparency(TRANSPARENT);
        let o = Color::Transparency(OPAQUE);
        let mut fuun_1 = Fuun::new("");
        fuun_1.add_color(t);
        fuun_1.add_color(o);
        fuun_1.add_color(o);
        assert_eq!(fuun_1.current_pixel(), Pixel::new(0, 0, 0, 170));
        let mut fuun_2 = Fuun::new("");
        fuun_2.add_color(b);
        fuun_2.add_color(y);
        fuun_2.add_color(c);
        assert_eq!(fuun_2.current_pixel(), Pixel::new(85, 170, 85, 255));
        let mut fuun_3 = Fuun::new("");
        fuun_3.add_color(y);
        fuun_3.add_color(t);
        fuun_3.add_color(o);
        assert_eq!(fuun_3.current_pixel(), Pixel::new(127, 127, 0, 127));
        let mut fuun_4 = Fuun::new("");
        for _ in 0..18 {
            fuun_4.add_color(b);
        }
        for _ in 0..7 {
            fuun_4.add_color(r);
        }
        for _ in 0..39 {
            fuun_4.add_color(m);
        }
        for _ in 0..10 {
            fuun_4.add_color(w);
        }
        for _ in 0..3 {
            fuun_4.add_color(o);
        }
        fuun_4.add_color(t);
        assert_eq!(fuun_4.current_pixel(), Pixel::new(143, 25, 125, 191));
    }
}
