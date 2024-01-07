use std::{collections::HashSet, fs, io};

use movegen::{
    board::Board,
    cli::{handle_command, split_ignore_quotes, Cli, Command},
};
use clap::Parser;

fn main() {
    let args = Cli::parse();
    let mut board = Board::default();

    match args.command {
        Command::Interactive => loop {
            let mut std_in = String::new();
            io::stdin()
                .read_line(&mut std_in)
                .expect("reading std in should succeed");

            let args_no_newlines = std_in.replace("\n", "");
            let mut args = split_ignore_quotes(&args_no_newlines);
            args.push_front("chess".to_owned());
            match Cli::try_parse_from(args) {
                Ok(args) => match args.command {
                    Command::Exit => break,
                    cmd => handle_command(cmd, &mut board),
                },
                Err(err) => {
                    err.print()
                        .unwrap_or_else(|_| println!("Failed to display parsing error"));
                }
            }
        },
        cmd => handle_command(cmd, &mut board),
    };
}
