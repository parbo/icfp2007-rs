use log;
use std::collections::VecDeque;

type DNA = im_rc::Vector<char>;

fn dna_from_str(s: &str) -> DNA {
    let mut d = DNA::new();
    for c in s.chars() {
        d.push_back(c);
    }
    d
}

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
    dna: DNA,
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
            dna: im_rc::Vector::new(),
            rna_store,
        }
    }

    pub fn set_dna_and_prefix(&mut self, dna_str: &str, prefix: Option<&str>) {
        let mut v = im_rc::Vector::new();
        //	v.reserve(dna_str.len() + 100);
        if let Some(p) = prefix {
            v.append(dna_from_str(p));
        }
        v.append(dna_from_str(dna_str));
        self.dna = v;
    }

    pub fn execute(&mut self) {
        log::info!("dna is {} bases long", self.dna.len());
        //        log::info!("dna starts with {}", self.dna.slice(0..32).to_string());
        let mut i = 0;
        loop {
            if self.execute_step() {
                break;
            }
            i = i + 1;
            if i % 1000 == 0 {
                log::info!("at step {}, dna: {}", i, self.dna.len());
                //                log::info!("dna starts with {}", self.dna.slice(0..32).to_string());
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

    fn nat(mut chars: im_rc::vector::Iter<char>) -> Option<(usize, usize)> {
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

    fn consts(mut chars: im_rc::vector::Iter<char>) -> (String, usize) {
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
                            if *cc == 'C' {
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
        log::debug!("pattern: dna available {}", self.dna.len());
        while !ret {
            let mut chars = self.dna.iter();
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
                                    let mut s = String::new();
                                    log::debug!("produce rna, dna available {}", self.dna.len());
                                    for c in self.dna.iter().skip(3).take(7) {
                                        s.push(*c);
                                    }
                                    log::debug!("produce rna, dna available {}", self.dna.len());
                                    self.rna_store.store(s);
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
                self.dna.len()
            );
            self.dna = self.dna.split_off(consumed);
        }
        Some(p)
    }

    fn template(&mut self) -> Option<Vec<TItem>> {
        let mut t = vec![];
        let mut ret = false;
        while !ret {
            let mut chars = self.dna.iter();
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
                            let next = self.dna.clone().split_off(2 + consumed);
                            let next_chars = next.iter();
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
                                    let mut s = String::new();
                                    for c in self.dna.iter().skip(3).take(7) {
                                        s.push(*c);
                                    }
                                    self.rna_store.store(s);
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
                    if self.dna[i] == *b {
                        i = i + 1;
                    } else {
                        log::debug!("no match for {}", b);
                        return;
                    }
                }
                PItem::Skip(n) => {
                    if i + *n > self.dna.len() {
                        log::debug!("can't skip {}", *n);
                        return;
                    }
                    i = i + *n;
                }
                PItem::Search(s) => {
                    let mut n = i;
                    let slen = s.len();
                    let dlen = self.dna.len();
                    loop {
                        if n + slen > dlen {
                            return;
                        }
                        if self.dna.iter().skip(n).take(slen).map(|x| *x).eq(s.chars()) {
                            log::debug!("found {} at {}", s, n);
                            i = n + slen;
                            break;
                        }
                        n = n + 1;
                    }
                }
                PItem::Open => c.push_front(i),
                PItem::Close => {
                    let cval = c.pop_front().expect("malformed pattern");
                    e.push(self.dna.clone().slice(cval..i));
                }
            }
        }
        log::debug!("dna = dna[{}..]", i);
        if i < self.dna.len() {
            self.dna = self.dna.split_off(i);
        } else {
            self.dna.clear();
        }
        self.replace(template, e);
    }

    fn replace(&mut self, template: &[TItem], e: Vec<DNA>) {
        log::debug!("replacing {:?} with envs {:?}", template, e.len());
        let mut r = im_rc::Vector::new();
        for t in template {
            match t {
                TItem::Base(b) => {
                    r.push_back(*b);
                }
                TItem::Ref(n, l) => {
                    if *n < e.len() {
                        r.append(Dna2Rna::protect(*l, &e[*n]));
                    }
                }
                TItem::RefLen(n) => {
                    if *n < e.len() {
                        r.append(Dna2Rna::asnat(e[*n].len()));
                    } else {
                        r.append(Dna2Rna::asnat(0));
                    }
                }
            }
        }
        std::mem::swap(&mut self.dna, &mut r);
        self.dna.append(r);
    }

    fn protect(l: usize, d: &DNA) -> DNA {
        if l == 0 {
            d.clone()
        } else {
            Dna2Rna::protect(l - 1, &Dna2Rna::quote(d))
        }
    }

    fn quote(d: &DNA) -> DNA {
        let mut ret = im_rc::Vector::new();
        for c in d.iter() {
            match c {
                'I' => ret.push_back('C'),
                'C' => ret.push_back('F'),
                'F' => ret.push_back('P'),
                'P' => {
                    ret.push_back('I');
                    ret.push_back('C');
                }
                _ => return im_rc::Vector::new(),
            }
        }
        ret
    }

    fn asnat(mut n: usize) -> DNA {
        let mut ret = im_rc::Vector::new();
        loop {
            if n == 0 {
                ret.push_back('P');
                break;
            } else if n % 2 == 0 {
                ret.push_back('I');
                n = n / 2;
            } else {
                ret.push_back('C');
                n = n / 2;
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn str_from_dna(d: &DNA) -> String {
        let mut s = String::new();
        for c in d.iter() {
            s.push(*c);
        }
        s
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
        assert_eq!(Dna2Rna::nat(dna_from_str("P").iter()), Some((0, 1)));
        assert_eq!(Dna2Rna::nat(dna_from_str("IP").iter()), Some((0, 2)));
        assert_eq!(Dna2Rna::nat(dna_from_str("FP").iter()), Some((0, 2)));
        assert_eq!(Dna2Rna::nat(dna_from_str("CP").iter()), Some((1, 2)));
        assert_eq!(Dna2Rna::nat(dna_from_str("CIP").iter()), Some((1, 3)));
        assert_eq!(Dna2Rna::nat(dna_from_str("CFP").iter()), Some((1, 3)));
        assert_eq!(Dna2Rna::nat(dna_from_str("ICP").iter()), Some((2, 3)));
        assert_eq!(Dna2Rna::nat(dna_from_str("ICP").iter()), Some((2, 3)));
        assert_eq!(
            Dna2Rna::nat(dna_from_str("IFCICFICFP").iter()),
            Some((148, 10))
        );
    }

    #[test]
    fn consts() {
        init();
        assert_eq!(
            Dna2Rna::consts(dna_from_str("IP").iter()),
            (String::from(""), 0)
        );
        assert_eq!(
            Dna2Rna::consts(dna_from_str("IF").iter()),
            (String::from(""), 0)
        );
        assert_eq!(
            Dna2Rna::consts(dna_from_str("CFIF").iter()),
            (String::from("IC"), 2)
        );
        assert_eq!(
            Dna2Rna::consts(dna_from_str("ICFPICFP").iter()),
            (String::from("PCFPCF"), 8)
        );
    }

    #[test]
    fn asnat() {
        init();
        assert_eq!(str_from_dna(&Dna2Rna::asnat(0)), "P");
        assert_eq!(str_from_dna(&Dna2Rna::asnat(1)), "CP");
        assert_eq!(str_from_dna(&Dna2Rna::asnat(2)), "ICP");
        assert_eq!(str_from_dna(&Dna2Rna::asnat(3)), "CCP");
        assert_eq!(str_from_dna(&Dna2Rna::asnat(4)), "IICP");
    }

    #[test]
    fn asnat_to_nat() {
        init();
        for i in 0..100 {
            log::info!("i: {}", i);
            let r = Dna2Rna::nat(Dna2Rna::asnat(i).iter());
            assert_ne!(r, None);
            if let Some((n, _)) = r {
                assert_eq!(n, i);
            }
        }
    }

    #[test]
    fn quote() {
        init();
        assert_eq!(
            str_from_dna(&Dna2Rna::quote(&dna_from_str("ICFP"))),
            "CFPIC"
        )
    }

    #[test]
    fn protect() {
        init();
        assert_eq!(
            str_from_dna(&Dna2Rna::protect(3, &dna_from_str("ICFP"))),
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
        assert_eq!(str_from_dna(&dna.dna), "CPF");
    }

    #[test]
    fn complete_step() {
        init();
        let mut rna_1 = VecRnaStore::new();
        let mut dna_1 = Dna2Rna::new(&mut rna_1);
        dna_1.set_dna_and_prefix("IIPIPICPIICICIIFICCIFPPIICCFPC", None);
        dna_1.execute_step();
        assert_eq!(str_from_dna(&dna_1.dna), "PICFC");
        let mut rna_2 = VecRnaStore::new();
        let mut dna_2 = Dna2Rna::new(&mut rna_2);
        dna_2.set_dna_and_prefix("IIPIPICPIICICIIFICCIFCCCPPIICCFPC", None);
        dna_2.execute_step();
        assert_eq!(str_from_dna(&dna_2.dna), "PIICCFCFFPC");
        let mut rna_3 = VecRnaStore::new();
        let mut dna_3 = Dna2Rna::new(&mut rna_3);
        dna_3.set_dna_and_prefix("IIPIPIICPIICIICCIICFCFC", None);
        dna_3.execute_step();
        assert_eq!(str_from_dna(&dna_3.dna), "I");
    }
}
