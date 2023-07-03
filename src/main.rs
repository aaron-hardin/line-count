use std::fmt::Write;
use std::fs::{self, File};
use std::io::{BufReader, BufRead};

use clap::Parser;

type ErrBox = Box<dyn std::error::Error + Send + Sync>;

/// A CLI for counting lines in files
#[derive(Debug, Parser)]
#[clap(name = "line-count")]
#[clap(about = "A CLI for counting lines in files", long_about = None)]
struct Cli {
    /// Sets the directory to count lines in files
    #[clap(short, long)]
    directory: String,
    /// If true, sorts by count desc, otherwise sorts by count asc
    #[clap(long)]
    sort_desc: bool,
}

#[tokio::main]
async fn main() -> Result<(), ErrBox> {
    let args = Cli::parse();
    let paths = fs::read_dir(args.directory).unwrap();

    let mut tasks = vec![];
    let mut counts: Vec<(usize, String)> = vec![];
    for path in paths {
        tasks.push(tokio::spawn(async move {
            let path = path.unwrap().path();
            let file_name = path.display().to_string();
            let file = BufReader::new(File::open(path).expect("Unable to open file"));
            (file.lines().into_iter().count(), file_name)
        }));
    }

    for task in tasks {
        counts.push(task.await.unwrap());
    }

    if args.sort_desc {
        counts.sort_by(|a, b| b.0.cmp(&a.0));
    } else {
        counts.sort_by_key(|c| c.0);
    }

    let mut output = String::new();
    for (count, name) in counts {
        writeln!(&mut output, "{count}: {name}").unwrap();
    }
    println!("{output}");

    Ok(())
}
