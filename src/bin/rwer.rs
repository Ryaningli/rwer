use clap::Parser;
use rwer::cli::{Cli, EvalInput, build_pipeline, resolve_inputs};

fn main() {
    let cli = Cli::parse();

    let (reference, hypothesis) = match resolve_inputs(&cli) {
        Ok(vals) => vals,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    let input = EvalInput {
        reference,
        hypothesis,
        character: cli.character,
        alignment: cli.alignment,
        all: cli.all,
    };

    let pipeline = build_pipeline(&cli);
    let pipeline_ref = pipeline.as_deref();
    let result = rwer::cli::process_and_format(&input, pipeline_ref);
    print!("{result}");
}
