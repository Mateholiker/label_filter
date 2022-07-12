use std::sync::Arc;

use eframe::egui::{Button, Grid, ProgressBar, Ui};

use crate::{sub_filter::FilterInfo, Label, LabeledData, SubFilter, SubLabel, TopLabel};

use self::filter_calculation::ThreadCommunicator;

mod filter_calculation;

pub struct MainFilter<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    filters: Vec<SubFilter<L, TL, SL>>,
    top_level_label_options: Vec<L>,
    default_label: Option<L>,
    needs_init: bool,
    thread_communicator: Arc<ThreadCommunicator<L, TL, SL>>,
}

impl<L, TL, SL> MainFilter<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    pub fn new() -> MainFilter<L, TL, SL> {
        MainFilter {
            filters: Vec::new(),
            top_level_label_options: Vec::new(),
            default_label: None,

            needs_init: true,
            thread_communicator: ThreadCommunicator::new(),
        }
    }

    pub fn get_filter_map<D: LabeledData<L, TL, SL>>(&self, data: &[D]) -> Vec<usize> {
        data.iter()
            .enumerate()
            .filter_map(|(i, data)| {
                for filter in &self.filters {
                    if !filter.filter(data) {
                        return None;
                    }
                }
                Some(i)
            })
            .collect()
    }

    /// returns if the filter_map could have changed
    pub fn show<D: LabeledData<L, TL, SL>>(&mut self, ui: &mut Ui, data: &[D]) -> bool {
        if self.needs_init {
            self.update_all_filter(data);
            self.needs_init = false;
        }
        if let Some((filters, top_level_label_options)) =
            self.thread_communicator.try_get_finished()
        {
            self.filters = filters;
            if self.default_label.is_none() {
                self.default_label = top_level_label_options.first().cloned();
            }
            self.top_level_label_options = top_level_label_options;
        }

        let mut filter_was_changed = false;
        let mut new_filter = None;

        ui.horizontal(|ui| {
            let button = if self.top_level_label_options.is_empty() {
                Button::new("Add Pointless Filter")
            } else {
                Button::new("Add Filter")
            };

            let clicked = ui
                .add_enabled(self.default_label.is_some(), button)
                .clicked();

            if clicked {
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

                let label = self
                    .top_level_label_options
                    .pop()
                    .or_else(|| self.default_label.clone());

                new_filter = label.map(|label| SubFilter::new(label, id));
            }

            //if !self.thread_communicator.is_idle() {
            let (a, b) = self.thread_communicator.get_progress();
            let progress = a as f32 / b as f32;
            let progress_bar = ProgressBar::new(progress).animate(true);
            ui.add(progress_bar);
            //}
        });

        Grid::new("label_filter_lib").show(ui, |ui| {
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

    fn update_all_filter<D: LabeledData<L, TL, SL>>(&self, data: &[D]) {
        self.thread_communicator.start(data, &self.filters);
    }
}

impl<L, TL, SL> Default for MainFilter<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    fn default() -> Self {
        Self::new()
    }
}
