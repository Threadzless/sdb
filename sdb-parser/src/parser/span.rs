use std::{ops::RangeInclusive, fmt::Debug};

// #[derive(Debug)]
pub struct Span(pub(crate) RangeInclusive<usize>);

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = std::env::args().collect::<Vec<String>>();

        if let Some( raw ) = args.get(1) {
            let rng = self.0.clone();
            let slice = &raw[rng];
            write!(f, "Span[ {:?} ] \t{:?}", self.0, slice)
        }
        else {
            write!(f, "Span[ {:?} ]", self.0)
        }
    }
}