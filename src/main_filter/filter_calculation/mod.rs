use std::{collections::HashMap, sync::Arc};

use crate::{sub_filter::SubFilter, Label, SubLabel, TopLabel};

pub(crate) use self::thread_communicator::ThreadCommunicator;
use self::work::Work;

const NUMBER_OF_THREADS: u8 = 7;

mod label_vec;
mod thread_communicator;
mod work;
mod work_state;

fn calculate_filter_options<L, TL, SL>(manager: Arc<ThreadCommunicator<L, TL, SL>>, id: u8) -> !
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    'infinity_loop: loop {
        let (filter, all_filters, labels) = match manager.get_work(id) {
            Work::FilterLabel {
                filter,
                all_filters,
                labels,
            } => (Some(filter), all_filters, labels),

            Work::MainFilterOptins {
                all_filters,
                labels,
            } => (None, all_filters, labels),

            Work::NothingToDo => continue 'infinity_loop,
        };

        let mut data_counter = 0;
        let mut label_map = HashMap::new();

        'label_chunk_loop: for label_chunk in labels.iter() {
            //filter the chunk
            for filter in all_filters
                .iter()
                .filter(|&f| Some(f) != filter.as_ref().map(|(_i, f)| f))
            {
                if !filter.filter(&label_chunk) {
                    continue 'label_chunk_loop;
                }
            }

            //increase the counter
            data_counter += 1;

            //insert the labels
            for label in label_chunk {
                let label_counter = label_map.entry(label).or_insert(0);
                *label_counter += 1;
            }
        }

        //get the current label to calculate the usefull sub level labels
        let current_label = filter.as_ref().map(|(_i, f)| f.label());

        //get the usefull labels
        //these are those which are in some but not all Trajectories
        let mut usefull_top_level_labels: Vec<L> = Vec::new();
        let mut usefull_sub_level_labels: Vec<L> = Vec::new();

        'label_loop: for (label, counter) in label_map.drain() {
            assert!(counter > 0);
            if counter == data_counter {
                continue 'label_loop;
            }
            match current_label {
                Some(current_label)
                    if current_label.get_top_level_label() == label.get_top_level_label() =>
                {
                    if !usefull_sub_level_labels.iter().any(|usefull_label| {
                        usefull_label.get_sub_level_label() == label.get_sub_level_label()
                    }) {
                        usefull_sub_level_labels.push(label.clone());
                    }
                }

                _ => {
                    if !usefull_top_level_labels.iter().any(|usefull_label| {
                        usefull_label.get_top_level_label() == label.get_top_level_label()
                    }) {
                        usefull_top_level_labels.push(label.clone());
                    }
                }
            }
        }

        usefull_sub_level_labels.sort();
        usefull_top_level_labels.sort();

        if let Some((i, core)) = filter {
            //we had the FilterLabel work
            let filter = SubFilter::from_core_with_label_options(
                core,
                usefull_top_level_labels,
                usefull_sub_level_labels,
            );
            manager.push_finished_filter(i, filter, id);
        } else {
            //we had the MainFilterOptins work
            assert!(usefull_sub_level_labels.is_empty());
            manager.push_main_filter_label_options(usefull_top_level_labels, id)
        }
    }
}
