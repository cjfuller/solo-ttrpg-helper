use clap::Parser;

use solo_ttrpg_helper::dice::Dice;

#[derive(Debug, clap::Subcommand)]
#[clap(trailing_var_arg = true)]
enum Command {
    #[clap(alias("r"), trailing_var_arg = true)]
    Roll { dice_spec: Vec<String> },
}

#[derive(Debug, clap::Parser)]
struct CLI {
    #[clap(subcommand)]
    subcommand: Command,
}

fn main() {
    let cli = CLI::parse();
    match cli.subcommand {
        Command::Roll { dice_spec } => {
            let s = dice_spec.join(" ");
            let dice: Dice = s.parse().unwrap();
            println!("{}", dice.roll());
        }
    }
}
