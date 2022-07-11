use crate::{Label, SubLabel, TopLabel};

pub trait LabeledData<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    fn get_labels(&self) -> &[L];
}

impl<'s, L, TL, SL> LabeledData<L, TL, SL> for &'s [L]
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    fn get_labels(&self) -> &[L] {
        self
    }
}
