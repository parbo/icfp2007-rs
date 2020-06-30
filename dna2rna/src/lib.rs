use ropey::Rope;

pub struct Dna2Rna {
    dna: Rope,
    rna: Rope,
}

impl Dna2Rna {
    pub fn new(dna: &str) -> Dna2Rna {
	Dna2Rna {
	    dna: Rope::from_str(dna),
	    rna: Rope::new(),
	}
    }

    pub fn execute(&self) {
	println!("dna is {} bases long", self.dna.len_chars());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
