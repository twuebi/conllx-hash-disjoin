use std::collections::HashSet;
use std::hash::Hasher;
use std::io::BufWriter;
use stdinout::{Input, OrExit, Output};

use clap::{App, AppSettings, Arg};
use conllx::io::{ReadSentence, WriteSentence};
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
        .arg(Arg::with_name(REMOVE_SET).help("REMOVE_SET data").index(1))
        .arg(Arg::with_name(KEEP_SET).help("KEEP_SET data").index(2))
        .arg(Arg::with_name(OUTPUT).help("Output").index(3))
        .help(
            "Removes sentences present in <REMOVE_SET> from <KEEP_SET> based on their xxHash. \
             Reads and writes conllx.",
        )
        .get_matches();

    let remove_set = matches.value_of(REMOVE_SET).map(ToOwned::to_owned);
    let keep_set = matches.value_of(KEEP_SET).map(ToOwned::to_owned);
    let output = matches.value_of(OUTPUT).map(ToOwned::to_owned);

    let keep_set = Input::from(keep_set);
    let remove_set = Input::from(remove_set);
    let output = Output::from(output);

    let keep_set =
        conllx::io::Reader::new(keep_set.buf_read().or_exit("Failed opening keep input.", 1));
    let remove_set = conllx::io::Reader::new(
        remove_set
            .buf_read()
            .or_exit("Failed opening remove input.", 1),
    );
    let output = output.write().or_exit("Failed opeining output", 1);
    let mut output = conllx::io::Writer::new(BufWriter::new(output));

    let hash_builder = RandomXxHashBuilder64::default();

    let mut n_sents = 0;
    let remove_set = remove_set
        .sentences()
        .map(|sent| {
            let sent = sent.or_exit("Cannot parse sent", 1);
            n_sents += 1;
            hash_sentence(&sent, &hash_builder)
        })
        .collect::<HashSet<u64>>();
    eprintln!(
        "Done collecting {} unique hashes from {} sentences!!",
        remove_set.len(),
        n_sents
    );

    let mut n_sents = 0;
    let keep_set = keep_set
        .sentences()
        .map(|sent| {
            let sent = sent.or_exit("Cannot parse sent", 1);
            let hash = hash_sentence(&sent, &hash_builder);
            n_sents += 1;
            if !remove_set.contains(&hash) {
                output.write_sentence(&sent).or_exit("failed writing", 1);
            }
            hash
        })
        .collect::<HashSet<u64>>();
    eprintln!(
        "Encountered {} unique hashes in {} sentences.",
        keep_set.len(),
        n_sents
    );
}

fn hash_sentence<T>(sentence: &conllx::graph::Sentence, hash_builder: &T) -> u64
where
    T: BuildHasher,
{
    let mut hasher = hash_builder.build_hasher();
    for token in sentence.iter().skip(1) {
        hasher.write(token.token().expect("token").form().as_bytes());
    }
    hasher.finish()
}
