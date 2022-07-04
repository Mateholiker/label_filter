use std::{collections::HashMap, fmt::Display};

use eframe::egui::{Button, Grid, Ui};

use crate::{sub_filter::FilterInfo, Label, LabeledData, SubFilter};

pub struct MainFilter<D, L, TL, SL>
where
    D: LabeledData<L, TL, SL>,
    L: Label<TL, SL>,
    TL: Clone + Eq + Display,
    SL: Clone + Eq + Display,
{
    filters: Vec<SubFilter<D, L, TL, SL>>,
    top_level_label_options: Vec<L>,
    needs_init: bool,
}

impl<D, L, TL, SL> MainFilter<D, L, TL, SL>
where
    D: LabeledData<L, TL, SL>,
    L: Label<TL, SL>,
    TL: Clone + Eq + Display,
    SL: Clone + Eq + Display,
{
    pub fn new() -> MainFilter<D, L, TL, SL> {
        MainFilter {
            filters: Vec::new(),
            top_level_label_options: Vec::new(),
            needs_init: true,
        }
    }

    pub fn get_filter_map(&self, data: &[D]) -> Vec<usize> {
        data.iter()
            .enumerate()
            .filter_map(|(i, trajectory)| {
                for filter in &self.filters {
                    if !filter.filter(trajectory) {
                        return None;
                    }
                }
                Some(i)
            })
            .collect()
    }

    /// returns if the filter_map could have changed
    pub fn show(&mut self, ui: &mut Ui, data: &[D]) -> bool {
        if self.needs_init {
            self.update_all_filter(data);
            self.needs_init = false;
        }
        let mut filter_was_changed = false;
        let mut new_filter = None;
        Grid::new("label_filter_lib").show(ui, |ui| {
            let button = Button::new("Add Filter");
            if ui
                .add_enabled(!self.top_level_label_options.is_empty(), button)
                .clicked()
            {
                assert!(!self.top_level_label_options.is_empty());
                filter_was_changed = true;
                let id = (0..)
                    .find(|&id| {
                        for filter in self.filters.iter() {
                            if filter.id() == id {
                                return false;
                            }
                        }
                        true
                    })
                    .expect("not to have more than i32 many filterns");
                new_filter = Some(SubFilter::new(
                    self.top_level_label_options
                        .pop()
                        .expect("unreachable since we asserted that is is not empty"),
                    id,
                ));
            }
            ui.end_row();

            self.filters.drain_filter(|filter| {
                let FilterInfo {
                    needs_removal,
                    was_changed,
                } = filter.show(ui);
                filter_was_changed |= was_changed;
                ui.end_row();
                needs_removal
            });
        });

        if let Some(new_filter) = new_filter {
            assert!(filter_was_changed);
            self.filters.push(new_filter);
        }

        if filter_was_changed {
            self.update_all_filter(data);
        }

        filter_was_changed
    }

    /// if this function takes to much time we need to use a filter_mask instead filtering every trajectory (number of filters + 1) times
    /// but this would create the need to allocate the memory for the filter_mask and may not yield to much speedup
    fn update_all_filter(&mut self, trajectories: &[D]) {
        //index of the filter thats gets an update
        let mut i = 0;
        //this may seem to look like a mistake but the <= is on purpos so whene i == self.filter.len() we use the filter list on our own
        while i <= self.filters.len() {
            let mut trajectory_counter = 0;
            let mut label_map = HashMap::new();
            'trajectory_loop: for trajectory in trajectories {
                //filter the trajectories
                for (_, filter) in self.filters.iter().enumerate().filter(|&(j, _)| j != i) {
                    if !filter.filter(trajectory) {
                        continue 'trajectory_loop;
                    }
                }

                //increase the counter
                trajectory_counter += 1;

                //insert the labels
                for label in trajectory.get_labels() {
                    let label_counter = label_map.entry(label.clone()).or_insert(0);
                    *label_counter += 1;
                }
            }

            //get the current label to calculate the usefull sub level labels
            let current_label = self.filters.get(i).map(|f| f.label());

            //get the usefull labels
            //these are those which are in some but not all Trajectories
            let mut usefull_top_level_labels: Vec<L> = Vec::new();
            let mut usefull_sub_level_labels: Vec<L> = Vec::new();
            'label_loop: for (label, counter) in label_map.drain() {
                assert!(counter > 0);
                if counter < trajectory_counter {
                    if let Some(current_label) = current_label && current_label.get_top_level_label() == label.get_top_level_label() {
                        for usefull_label in &usefull_sub_level_labels {
                            if usefull_label.get_sub_level_label() == label.get_sub_level_label() {
                                continue 'label_loop;
                            }
                        }
                        usefull_sub_level_labels.push(label)
                    } else {
                        for usefull_label in &usefull_top_level_labels {
                            if usefull_label.get_top_level_label() == label.get_top_level_label() {
                                continue 'label_loop;
                            }
                        }
                        usefull_top_level_labels.push(label);
                    }
                }
            }

            usefull_top_level_labels.sort();
            if let Some(filter) = self.filters.get_mut(i) {
                usefull_sub_level_labels.sort();
                filter.set_top_level_label_options(usefull_top_level_labels);
                filter.set_sub_level_label_options_for_current_label(usefull_sub_level_labels);
            } else {
                self.top_level_label_options = usefull_top_level_labels;
            }

            i += 1;
        }
    }
}

impl<D, L, TL, SL> Default for MainFilter<D, L, TL, SL>
where
    D: LabeledData<L, TL, SL>,
    L: Label<TL, SL>,
    TL: Clone + Eq + Display,
    SL: Clone + Eq + Display,
{
    fn default() -> Self {
        Self::new()
    }
}
