use std::io::{self, stdin, BufRead, BufWriter, Write, BufReader};
use std::collections::HashSet;
use std::hash::Hasher;
use stdinout::{Input, OrExit, Output};

use clap::{App, AppSettings, Arg};
use core::hash::BuildHasher;
use twox_hash::RandomXxHashBuilder64;

static KEEP_SET: &str = "KEEP_SET";
static REMOVE_SET: &str = "REMOVE_SET";
static OUTPUT: &str = "OUTPUT";

static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
];

fn main() {
    let matches = App::new("conllx-hash-disjoin")
        .settings(DEFAULT_CLAP_SETTINGS)
        .arg(Arg::with_name(OUTPUT).help("Output").index(1))
        .help(
            "Removes sentences present in <REMOVE_SET> from <KEEP_SET> based on their xxHash. \
             Reads and writes conllx.",
        )
        .get_matches();

    let output = matches.value_of(OUTPUT).map(ToOwned::to_owned);

    let output = Output::from(output);

    let output = output.write().or_exit("Failed opeining output", 1);
    let mut output = BufWriter::new(output);

    let hash_builder = RandomXxHashBuilder64::default();

    let mut n_sents = 0;
    let stdin = stdin();
    let reader = stdin.lock();
    let mut remove_set = HashSet::new();

    for line in reader.lines() {
        let line = line.or_exit("Cannot read line", 1);
        let hash = hash_sentence(&line, &hash_builder);
        n_sents += 1;
        if !remove_set.contains(&hash) {
            remove_set.insert(hash);
            write!(output,"{}\n", line);
        }
    }
    
    eprintln!(
        "Encountered {} unique hashes in {} sentences.",
        remove_set.len(),
        n_sents
    );
}

fn hash_sentence<T>(sentence: &str, hash_builder: &T) -> u64
where
    T: BuildHasher,
{
    let mut hasher = hash_builder.build_hasher();
    hasher.write(sentence.as_bytes());
    hasher.finish()
}
