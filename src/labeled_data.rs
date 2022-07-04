use std::fmt::Display;

use crate::Label;

pub trait LabeledData<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: Clone + Eq + Display,
    SL: Clone + Eq + Display,
{
    fn get_labels(&self) -> &[L];
}
