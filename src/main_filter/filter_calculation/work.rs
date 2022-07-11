use std::sync::Arc;

use crate::{sub_filter::SubFilterCore, Label, SubLabel, TopLabel};

use super::label_vec::LabelVec;

pub(crate) enum Work<L, TL, SL>
where
    L: Label<TL, SL>,
    TL: TopLabel,
    SL: SubLabel,
{
    NothingToDo,

    FilterLabel {
        filter: (usize, SubFilterCore<L, TL, SL>),
        all_filters: Arc<Vec<SubFilterCore<L, TL, SL>>>,
        labels: Arc<LabelVec<L, TL, SL>>,
    },

    MainFilterOptins {
        all_filters: Arc<Vec<SubFilterCore<L, TL, SL>>>,
        labels: Arc<LabelVec<L, TL, SL>>,
    },
}
