use std::{marker::PhantomData, ops::Deref};

use eframe::{
    egui::{ComboBox, Label as EguiLabel, RichText, Ui},
    epaint::Color32,
};

use crate::{Label, LabeledData, SubLabel, TopLabel};

pub(crate) struct SubFilterCore<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    id: u32,
    label: L,
    inverted: bool,

    marker_0: PhantomData<TL>,
    marker_1: PhantomData<SL>,
}

impl<L, TL, SL> SubFilterCore<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    fn new(id: u32, label: L, inverted: bool) -> Self {
        Self {
            id,
            label,
            inverted,
            marker_0: PhantomData,
            marker_1: PhantomData,
        }
    }

    pub(crate) fn filter<D: LabeledData<L, TL, SL>>(&self, data: &D) -> bool {
        data.get_labels().contains(&self.label) ^ self.inverted
    }

    pub(crate) fn label(&self) -> &L {
        &self.label
    }

    pub(crate) fn id(&self) -> u32 {
        self.id
    }
}

impl<L, TL, SL> Clone for SubFilterCore<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            label: self.label.clone(),
            inverted: self.inverted,
            marker_0: PhantomData,
            marker_1: PhantomData,
        }
    }
}

impl<L, TL, SL> PartialEq for SubFilterCore<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    fn eq(&self, other: &Self) -> bool {
        let eq = self.id == other.id;

        if eq && (self.label != other.label || self.inverted != other.inverted) {
            panic!("FilterCore has same Id but not same label and same inversion")
        }
        eq
    }
}

impl<L, TL, SL> Eq for SubFilterCore<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
}

#[derive(Clone)]
pub(crate) struct SubFilter<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    core: SubFilterCore<L, TL, SL>,

    usefull_top_level_labels: Vec<L>,
    useless_top_level_labels: Vec<L>,
    usefull_sub_level_labels: Vec<L>,
    useless_sub_level_labels: Vec<L>,
}

impl<L, TL, SL> SubFilter<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    pub(crate) fn new(label: L, id: u32) -> SubFilter<L, TL, SL> {
        SubFilter {
            core: SubFilterCore::new(id, label, false),
            usefull_top_level_labels: Vec::new(),
            useless_top_level_labels: Vec::new(),
            usefull_sub_level_labels: Vec::new(),
            useless_sub_level_labels: Vec::new(),
        }
    }

    pub(crate) fn from_core_with_label_options(
        core: SubFilterCore<L, TL, SL>,

        usefull_top_level_labels: Vec<L>,
        useless_top_level_labels: Vec<L>,
        usefull_sub_level_labels: Vec<L>,
        useless_sub_level_labels: Vec<L>,
    ) -> SubFilter<L, TL, SL> {
        SubFilter {
            core,
            usefull_top_level_labels,
            useless_top_level_labels,
            usefull_sub_level_labels,
            useless_sub_level_labels,
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui) -> FilterInfo {
        if self.inverted {
            let rich_text = RichText::new("Not").color(Color32::RED);
            ui.add(<EguiLabel>::new(rich_text));
        } else {
            ui.add(EguiLabel::new(""));
        }

        let top_changed = ComboBox::from_id_source(format!("top_level_label_{}", self.id))
            .selected_text(self.label.get_top_level_label().to_string())
            .show_ui(ui, |ui| {
                let mut changed = false;
                for top_level_label in self.usefull_top_level_labels.iter() {
                    changed |= ui
                        .selectable_value(
                            &mut self.core.label,
                            top_level_label.clone(),
                            top_level_label.get_top_level_label().to_string(),
                        )
                        .changed();
                }
                for top_level_label in self.useless_top_level_labels.iter() {
                    let text = RichText::new(top_level_label.get_top_level_label().to_string())
                        .color(Color32::DARK_GRAY);
                    changed |= ui
                        .selectable_value(&mut self.core.label, top_level_label.clone(), text)
                        .changed();
                }
                changed
            })
            .inner
            .unwrap_or(false);

        let sub_changed = if let Some(sub_level_label) = self.label.get_sub_level_label() {
            ComboBox::from_id_source(format!("sub_level_label_{}", self.id))
                .selected_text(sub_level_label.to_string())
                .show_ui(ui, |ui| {
                    let mut changed = false;
                    for sub_level_label in self.usefull_sub_level_labels.iter() {
                        let text =
                            if let Some(sub_level_label) = sub_level_label.get_sub_level_label() {
                                sub_level_label.to_string()
                            } else {
                                continue;
                            };
                        changed |= ui
                            .selectable_value(&mut self.core.label, sub_level_label.clone(), text)
                            .changed();
                    }
                    for sub_level_label in self.useless_sub_level_labels.iter() {
                        let text =
                            if let Some(sub_level_label) = sub_level_label.get_sub_level_label() {
                                RichText::new(sub_level_label.to_string()).color(Color32::DARK_GRAY)
                            } else {
                                continue;
                            };
                        changed |= ui
                            .selectable_value(&mut self.core.label, sub_level_label.clone(), text)
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
            self.core.inverted = !self.core.inverted;
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

    pub(crate) fn clone_core(&self) -> SubFilterCore<L, TL, SL> {
        self.core.clone()
    }
}

impl<L, TL, SL> Deref for SubFilter<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    type Target = SubFilterCore<L, TL, SL>;

    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

impl<L, TL, SL> From<SubFilterCore<L, TL, SL>> for SubFilter<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    fn from(core: SubFilterCore<L, TL, SL>) -> Self {
        SubFilter {
            core,

            usefull_top_level_labels: Vec::new(),
            useless_top_level_labels: Vec::new(),
            usefull_sub_level_labels: Vec::new(),
            useless_sub_level_labels: Vec::new(),
        }
    }
}

pub(crate) struct FilterInfo {
    pub(crate) needs_removal: bool,
    pub(crate) was_changed: bool,
}
