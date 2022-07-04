use std::{fmt::Display, marker::PhantomData};

use eframe::{
    egui::{ComboBox, Label as EguiLabel, RichText, Ui},
    epaint::Color32,
};

use crate::{Label, LabeledData};

pub(crate) struct SubFilter<D, L, TL, SL>
where
    D: LabeledData<L, TL, SL>,
    L: Label<TL, SL>,
    TL: Clone + Eq + Display,
    SL: Clone + Eq + Display,
{
    id: u32,
    label: L,
    inverted: bool,

    top_level_label_options: Vec<L>,
    sub_level_label_options_for_current_label: Vec<L>,

    marker_0: PhantomData<D>,
    marker_1: PhantomData<TL>,
    marker_2: PhantomData<SL>,
}

impl<D, L, TL, SL> SubFilter<D, L, TL, SL>
where
    D: LabeledData<L, TL, SL>,
    L: Label<TL, SL>,
    TL: Clone + Eq + Display,
    SL: Clone + Eq + Display,
{
    pub(crate) fn new(label: L, id: u32) -> SubFilter<D, L, TL, SL> {
        SubFilter {
            id,
            label,
            inverted: false,
            top_level_label_options: Vec::new(),
            sub_level_label_options_for_current_label: Vec::new(),
            marker_0: PhantomData,
            marker_1: PhantomData,
            marker_2: PhantomData,
        }
    }

    pub(crate) fn label(&self) -> &L {
        &self.label
    }

    pub(crate) fn id(&self) -> u32 {
        self.id
    }

    pub(crate) fn filter(&self, trajectory: &D) -> bool {
        trajectory.get_labels().contains(&self.label) ^ self.inverted
    }

    pub(crate) fn set_sub_level_label_options_for_current_label(&mut self, options: Vec<L>) {
        assert!(options
            .iter()
            .map(|l| l.get_top_level_label() == self.label.get_top_level_label())
            .all(|elem| elem));

        self.sub_level_label_options_for_current_label = options;
    }

    pub(crate) fn set_top_level_label_options(&mut self, options: Vec<L>) {
        self.top_level_label_options = options;
    }

    pub(crate) fn show(&mut self, ui: &mut Ui) -> FilterInfo {
        if self.inverted {
            let rich_text = RichText::new("Not").color(Color32::RED);
            ui.add(<EguiLabel>::new(rich_text));
        } else {
            ui.add(EguiLabel::new(""));
        }

        let top_changed = ComboBox::from_id_source(format!("top_level_label_{}", self.id))
            .selected_text(format!("{}", self.label.get_top_level_label()))
            .show_ui(ui, |ui| {
                let mut changed = false;
                for top_level_label in self.top_level_label_options.iter() {
                    changed |= ui
                        .selectable_value(
                            &mut self.label,
                            top_level_label.clone(),
                            format!("{}", top_level_label.get_top_level_label()),
                        )
                        .changed();
                }
                changed
            })
            .inner
            .unwrap_or(false);

        let sub_changed = if let Some(sub_level_label) = self.label.get_sub_level_label() {
            ComboBox::from_id_source(format!("sub_level_label_{}", self.id))
                .selected_text(format!("{}", sub_level_label))
                .show_ui(ui, |ui| {
                    let mut changed = false;
                    for sub_level_label in self.sub_level_label_options_for_current_label.iter() {
                        let text =
                            if let Some(sub_level_label) = sub_level_label.get_sub_level_label() {
                                format!("{}", sub_level_label)
                            } else {
                                "None".to_owned()
                            };
                        changed |= ui
                            .selectable_value(&mut self.label, sub_level_label.clone(), text)
                            .changed();
                    }
                    changed
                })
                .inner
                .unwrap_or(false)
        } else {
            ui.label("");
            false
        };

        let inverted = if ui.button("invert").clicked() {
            self.inverted = !self.inverted;
            true
        } else {
            false
        };

        let removed = ui.button("remove").clicked();

        FilterInfo {
            was_changed: top_changed || sub_changed || inverted || removed,
            needs_removal: removed,
        }
    }
}

pub(crate) struct FilterInfo {
    pub(crate) needs_removal: bool,
    pub(crate) was_changed: bool,
}
