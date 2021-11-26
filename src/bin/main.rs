use ron_parser::parse;

fn main() {
    // Parse input arguments
    let args: Vec<String> = std::env::args().collect();
    for arg in args.iter().skip(1) {
        let input = if arg.ends_with(".ron") {
            std::fs::read_to_string(arg).unwrap()
        } else {
            arg.clone()
        };

        match parse(&input) {
            Ok(val) => println!("{:?}", val),
            Err(err) => {
                println!("{:?}", err.partial_parse);
                err.emit();
            }
        }
    }
}
