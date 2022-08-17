use std::{
    mem::replace,
    ops::{Deref, DerefMut},
    sync::{Arc, Condvar, Mutex},
    thread::spawn,
};

use crate::{sub_filter::SubFilter, Label, LabeledData, SubLabel, TopLabel};

use super::{
    calculate_filter_options,
    label_vec::LabelVec,
    work::Work,
    work_state::{ThreadState, WorkState},
    NUMBER_OF_THREADS,
};

pub(crate) struct ThreadCommunicator<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    work_state: Mutex<WorkState<L, TL, SL>>,
    condvar: Condvar,
}

impl<L, TL, SL> ThreadCommunicator<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    pub(crate) fn new() -> Arc<Self> {
        let tc = Self {
            work_state: Mutex::new(WorkState::NothingToDo),
            condvar: Condvar::new(),
        };
        let arc_tc = Arc::new(tc);
        for id in 0..NUMBER_OF_THREADS {
            let manager = arc_tc.clone();
            spawn(move || calculate_filter_options(manager, id));
        }

        arc_tc
    }

    pub(crate) fn get_work(&self, id: u8) -> Work<L, TL, SL> {
        let current_work_state = self.work_state.lock().unwrap();
        let mut current_work_state = self
            .condvar
            .wait_while(current_work_state, |work_state| {
                !matches!(work_state, WorkState::Working { .. })
            })
            .unwrap();

        if let WorkState::Working {
            thread_state,
            unfinished_work,

            finished_filters,
            finished_main_filter_label,
            ..
        } = current_work_state.deref_mut()
        {
            let work = unfinished_work.pop().unwrap_or(Work::NothingToDo);
            thread_state[id as usize] = if matches!(work, Work::NothingToDo) {
                ThreadState::Finished
            } else {
                ThreadState::Working
            };
            if thread_state
                .iter()
                .all(|state| matches!(state, ThreadState::Finished))
            {
                let mut filter = replace(finished_filters, Vec::with_capacity(0));
                let main_filter_label_options =
                    replace(finished_main_filter_label, Vec::with_capacity(0));

                filter.sort_by_key(|(i, _f)| *i);
                let filter = filter.drain(..).map(|(_i, f)| f).collect();

                let new_state = WorkState::Finished {
                    filter,
                    main_filter_label_options,
                };
                *current_work_state = new_state;
            }
            work
        } else {
            unreachable!()
        }
    }

    pub(crate) fn push_main_filter_label_options(&self, main_filter_label_options: Vec<L>, id: u8) {
        let mut current_work_state = self.work_state.lock().unwrap();
        if let WorkState::Working {
            thread_state,
            finished_main_filter_label,
            ..
        } = current_work_state.deref_mut()
        {
            if matches!(thread_state[id as usize], ThreadState::Working) {
                *finished_main_filter_label = main_filter_label_options;
            }
        } else {
            unreachable!()
        }
    }

    pub(crate) fn push_finished_filter(&self, index: usize, filter: SubFilter<L, TL, SL>, id: u8) {
        let mut current_work_state = self.work_state.lock().unwrap();
        if let WorkState::Working {
            thread_state,
            finished_filters,
            ..
        } = current_work_state.deref_mut()
        {
            if matches!(thread_state[id as usize], ThreadState::Working) {
                finished_filters.push((index, filter));
            }
        } else {
            unreachable!()
        }
    }

    pub(crate) fn start<D: LabeledData<L, TL, SL>>(
        &self,
        data: &[D],
        filter: &[SubFilter<L, TL, SL>],
    ) {
        let labels: Arc<LabelVec<_, _, _>> = Arc::new(data.into());
        let all_filters: Arc<Vec<_>> = Arc::new(filter.iter().map(|f| f.clone_core()).collect());

        let mut unfinished_work: Vec<_> = all_filters
            .iter()
            .cloned()
            .enumerate()
            .map(|f| Work::FilterLabel {
                filter: f,
                all_filters: all_filters.clone(),
                labels: labels.clone(),
            })
            .collect();
        unfinished_work.push(Work::MainFilterOptins {
            all_filters: all_filters.clone(),
            labels,
        });

        let new_work_state = WorkState::Working {
            thread_state: [ThreadState::Outdated; NUMBER_OF_THREADS as usize],

            all_filters_len: all_filters.len(),
            unfinished_work,

            finished_filters: Vec::new(),
            finished_main_filter_label: Vec::new(),
        };

        let mut current_work_state = self.work_state.lock().unwrap();
        *current_work_state = new_work_state;
        self.condvar.notify_all();
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn try_get_finished(&self) -> Option<(Vec<SubFilter<L, TL, SL>>, Vec<L>)> {
        let mut current_work_state = self.work_state.lock().unwrap();
        if matches!(current_work_state.deref(), WorkState::Finished { .. }) {
            if let WorkState::Finished {
                filter,
                main_filter_label_options,
            } = replace(current_work_state.deref_mut(), WorkState::NothingToDo)
            {
                Some((filter, main_filter_label_options))
            } else {
                unreachable!()
            }
        } else {
            None
        }
    }

    ///returns a progress (a, b) which means a/b
    pub(crate) fn get_progress(&self) -> (usize, usize) {
        let current_work_state = self.work_state.lock().unwrap();
        let state = current_work_state.deref();
        match state {
            WorkState::Working {
                all_filters_len,
                thread_state,
                unfinished_work,
                ..
            } => {
                let b = all_filters_len + 1;
                let a = b
                    - (unfinished_work.len()
                        + thread_state
                            .iter()
                            .filter(|s| matches!(s, ThreadState::Working))
                            .count());

                (a, b)
            }

            WorkState::Finished { filter, .. } => {
                let a = filter.len() + 1;
                (a, a)
            }

            WorkState::NothingToDo => (1, 1),
        }
    }
}
