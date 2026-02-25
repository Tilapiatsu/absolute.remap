use anyhow::Result;
use colored::*;
use log::LevelFilter;
use simplelog::{ColorChoice, Config as TermConfig, TermLogger, TerminalMode};
use std::process;
mod args;

use args::Config;
use clap::Parser;

mod remap;
mod state_machine;
use remap::remap_evdev::remap_evdev;

fn main() -> Result<()> {
    let config: Config = Config::parse();
    println!("{:?}", config);

    let trace = match config.print_debug {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        2 => LevelFilter::Trace,
        _ => {
            eprintln!(
                "{}",
                "--print_debug parameter incorect. It should be 0, 1 or 2.".red()
            );
            process::exit(1);
        }
    };
    TermLogger::init(
        trace,
        TermConfig::default(),
        TerminalMode::Stdout,
        ColorChoice::Auto,
    );
    remap_evdev(
        &config.device,
        &{ config.print_debug == 2 },
        &config.forward,
    )
}
