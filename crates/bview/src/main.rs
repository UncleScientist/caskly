use blorb::BlorbReader;

fn main() {
    let filename = std::env::args().skip(1).next().unwrap();
    println!("reading file \"{filename}\"");
    let filedata = std::fs::read(filename).expect("unable to open file");
    let blorb = BlorbReader::new(filedata);
    if let Ok(blorb) = blorb {
        blorb.dump_rsrc_usage();
        for chunk in blorb.iter() {
            println!("{chunk:?}");
        }
    } else {
        let err = blorb.unwrap_err();
        println!("read error: {err}");
        std::process::exit(1);
    }
}
