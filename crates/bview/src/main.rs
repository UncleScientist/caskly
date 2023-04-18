use blorb::{chunk::BlorbChunk, error::BlorbError, BlorbReader};

fn main() {
    let filename = std::env::args().nth(1).unwrap();
    println!("reading file \"{filename}\"");
    let filedata = std::fs::read(filename).expect("unable to open file");
    let blorb = BlorbReader::new(filedata);
    if let Ok(blorb) = blorb {
        blorb.dump_rsrc_usage();
        for chunk in blorb.iter() {
            match chunk {
                Ok(chunk) => match TryInto::<BlorbChunk>::try_into(&chunk) {
                    Ok(chunk) => println!("{chunk:?}"),
                    Err(e) if e == BlorbError::ConversionFailed => println!("{chunk:?}"),
                    Err(e) => panic!("interpration failed - {e}"),
                },
                Err(e) => panic!("invalid chunk - {e}"),
            }
        }
    } else {
        let err = blorb.unwrap_err();
        println!("read error: {err}");
        std::process::exit(1);
    }
}
