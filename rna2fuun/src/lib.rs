type Coord = u32;

#[derive(Clone, Copy)]
struct Pos {
    x: Coord,
    y: Coord,
}

type Component = u8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Rgb {
    r: Component,
    g: Component,
    b: Component,
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
struct Pixel {
    color: Rgb,
    alpha: Transparency,
}

impl Pixel {
    fn new(r: Component, g: Component, b: Component, a: Transparency) -> Pixel {
        Pixel {
            color: Rgb { r, g, b },
            alpha: a,
        }
    }
}

struct Bitmap {
    pixels: Vec<Pixel>, // 600 x 600
}

impl Bitmap {
    fn new() -> Bitmap {
        Bitmap {
            pixels: vec![Pixel::new(0, 0, 0, 0); 360000],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Color {
    Rgb(Rgb),
    Transparency(Transparency),
}

enum Dir {
    N,
    E,
    S,
    W,
}

struct Fuun {
    bucket: Vec<Color>,
    position: Pos,
    mark: Pos,
    dir: Dir,
    bitmaps: Vec<Bitmap>,
}

impl Fuun {
    fn new() -> Fuun {
        Fuun {
            bucket: vec![],
            position: Pos { x: 0, y: 0 },
            mark: Pos { x: 0, y: 0 },
            dir: Dir::E,
            bitmaps: vec![Bitmap::new()],
        }
    }

    fn add_color(&mut self, color: Color) {
        self.bucket.insert(0, color);
    }

    fn current_pixel(&self) -> Pixel {
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
        Pixel::new(
            ((rc * ac) / 255) as Component,
            ((gc * ac) / 255) as Component,
            ((bc * ac) / 255) as Component,
            ac as Component,
        )
    }
}

pub fn add_one(x: i32) -> i32 {
    x + 1
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
        let mut fuun_1 = Fuun::new();
        fuun_1.add_color(t);
        fuun_1.add_color(o);
        fuun_1.add_color(o);
        assert_eq!(fuun_1.current_pixel(), Pixel::new(0, 0, 0, 170));
        let mut fuun_2 = Fuun::new();
        fuun_2.add_color(b);
        fuun_2.add_color(y);
        fuun_2.add_color(c);
        assert_eq!(fuun_2.current_pixel(), Pixel::new(85, 170, 85, 255));
        let mut fuun_3 = Fuun::new();
        fuun_3.add_color(y);
        fuun_3.add_color(t);
        fuun_3.add_color(o);
        assert_eq!(fuun_3.current_pixel(), Pixel::new(127, 127, 0, 127));
        let mut fuun_4 = Fuun::new();
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
