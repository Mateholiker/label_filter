use crate::{sub_filter::SubFilter, Label, SubLabel, TopLabel};

use super::{work::Work, NUMBER_OF_THREADS};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ThreadState {
    Finished,
    Working,
    Outdated,
}

pub(crate) enum WorkState<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    NothingToDo,
    Working {
        thread_state: [ThreadState; NUMBER_OF_THREADS as usize],

        all_filters_len: usize,
        unfinished_work: Vec<Work<L, TL, SL>>,

        finished_filters: Vec<(usize, SubFilter<L, TL, SL>)>,
        finished_main_filter_label: Vec<L>,
    },
    Finished {
        filter: Vec<SubFilter<L, TL, SL>>,
        main_filter_label_options: Vec<L>,
    },
}
