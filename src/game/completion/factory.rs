use super::flagged::{FlaggedCompletionSource, SimpleCompletionSource};
use super::multiplex::MultiplexCompletionSource;
use super::recency::RecencyCompletionSource;
use crate::game::completion::markov::MarkovCompletionSource;
use crate::game::completion::multiplex::word_index::WordIndexWeightedRandomSelector;
use crate::game::completion::{CompletionSource, ProcessFlags};

pub struct GameCompletionsFactory;

fn create_sent_source() -> MultiplexCompletionSource<Box<dyn SimpleCompletionSource>> {
    MultiplexCompletionSource {
        sources: vec![
            Box::new(MarkovCompletionSource::default()),
            Box::new(RecencyCompletionSource::with_max_entries(1000)),
        ],
        selector_factory: Box::new(WordIndexWeightedRandomSelector::with_weights_by_index(
            vec![
                // The markov trie has a max depth of 5; at that point, we start to suspect
                // that it's not a structured command, so we let recency have more weight
                vec![100, 0],
                vec![100, 0],
                vec![100, 0],
                vec![100, 0],
                // After the first few words, still prefer markov, but
                // give recent words a bit of a chance, too
                vec![50, 50],
            ],
        )),
    }
}

impl GameCompletionsFactory {
    pub fn create() -> MultiplexCompletionSource<Box<dyn CompletionSource>> {
        let sent_source =
            FlaggedCompletionSource::accepting_flags(create_sent_source(), ProcessFlags::SENT);

        let received_source = FlaggedCompletionSource::accepting_flags(
            RecencyCompletionSource::default(),
            ProcessFlags::RECEIVED,
        );

        MultiplexCompletionSource {
            sources: vec![Box::new(sent_source), Box::new(received_source)],
            selector_factory: Box::new(WordIndexWeightedRandomSelector::with_weights_by_index(
                vec![
                    // First word? Prefer commandCompletions ALWAYS; We'll still
                    // fallback to output if commandCompletion doesn't have anything
                    vec![100, 0],
                    // Second word? Actually, prefer output a bit
                    // eg: get <thing>; enter <thing>; look <thing>
                    vec![35, 65],
                    // Otherwise, just split it evenly
                    vec![50, 50],
                ],
            )),
        }
    }
}
