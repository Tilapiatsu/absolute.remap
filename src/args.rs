use clap::{ArgAction, Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Config {
    #[clap(short, long)]
    /// The path to the device to remap. eg: /dev/input/event6
    pub device: String,

    #[clap(short, long, action = ArgAction::SetFalse, default_value_t = true)]
    /// If enable, all other events is forwarded without any remapping.
    /// default = true
    pub forward: bool,

    #[clap(short, long, default_value_t = 0)]
    /// defines the amount of detail printed in the terminal as debug information |
    /// 0 : Off |
    /// 1 : Minimal |
    /// 2 : Complete |
    pub print_debug: u8,
}
