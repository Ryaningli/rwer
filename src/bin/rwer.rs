use clap::Parser;
use rwer::cli::{Cli, EvalInput, build_pipeline};

fn main() {
    let cli = Cli::parse();

    let input = EvalInput {
        reference: cli.reference.clone().unwrap_or_default(),
        hypothesis: cli.hypothesis.clone().unwrap_or_default(),
        character: cli.character,
        alignment: cli.alignment,
        all: cli.all,
    };

    let pipeline = build_pipeline(&cli);
    let pipeline_ref = pipeline.as_deref();
    let result = rwer::cli::process_and_format(&input, pipeline_ref);
    print!("{result}");
}
