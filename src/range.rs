use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Range {
    start_character: usize,
    end_character: usize,
}

#[derive(Error, Debug, PartialEq)]
pub enum RangeError {
    #[error("start_character is greater than end_character")]
    InvalidRangeError { start_character: usize, end_character: usize }
}

impl Range {
    fn new(start_character: usize, end_character: usize) -> Result<Range, RangeError> {
        if end_character < start_character {
            return Err(RangeError::InvalidRangeError { start_character, end_character })
        }
        Ok(Range { start_character, end_character })
    }
}

#[cfg(test)]
mod tests {
    use crate::range::RangeError;

    use super::Range;

    #[test]
    fn range_new_is_return_to_invalid_range_error_when_start_character_greater_than_end_character() {
        let result = Range::new(100, 10);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err(), RangeError::InvalidRangeError { start_character: 100, end_character: 10 })
    }

    #[test]
    fn range_new_is_return_to_range_object() {
        let result = Range::new(10, 20);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), Range { start_character: 10, end_character: 20 })
    }
}
