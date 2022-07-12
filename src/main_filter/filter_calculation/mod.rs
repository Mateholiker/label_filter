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

        for label_chunk in labels.iter() {
            //filter the chunk
            let mut filtered_out = false;
            'inner: for filter in all_filters
                .iter()
                .filter(|&f| Some(f) != filter.as_ref().map(|(_i, f)| f))
            {
                filtered_out |= !filter.filter(&label_chunk);
                if filtered_out {
                    break 'inner;
                }
            }

            //increment tho data_counter just if the trajectory is not filtered out
            data_counter += !filtered_out as u32;

            //insert the labels
            for label in label_chunk {
                let not_filtered_out_counter = label_map.entry(label).or_insert(0);
                *not_filtered_out_counter += !filtered_out as u32;
            }
        }

        //get the current label to calculate the usefull sub level labels
        let current_label = filter.as_ref().map(|(_i, f)| f.label());

        //get the usefull labels
        //these are those which are in some but not all Trajectories
        let mut usefull_top_level_labels: Vec<L> = Vec::new();
        let mut usefull_sub_level_labels: Vec<L> = Vec::new();

        let mut useless_top_level_labels: Vec<L> = Vec::new();
        let mut useless_sub_level_labels: Vec<L> = Vec::new();

        for (label, not_filtered_out_counter) in label_map.drain() {
            match current_label {
                Some(current_label)
                    if current_label.get_top_level_label() == label.get_top_level_label() =>
                {
                    if not_filtered_out_counter == 0 || not_filtered_out_counter == data_counter {
                        if !useless_sub_level_labels.iter().any(|useless_label| {
                            useless_label.get_sub_level_label() == label.get_sub_level_label()
                        }) {
                            useless_sub_level_labels.push(label.clone());
                        }
                    } else if !usefull_sub_level_labels.iter().any(|usefull_label| {
                        usefull_label.get_sub_level_label() == label.get_sub_level_label()
                    }) {
                        usefull_sub_level_labels.push(label.clone());
                    }
                }

                _ => {
                    let is_in_useless = useless_top_level_labels.iter().any(|useless_label| {
                        useless_label.get_top_level_label() == label.get_top_level_label()
                    });

                    let is_in_useful = usefull_top_level_labels.iter().any(|usefull_label| {
                        usefull_label.get_top_level_label() == label.get_top_level_label()
                    });

                    if not_filtered_out_counter == 0 || not_filtered_out_counter == data_counter {
                        if !is_in_useful && !is_in_useless {
                            useless_top_level_labels.push(label.clone());
                        }
                    } else if !is_in_useful {
                        usefull_top_level_labels.push(label.clone());
                    }
                }
            }
        }

        usefull_top_level_labels.sort();

        //it is possible that we have added a label to useless top labels and then added it to usefull top labels
        useless_top_level_labels.drain_filter(|useless_label| {
            usefull_top_level_labels.iter().any(|usefull_label| {
                usefull_label.get_top_level_label() == useless_label.get_top_level_label()
            })
        });

        useless_top_level_labels.sort();
        usefull_sub_level_labels.sort();
        useless_sub_level_labels.sort();

        if let Some((i, core)) = filter {
            //we had the FilterLabel work
            let filter = SubFilter::from_core_with_label_options(
                core,
                usefull_top_level_labels,
                useless_top_level_labels,
                usefull_sub_level_labels,
                useless_sub_level_labels,
            );
            manager.push_finished_filter(i, filter, id);
        } else {
            //we had the MainFilterOptins work
            assert!(usefull_sub_level_labels.is_empty());
            manager.push_main_filter_label_options(usefull_top_level_labels, id)
        }
    }
}
