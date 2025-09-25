use clap::CommandFactory;
use clap_complete::{generate_to, shells::Zsh};
// use clap_complete::{Bash, Fish}
use std::{path::PathBuf, fs};
use clap_mangen::Man;

include!("src/cli.rs");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from("target/completions");
    fs::create_dir_all(&out_dir).expect("failed to create completion dir");

    let mut cmd = Cli::command();
    generate_to(Zsh, &mut cmd, "cave", &out_dir).unwrap();

    let out_dir = PathBuf::from("target/man");
    fs::create_dir_all(&out_dir).expect("failed to create man dir");
    let man = Man::new(Cli::command());
    let mut file = fs::File::create(out_dir.join("cave.1")).unwrap();
    man.render(&mut file).unwrap();

    tonic_build::compile_protos("proto/cave_telem.proto")?;
    Ok(())
}