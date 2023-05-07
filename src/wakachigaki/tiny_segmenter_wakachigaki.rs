use super::wakachigaki::Wakachigaki;
use derive_more::Constructor;
use tinysegmenter::tokenize;

#[derive(Debug, Constructor)]
pub struct TinySegmentWakachigaki {}

impl Wakachigaki for TinySegmentWakachigaki {
    fn segment(&self, text: &str) -> Vec<String> {
        tokenize(text)
    }
}
