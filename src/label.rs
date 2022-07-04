use std::hash::Hash;

pub trait Label<TL, SL>: Eq + Hash + Clone + Ord
where
    TL: Clone + Eq,
    SL: Clone,
{
    fn get_top_level_label(&self) -> &TL;

    fn get_sub_level_label(&self) -> Option<&SL>;
}
