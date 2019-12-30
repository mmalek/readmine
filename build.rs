use clap::Shell;
use std::env;

mod cli {
    include!("src/cli.rs");
}

fn main() {
    if let Some(outdir) = env::var_os("OUT_DIR") {
        let mut app = cli::build_cli();
        let bin_name = "readmine";
        app.gen_completions(bin_name, Shell::Bash, outdir.clone());
        app.gen_completions(bin_name, Shell::Fish, outdir.clone());
        app.gen_completions(bin_name, Shell::Zsh, outdir.clone());
        app.gen_completions(bin_name, Shell::PowerShell, outdir.clone());
        app.gen_completions(bin_name, Shell::Elvish, outdir.clone());
    }
}
