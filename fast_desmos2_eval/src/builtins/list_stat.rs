// use fast_desmos2_comms::{value::ops::iter_full, List};

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ListStat {
    Mean,
    Min,
    Max,
    Total,
}

impl ListStat {
    pub const fn from_str(from: &[u8]) -> Option<Self> {
        Some(match from {
            b"mean" => Self::Mean,
            b"min" => Self::Min,
            b"max" => Self::Max,
            b"total" => Self::Total,
            _ => return None,
        })
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Mean => "mean",
            Self::Min => "min",
            Self::Max => "max",
            Self::Total => "total",
        }
    }

    // pub fn apply_numbers(&self, val: List<f64>) -> List<f64> {
    //     match self {
    //         ListStat::Total => val.fold(List::Term(0.0), &List::add),
    //         ListStat::Mean => {
    //             let len = val.len().map_or(1.0, |x| x as f64);
    //             val.fold(List::Term(0.0), &List::add).map(&|x| x / len)
    //         }
    //         ListStat::Min => val.fold_iter(List::Term(f64::INFINITY), &f64::min),
    //         ListStat::Max => val.fold_iter(List::Term(f64::NEG_INFINITY), &f64::max),
    //     }
    // }
}
