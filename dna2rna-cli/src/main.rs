use clap::{App, Arg};
use log;
use std::fs;

fn main() {
    env_logger::init();

    let matches = App::new("dna2rna-cli")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("PREFIX")
                .short("p")
                .long("prefix")
                .takes_value(true)
                .help("Sets the prefix to use"),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("Sets the output file name"),
        )
        .get_matches();

    log::debug!("matches: {:?}", matches);

    let filename = matches.value_of("INPUT").unwrap();
    let prefix = matches.value_of("PREFIX");
    let output = matches.value_of("OUTPUT");

    if let Ok(dna) = fs::read_to_string(filename) {
        log::info!("prefix: {:?}", prefix);
        let mut d = dna2rna::Dna2Rna::new(&dna, prefix);
        d.execute();
        if let Some(out_filename) = output {
            let buf = fs::File::create(out_filename).unwrap();
            d.save_rna(buf).unwrap();
        }
    } else {
        log::error!("error reading file {}", filename);
    }
}
