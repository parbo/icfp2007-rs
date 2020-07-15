use log;
use ropey::Rope;
use std::collections::VecDeque;

pub trait RnaStore {
    fn store(&mut self, rna: String);
}

pub struct VecRnaStore {
    pub rna: Vec<String>,
}

impl VecRnaStore {
    pub fn new() -> VecRnaStore {
        VecRnaStore { rna: vec![] }
    }
}

impl RnaStore for VecRnaStore {
    fn store(&mut self, rna: String) {
        self.rna.push(rna);
    }
}

pub struct Dna2Rna<'a> {
    dna: Rope,
    rna_store: &'a mut dyn RnaStore,
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

impl<'a> Dna2Rna<'a> {
    pub fn new(rna_store: &'a mut dyn RnaStore) -> Dna2Rna<'a> {
        Dna2Rna {
            dna: Rope::new(),
            rna_store,
        }
    }

    pub fn set_dna_and_prefix(&mut self, dna_str: &str, prefix: Option<&str>) {
        let mut dna = if let Some(p) = prefix {
            Rope::from_str(&p)
        } else {
            Rope::new()
        };
        dna.append(Rope::from_str(dna_str));
        self.dna = dna;
    }

    pub fn execute(&mut self) {
        log::info!("dna is {} bases long", self.dna.len_chars());
        let mut i = 0;
        loop {
            if self.execute_step() {
                break;
            }
            i = i + 1;
            if i % 1000 == 0 {
                log::info!("at step {}, dna: {}", i, self.dna.len_chars());
            }
        }
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
        let n = bits
            .iter()
            .enumerate()
            .map(|(i, b)| b * 2usize.pow(i as u32))
            .sum();
        Some((n, consumed))
    }

    fn consts(mut chars: ropey::iter::Chars) -> (String, usize) {
        let mut s = vec![];
        let mut extra = 0;
        loop {
            if let Some(c) = chars.next() {
                match c {
                    'C' => s.push('I'),
                    'F' => s.push('C'),
                    'P' => s.push('F'),
                    'I' => {
                        if let Some(cc) = chars.next() {
                            if cc == 'C' {
                                extra = extra + 1;
                                s.push('P');
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    _ => {
                        break;
                    }
                }
            } else {
                break;
            }
        }
        let consumed = s.len() + extra;
        let ret = s.into_iter().collect();
        (ret, consumed)
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
                            let (s, consumed) = Dna2Rna::consts(chars);
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
                                    self.rna_store.store(self.dna.slice(3..10).into());
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
            self.dna = self.dna.split_off(consumed);
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
                                    self.rna_store.store(self.dna.slice(3..10).into());
                                    10
                                }
                                _ => panic!(),
                            };
                            consumed
                        }
                        _ => panic!(),
                    };
                    consumed
                }
                _ => panic!(),
            };
            self.dna = self.dna.split_off(consumed);
        }
        Some(t)
    }

    fn match_replace(&mut self, pattern: &[PItem], template: &[TItem]) {
        let mut e = vec![];
        let mut c = VecDeque::new();
        let mut i = 0;
        log::debug!("match/replace: {:?} {:?}", pattern, template);
        for p in pattern {
            log::debug!("pattern: {:?}", p);
            match p {
                PItem::Base(b) => {
                    if self.dna.char(i) == *b {
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
                    i = i + *n;
                }
                PItem::Search(s) => {
                    let mut n = i;
                    let slen = s.len();
                    let dlen = self.dna.len_chars();
                    let hay = self.dna.slice(n..);
                    let mut hay_iter = hay.chars();
                    loop {
                        if n + slen > dlen {
                            return;
                        }
                        let h = hay_iter.clone();
                        if h.take(slen).eq(s.chars()) {
                            log::debug!("found {} at {}", s, n);
                            i = n + slen;
                            break;
                        }
                        hay_iter.next().unwrap();
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
        if i < self.dna.len_chars() {
            self.dna = self.dna.slice(i..).into();
        } else {
            self.dna = Rope::new();
        }
        self.replace(template, e);
    }

    fn replace(&mut self, template: &[TItem], e: Vec<Rope>) {
        log::debug!("replacing {:?} with envs {:?}", template, e.len());
        let mut r = Rope::new();
        let mut bases = String::new();
        for t in template {
            match t {
                TItem::Base(b) => {
                    bases.push(*b);
                }
                TItem::Ref(n, l) => {
                    if bases.len() > 0 {
                        r.append(Rope::from(bases.as_str()));
                        bases.clear();
                    }
                    if *n < e.len() {
                        r.append(Dna2Rna::protect(*l, &e[*n]));
                    }
                }
                TItem::RefLen(n) => {
                    if bases.len() > 0 {
                        r.append(Rope::from(bases.as_str()));
                        bases.clear();
                    }
                    if *n < e.len() {
                        r.append(Dna2Rna::asnat(e[*n].len_chars()));
                    } else {
                        r.append(Dna2Rna::asnat(0));
                    }
                }
            }
        }
        if bases.len() > 0 {
            r.append(Rope::from(bases.as_str()));
            bases.clear();
        }
        std::mem::swap(&mut self.dna, &mut r);
        self.dna.append(r);
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
                ret.push('P');
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

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn pattern() {
        init();
        let mut rna_1 = VecRnaStore::new();
        let mut dna_1 = Dna2Rna::new(&mut rna_1);
        dna_1.set_dna_and_prefix("CIIC", None);
        let pt_1 = dna_1.pattern();
        assert_eq!(pt_1, Some(vec![PItem::Base('I')]));
        let mut rna_2 = VecRnaStore::new();
        let mut dna_2 = Dna2Rna::new(&mut rna_2);
        dna_2.set_dna_and_prefix("IIPIPICPIICICIIF", None);
        let pt_2 = dna_2.pattern();
        assert_eq!(
            pt_2,
            Some(vec![
                PItem::Open,
                PItem::Skip(2),
                PItem::Close,
                PItem::Base('P')
            ])
        );
        let mut rna_3 = VecRnaStore::new();
        let mut dna_3 = Dna2Rna::new(&mut rna_3);
        dna_3.set_dna_and_prefix("IIIICFPICFCIIC", None);
        let pt_3 = dna_3.pattern();
        assert_eq!(rna_3.rna, vec!["ICFPICF".to_string()]);
        assert_eq!(pt_3, Some(vec![PItem::Base('I')]));
    }

    #[test]
    fn nat() {
        init();
        assert_eq!(Dna2Rna::nat(Rope::from_str("P").chars()), Some((0, 1)));
        assert_eq!(Dna2Rna::nat(Rope::from_str("IP").chars()), Some((0, 2)));
        assert_eq!(Dna2Rna::nat(Rope::from_str("FP").chars()), Some((0, 2)));
        assert_eq!(Dna2Rna::nat(Rope::from_str("CP").chars()), Some((1, 2)));
        assert_eq!(Dna2Rna::nat(Rope::from_str("CIP").chars()), Some((1, 3)));
        assert_eq!(Dna2Rna::nat(Rope::from_str("CFP").chars()), Some((1, 3)));
        assert_eq!(Dna2Rna::nat(Rope::from_str("ICP").chars()), Some((2, 3)));
        assert_eq!(Dna2Rna::nat(Rope::from_str("ICP").chars()), Some((2, 3)));
        assert_eq!(
            Dna2Rna::nat(Rope::from_str("IFCICFICFP").chars()),
            Some((148, 10))
        );
    }

    #[test]
    fn consts() {
        init();
        assert_eq!(
            Dna2Rna::consts(Rope::from_str("IP").chars()),
            (String::from(""), 0)
        );
        assert_eq!(
            Dna2Rna::consts(Rope::from_str("IF").chars()),
            (String::from(""), 0)
        );
        assert_eq!(
            Dna2Rna::consts(Rope::from_str("CFIF").chars()),
            (String::from("IC"), 2)
        );
        assert_eq!(
            Dna2Rna::consts(Rope::from_str("ICFPICFP").chars()),
            (String::from("PCFPCF"), 8)
        );
    }

    #[test]
    fn asnat() {
        init();
        assert_eq!(Dna2Rna::asnat(0).to_string(), "P");
        assert_eq!(Dna2Rna::asnat(1).to_string(), "CP");
        assert_eq!(Dna2Rna::asnat(2).to_string(), "ICP");
        assert_eq!(Dna2Rna::asnat(3).to_string(), "CCP");
        assert_eq!(Dna2Rna::asnat(4).to_string(), "IICP");
    }

    #[test]
    fn asnat_to_nat() {
        init();
        for i in 0..100 {
            log::info!("i: {}", i);
            let r = Dna2Rna::nat(Dna2Rna::asnat(i).chars());
            assert_ne!(r, None);
            if let Some((n, _)) = r {
                assert_eq!(n, i);
            }
        }
    }

    #[test]
    fn quote() {
        init();
        assert_eq!(Dna2Rna::quote(&Rope::from_str("ICFP")).to_string(), "CFPIC")
    }

    #[test]
    fn protect() {
        init();
        assert_eq!(
            Dna2Rna::protect(3, &Rope::from_str("ICFP")).to_string(),
            "PICCFFP"
        )
    }

    #[test]
    fn match_replace() {
        init();
        let mut rna = VecRnaStore::new();
        let mut dna = Dna2Rna::new(&mut rna);
        dna.set_dna_and_prefix("IIIIIIIIIIICFPFF", None);
        dna.match_replace(
            &[
                PItem::Base('I'),
                PItem::Base('I'),
                PItem::Search("ICFP".into()),
                PItem::Base('F'),
            ],
            &[TItem::Base('C'), TItem::Base('P')],
        );
        assert_eq!(dna.dna.to_string(), "CPF");
    }

    #[test]
    fn complete_step() {
        init();
        let mut rna_1 = VecRnaStore::new();
        let mut dna_1 = Dna2Rna::new(&mut rna_1);
        dna_1.set_dna_and_prefix("IIPIPICPIICICIIFICCIFPPIICCFPC", None);
        dna_1.execute_step();
        assert_eq!(dna_1.dna.to_string(), "PICFC");
        let mut rna_2 = VecRnaStore::new();
        let mut dna_2 = Dna2Rna::new(&mut rna_2);
        dna_2.set_dna_and_prefix("IIPIPICPIICICIIFICCIFCCCPPIICCFPC", None);
        dna_2.execute_step();
        assert_eq!(dna_2.dna.to_string(), "PIICCFCFFPC");
        let mut rna_3 = VecRnaStore::new();
        let mut dna_3 = Dna2Rna::new(&mut rna_3);
        dna_3.set_dna_and_prefix("IIPIPIICPIICIICCIICFCFC", None);
        dna_3.execute_step();
        assert_eq!(dna_3.dna.to_string(), "I");
    }
}
