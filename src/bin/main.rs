use ron_parser::{load, parse};

fn main() {
    // Parse input arguments
    let args: Vec<String> = std::env::args().collect();
    for arg in args.iter().skip(1) {
        if arg.ends_with(".ron") {
            let parse = load(arg).unwrap();
            println!("{}", parse.value.fmt_as_rust());
            parse.emit();
        } else {
            match parse(arg, Some("<stdin>")) {
                Ok(val) => println!("{}", val.fmt_as_rust()),
                Err(err) => {
                    println!("{}", err.value.fmt_as_rust());
                    err.emit();
                }
            }
        }
    }
}
