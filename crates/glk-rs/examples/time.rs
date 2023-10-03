mod util;
use util::win::SimpleWindow;

use rglk::prelude::*;

fn main() {
    Glk::<SimpleWindow>::start(|glk| {
        let now = glk.current_time();
        println!("now = {now:?}");

        let utc = glk.time_to_date_utc(&now);
        println!("utc = {utc:?}");

        let local = glk.time_to_date_local(&now);
        println!("local = {local:?}");

        let simple = glk.current_simple_time(1);
        println!("simple, factor 1: {simple}");
        println!(
            "                : {:?}",
            glk.simple_time_to_date_utc(simple, 1)
        );
        println!(
            "                : {:?}",
            glk.simple_time_to_date_local(simple, 1)
        );

        let simple = glk.current_simple_time(3600);
        println!("simple, factor 3600: {simple}");
        println!(
            "                : {:?}",
            glk.simple_time_to_date_utc(simple, 3600)
        );
        println!(
            "                : {:?}",
            glk.simple_time_to_date_local(simple, 3600)
        );
    });
}
