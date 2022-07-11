use std::marker::PhantomData;

use crate::{Label, LabeledData, SubLabel, TopLabel};

pub(crate) struct LabelVec<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    labels: Vec<L>,
    chunk_borders: Vec<usize>,
    marker_0: PhantomData<TL>,
    marker_1: PhantomData<SL>,
}

impl<L, TL, SL> LabelVec<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    pub(crate) fn iter(&self) -> impl Iterator<Item = &[L]> {
        let mut last = 0;
        self.chunk_borders.iter().map(move |border| {
            let item = &self.labels[last..*border];
            last = *border;
            item
        })
    }
}

impl<D, L, TL, SL> From<&[D]> for LabelVec<L, TL, SL>
where
    D: LabeledData<L, TL, SL>,
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    fn from(data: &[D]) -> Self {
        let mut labels = Vec::new();
        let mut chunk_borders = Vec::new();
        for data in data {
            data.get_labels()
                .iter()
                .for_each(|l| labels.push(l.clone()));
            chunk_borders.push(labels.len());
        }

        Self {
            labels,
            chunk_borders,
            marker_0: PhantomData,
            marker_1: PhantomData,
        }
    }
}
