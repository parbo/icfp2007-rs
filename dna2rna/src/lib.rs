use ropey::Rope;

pub struct Dna2Rna {
    dna: Rope,
    rna: Rope,
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
    pub fn new(dna: &str) -> Dna2Rna {
        Dna2Rna {
            dna: Rope::from_str(dna),
            rna: Rope::new(),
        }
    }

    pub fn execute(&mut self) {
        println!("dna is {} bases long", self.dna.len_chars());
        if let Some(p) = self.pattern() {
            println!("pattern: {:?}", p);
            if let Some(t) = self.template() {
                println!("template: {:?}", t);
                self.match_replace(&p, &t);
            }
        }
    }

    fn nat(mut chars: ropey::iter::Chars) -> Option<(usize, usize)> {
        let mut bits = vec![];
        loop {
            let c = chars.next()?;
            match c {
                'P' => break,
                'I' | 'F' => bits.push(0),
                'C' => bits.push(1),
                _ => panic!(),
            }
        }
        let mut n = 0;
        for b in &bits {
            n = n * 2 + b
        }
        Some((n, bits.len()))
    }

    fn consts(mut chars: ropey::iter::Chars) -> Option<(String, usize)> {
        let mut s = String::new();
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
                _ => panic!(),
            }
        }
        let consumed = s.len() + extra;
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
                            t.push(TItem::Base('I'));
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

    fn match_replace(&mut self, _pattern: &[PItem], _template: &[TItem]) {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
