use clap::{App, Arg};
use std::fs;

fn main() {
    let matches = App::new("dna2rna-cli")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let filename = matches.value_of("INPUT").unwrap();

    if let Ok(dna) = fs::read_to_string(filename) {
        let mut d = dna2rna::Dna2Rna::new(&dna);
	d.execute();
    } else {
        println!("error reading file {}", filename);
    }
}
