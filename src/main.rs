use clap::Parser;
mod cli;

fn main() {
    let args = cli::Args::parse();

    match args.total_value {
        Some(value) => println!("Arg {:?}", value),
        None => println!("Not implemeted yet!"),
    }
}
