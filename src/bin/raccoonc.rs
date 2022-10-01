use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {}

fn main() {
    let _ = Args::parse();
    println!("raccoonc: The Raccoon Compiler CLI");
}
