#![allow(dead_code, unused_imports)]

mod executor;
mod file;
mod math;
// mod windowing;

use std::time::Instant;

use color_eyre::owo_colors::OwoColorize;
use color_eyre::Result as EyreResult;

fn main() -> EyreResult<()> {
    color_eyre::install()?;

    let start = Instant::now();

    // windowing::font_main();
    // windowing::run_app()?;
    // file::from_web();
    // executor::evaluator::main()?;

    println!(
        "{}, took {:?}",
        "DONE".bold().green(),
        start.elapsed().blue()
    );

    Ok(())
}
