use blorb::BlorbReader;

fn main() {
    let filename = std::env::args().skip(1).next().unwrap();
    println!("reading file \"{filename}\"");
    let filedata = std::fs::read(filename).expect("unable to open file");
    match BlorbReader::new(filedata) {
        Ok(b) => b.dump_rsrc_usage(),
        Err(e) => println!("read error: {e}"),
    }
}
