use std::{fmt::Display, hash::Hash};

pub trait Label<TL, SL>: Eq + Hash + Clone + Ord + Sync + Send + 'static
where
    TL: TopLabel,
    SL: SubLabel,
{
    fn get_top_level_label(&self) -> &TL;

    fn get_sub_level_label(&self) -> Option<&SL>;
}

pub trait SubLabel: Clone + Eq + Display + Sync + Send + 'static {}
impl<T: Clone + Eq + Display + Sync + Send + 'static> SubLabel for T {}

pub trait TopLabel: Clone + Eq + Display + Sync + Send + 'static {}
impl<T: Clone + Eq + Display + Sync + Send + 'static> TopLabel for T {}
