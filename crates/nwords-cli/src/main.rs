#![forbid(unsafe_code)]

fn main() {
    let output = nwords_cli::run(std::env::args().skip(1));

    print!("{}", output.stdout);
    eprint!("{}", output.stderr);
    std::process::exit(output.exit_code);
}
