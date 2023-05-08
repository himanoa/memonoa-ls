use std::{collections::HashMap, path::PathBuf};

use derive_more::{Constructor, Deref};

use crate::{range::Range, wakachigaki::wakachigaki::Wakachigaki};

#[derive(Debug, Clone)]
pub struct TokenizeContext<'a, W: Wakachigaki> {
    pub wakachigaki: W,
    pub documents: &'a HashMap<String, PathBuf>,
}

impl<'a, W: Wakachigaki> TokenizeContext<'a, W> {
    pub fn new(wakachigaki: W, documents: &'a HashMap<String, PathBuf>) -> Self {
        Self {
            wakachigaki,
            documents,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
#[deref(forward)]
pub struct MemonoaAst(Vec<MemonoaLine>);
#[derive(Debug, Clone, PartialEq, Eq, Deref)]
#[deref(forward)]
pub struct MemonoaLine(Vec<MemonoaWord>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemonoaWord {
    Link {
        path: PathBuf,
        value: String,
        range: Range,
    },
    Normal {
        value: String,
        range: Range,
    },
}

impl MemonoaWord {
    pub fn tokenize(
        documents: &HashMap<String, PathBuf>,
        word: impl Into<String>,
        start_position: usize,
    ) -> MemonoaWord {
        let word: String = word.into();
        match documents.get(&word) {
            Some(path) => MemonoaWord::Link {
                path: path.clone(),
                range: Range::new(start_position, &word),
                value: word.into(),
            },
            None => MemonoaWord::Normal{
                range: Range::new(start_position, &word),
                value: word.into(),
            },
        }
    }

    pub fn range(&self) -> &Range {
        match self {
            MemonoaWord::Link {  range, .. } => range,
            MemonoaWord::Normal { range, .. } => range
        }
    }
}

impl MemonoaLine {
    pub fn tokenize<W: Wakachigaki>(ctx: TokenizeContext<W>, text: String) -> MemonoaLine {
        MemonoaLine(
            ctx.wakachigaki
                .segment(&text)
                .iter()
                .fold(vec![], |acc, t| {
                    let mut next_line = acc.clone();
                    next_line.push(MemonoaWord::tokenize(
                        ctx.documents,
                        t,
                        acc.last().map_or_else(|| 0, |w| w.range().end_character),
                    ));
                    next_line
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use maplit::hashmap;

    use super::{MemonoaLine, MemonoaWord, TokenizeContext};
    use crate::{range::Range, wakachigaki::tiny_segmenter_wakachigaki::TinySegmentWakachigaki};

    #[test]
    fn memonoa_word_tokenize_is_return_to_normal_text() {
        assert_eq!(
            MemonoaWord::tokenize(&HashMap::new(), "私", 0),
            MemonoaWord::Normal { value: "私".to_string(), range: Range::new(0, "私")}
        )
    }

    #[test]
    fn memonoa_word_tokenize_is_return_to_link_text() {
        let path: PathBuf = ["tmp", "himanoa", "memo.md"].iter().collect();
        assert_eq!(
            MemonoaWord::tokenize(&hashmap!("私".to_string() => path.clone()), "私", 0),
            MemonoaWord::Link {
                path,
                value: "私".to_string(),
                range: Range::new(0, "私")
            }
        )
    }

    #[test]
    fn memonoa_line_tokenize_is_return_to_memonoa_line() {
        let wakachigaki = TinySegmentWakachigaki::new();
        let dict: HashMap<String, PathBuf> = hashmap!(
            "私".to_string() => ["tmp", "himanoa", "documents", "私.md"].iter().collect(),
            "Rustacean".to_string() => ["tmp", "himanoa", "documents", "Rustacean.md"].iter().collect()
        );
        let ctx = TokenizeContext::new(wakachigaki, &dict);
        assert_eq!(
            *MemonoaLine::tokenize(ctx, "私はRustaceanです".to_string()),
            [
                MemonoaWord::Link {
                    path: dict.get("私").unwrap().clone(),
                    value: "私".to_string(),
                    range: Range::new(0, "私")
                },
                MemonoaWord::Normal{ value: "は".to_string(), range: Range::new(1, "は") },
                MemonoaWord::Link {
                    path: dict.get("Rustacean").unwrap().clone(),
                    range: Range::new(2, "Rustacean"),
                    value: "Rustacean".to_string()
                },
                MemonoaWord::Normal{value: "です".to_string(), range: Range::new(11, "です")},
            ]
        )
    }
}
