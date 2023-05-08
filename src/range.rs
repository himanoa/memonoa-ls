#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Range {
    pub start_character: usize,
    pub end_character: usize,
}

impl Range {
    pub fn new(start_character: usize, text: &str) -> Range {
        Range { start_character, end_character: start_character + text.chars().count() }
    }
}

#[cfg(test)]
mod tests {
    use super::Range;

    #[test]
    fn range_new_is_return_to_range_object() {
        let result = Range::new(10, "foo12");
        assert_eq!(
            result,
            Range {
                start_character: 10,
                end_character:15 
            }
        )
    }
}
