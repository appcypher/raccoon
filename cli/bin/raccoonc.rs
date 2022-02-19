// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use clap::{Parser};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {}

fn main() {
    let _ = Args::parse();
    println!("raccoonc: The Raccoon Compiler CLI");
}
