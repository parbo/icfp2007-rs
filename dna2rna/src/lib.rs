use log;
use ropey::Rope;
use std::collections::VecDeque;
use std::io::Write;

pub struct Dna2Rna {
    dna: Rope,
    pub rna: Rope,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum PItem {
    Base(char),
    Skip(usize),
    Search(String),
    Open,
    Close,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TItem {
    Base(char),
    Ref(usize, usize),
    RefLen(usize),
}

impl Dna2Rna {
    pub fn new(dna_str: &str, prefix: Option<&str>) -> Dna2Rna {
        let mut dna = if let Some(p) = prefix {
            Rope::from_str(&p)
        } else {
            Rope::new()
        };
        log::info!("using prefix of len {}", dna.len_chars());
        dna.append(Rope::from_str(dna_str));
        Dna2Rna {
            dna,
            rna: Rope::new(),
        }
    }

    pub fn execute(&mut self) {
        log::info!("dna is {} bases long", self.dna.len_chars());
        log::info!("dna starts with {}", self.dna.slice(0..32).to_string());
        let mut i = 0;
        loop {
            if self.execute_step() {
                break;
            }
            i = i + 1;
            if i % 1000 == 0 {
                log::info!(
                    "at step {}, dna: {}, rna: {}",
                    i,
                    self.dna.len_chars(),
                    self.rna.len_chars()
                );
                log::info!("dna starts with {}", self.dna.slice(0..32).to_string());
            }
        }
        log::info!("rna is {} bases long", self.rna.len_chars());
        log::debug!("remaining dna: {:?}", self.dna);
    }

    pub fn execute_step(&mut self) -> bool {
        if let Some(p) = self.pattern() {
            if let Some(t) = self.template() {
                self.match_replace(&p, &t);
            } else {
                return true;
            }
        } else {
            return true;
        }
        false
    }

    pub fn save_rna<T: Write>(&self, writer: T) -> std::io::Result<()> {
        self.dna.write_to(writer)
    }

    fn nat(mut chars: ropey::iter::Chars) -> Option<(usize, usize)> {
        let mut bits = vec![];
        let mut consumed = 0;
        loop {
            let c = chars.next()?;
            consumed = consumed + 1;
            match c {
                'P' => break,
                'I' | 'F' => bits.push(0),
                'C' => bits.push(1),
                _ => panic!(),
            }
        }
        let mut n = 0;
        for b in bits.iter().rev() {
            n = n * 2 + b
        }
        Some((n, consumed))
    }

    fn consts(mut chars: ropey::iter::Chars) -> Option<(String, usize)> {
        let mut s = vec![];
        let mut extra = 0;
        loop {
            let c = chars.next()?;
            match c {
                'C' => s.push('I'),
                'F' => s.push('C'),
                'P' => s.push('F'),
                'I' => {
                    let cc = chars.next()?;
                    if cc == 'C' {
                        extra = extra + 1;
                        s.push('P');
                    } else {
                        break;
                    }
                }
                _ => return None,
            }
        }
        let consumed = s.len() + extra;
        let s: String = s.into_iter().rev().collect();
        Some((s, consumed))
    }

    fn pattern(&mut self) -> Option<Vec<PItem>> {
        let mut p = vec![];
        let mut level = 0;
        let mut ret = false;
        while !ret {
            let mut chars = self.dna.chars();
            let first = chars.next()?;
            let consumed = match first {
                'C' => {
                    p.push(PItem::Base('I'));
                    1
                }
                'F' => {
                    p.push(PItem::Base('C'));
                    1
                }
                'P' => {
                    p.push(PItem::Base('F'));
                    1
                }
                'I' => {
                    let second = chars.next()?;
                    let consumed = match second {
                        'C' => {
                            p.push(PItem::Base('P'));
                            2
                        }
                        'P' => {
                            let (n, consumed) = Dna2Rna::nat(chars)?;
                            p.push(PItem::Skip(n));
                            2 + consumed
                        }
                        'F' => {
                            let _ = chars.next()?;
                            let (s, consumed) = Dna2Rna::consts(chars)?;
                            p.push(PItem::Search(s));
                            // yes, 3
                            3 + consumed
                        }
                        'I' => {
                            let third = chars.next()?;
                            let consumed = match third {
                                'P' => {
                                    level = level + 1;
                                    p.push(PItem::Open);
                                    3
                                }
                                'C' | 'F' => {
                                    if level == 0 {
                                        ret = true;
                                    } else {
                                        level = level - 1;
                                        p.push(PItem::Close);
                                    }
                                    3
                                }
                                'I' => {
                                    self.rna.append(self.dna.slice(3..10).into());
                                    10
                                }
                                _ => 0,
                            };
                            consumed
                        }
                        _ => 0,
                    };
                    consumed
                }
                _ => 0,
            };
            if consumed == 0 {
                break;
            }
            log::debug!(
                "consuming dna: {} of available {}",
                consumed,
                self.dna.len_chars()
            );
            self.dna = self.dna.slice(consumed..).into();
        }
        Some(p)
    }

    fn template(&mut self) -> Option<Vec<TItem>> {
        let mut t = vec![];
        let mut ret = false;
        while !ret {
            let mut chars = self.dna.chars();
            let first = chars.next()?;
            let consumed = match first {
                'C' => {
                    t.push(TItem::Base('I'));
                    1
                }
                'F' => {
                    t.push(TItem::Base('C'));
                    1
                }
                'P' => {
                    t.push(TItem::Base('F'));
                    1
                }
                'I' => {
                    let second = chars.next()?;
                    let consumed = match second {
                        'C' => {
                            t.push(TItem::Base('P'));
                            2
                        }
                        'F' | 'P' => {
                            let (l, consumed) = Dna2Rna::nat(chars)?;
                            let next_chars = self.dna.chars_at(2 + consumed);
                            let (n, next_consumed) = Dna2Rna::nat(next_chars)?;
                            t.push(TItem::Ref(n, l));
                            2 + consumed + next_consumed
                        }
                        'I' => {
                            let third = chars.next()?;
                            let consumed = match third {
                                'C' | 'F' => {
                                    ret = true;
                                    3
                                }
                                'P' => {
                                    let (n, consumed) = Dna2Rna::nat(chars)?;
                                    t.push(TItem::RefLen(n));
                                    3 + consumed
                                }
                                'I' => {
                                    self.rna.append(self.dna.slice(3..10).into());
                                    10
                                }
                                _ => 0,
                            };
                            consumed
                        }
                        _ => 0,
                    };
                    consumed
                }
                _ => 0,
            };
            if consumed == 0 {
                break;
            }
            self.dna = self.dna.slice(consumed..).into();
        }
        Some(t)
    }

    fn match_replace(&mut self, pattern: &[PItem], template: &[TItem]) {
        let mut e = vec![];
        let mut c = VecDeque::new();
        let mut chars = self.dna.chars();
        let mut curr = chars.next().unwrap();
        let mut i = 0;
        log::debug!("match/replace: {:?} {:?}", pattern, template);
        for p in pattern {
            log::debug!("pattern: {:?}", p);
            match p {
                PItem::Base(b) => {
                    if curr == *b {
                        curr = chars.next().unwrap();
                        i = i + 1;
                    } else {
                        log::debug!("no match for {}", b);
                        return;
                    }
                }
                PItem::Skip(n) => {
                    if i + *n > self.dna.len_chars() {
                        log::debug!("can't skip {}", *n);
                        return;
                    }
                    for _ in 0..*n {
                        curr = chars.next().unwrap_or('X');
                        i = i + 1;
                    }
                }
                PItem::Search(s) => {
                    let mut n = i;
                    let slen = s.len();
                    let dlen = self.dna.len_chars();
                    loop {
                        if n + slen > dlen {
                            return;
                        }
                        if self.dna.slice(n..n + slen).chars().eq(s.chars()) {
                            i = n;
                            chars = self.dna.chars_at(i);
                            curr = chars.next().unwrap();
                            break;
                        }
                        n = n + 1;
                    }
                }
                PItem::Open => c.push_front(i),
                PItem::Close => {
                    let cval = c.pop_front().expect("malformed pattern");
                    e.push(Rope::from(self.dna.slice(cval..i)));
                }
            }
        }
        log::debug!("dna = dna[{}..]", i);
        self.dna = self.dna.slice(i..).into();
        self.replace(template, e);
    }

    fn replace(&mut self, template: &[TItem], e: Vec<Rope>) {
        log::debug!("replacing {:?} with {:?}", template, e);
        let mut r = Rope::new();
        for t in template {
            match t {
                TItem::Base(b) => {
                    let mut s = String::new();
                    s.push(*b);
                    r.append(Rope::from(s));
                }
                TItem::Ref(n, l) => {
                    if *n < e.len() {
                        r.append(Dna2Rna::protect(*l, &e[*n]));
                    }
                }
                TItem::RefLen(n) => {
                    if *n < e.len() {
                        r.append(Dna2Rna::asnat(e[*n].len_chars()));
                    } else {
                        r.append(Dna2Rna::asnat(0));
                    }
                }
            }
        }
        std::mem::swap(&mut self.dna, &mut r);
        self.dna.append(r);
        log::debug!("new dna: {:?}", self.dna);
    }

    fn protect(l: usize, d: &Rope) -> Rope {
        if l == 0 {
            d.clone()
        } else {
            Dna2Rna::protect(l - 1, &Dna2Rna::quote(d))
        }
    }

    fn quote(d: &Rope) -> Rope {
        let mut ret = String::new();
        for c in d.chars() {
            match c {
                'I' => ret.push('C'),
                'C' => ret.push('F'),
                'F' => ret.push('P'),
                'P' => {
                    ret.push('I');
                    ret.push('C');
                }
                _ => return Rope::new(),
            }
        }
        Rope::from(ret)
    }

    fn asnat(mut n: usize) -> Rope {
        let mut ret = String::new();
        loop {
            if n == 0 {
                break;
            } else if n % 2 == 0 {
                ret.push('I');
                n = n / 2;
            } else {
                ret.push('C');
                n = n / 2;
            }
        }
        Rope::from(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_examples() {
        let mut dna_1 = Dna2Rna::new("IIPIPICPIICICIIFICCIFPPIICCFPC", None);
        dna_1.execute_step();
        assert_eq!(dna_1.dna.to_string(), "PICFC");
        let mut dna_2 = Dna2Rna::new("IIPIPICPIICICIIFICCIFCCCPPIICCFPC", None);
        dna_2.execute_step();
        assert_eq!(dna_2.dna.to_string(), "PIICCFCFFPC");
        let mut dna_3 = Dna2Rna::new("IIPIPIICPIICIICCIICFCFC", None);
        dna_3.execute_step();
        assert_eq!(dna_3.dna.to_string(), "I");
    }
}
