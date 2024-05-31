use crate::parse_key;
use crate::ParseError;
use crate::Rule;
use pest::iterators::Pair;
use pest::Span;

pub(crate) struct Bounds<'a> {
    lower: Pair<'a, Rule>,
    upper: Pair<'a, Rule>,
    span: Span<'a>,
}

impl<'a> Bounds<'a> {
    pub fn new(pair: Pair<'a, Rule>) -> Self {
        let span = pair.as_span();
        let mut iter = pair.into_inner();
        Self {
            lower: iter.next().unwrap().to_owned(),
            upper: iter.next().unwrap().to_owned(),
            span,
        }
    }
    fn spanned_error(&self, message: String) -> ParseError {
        let err = pest::error::Error::new_from_span(
            pest::error::ErrorVariant::<Rule>::CustomError { message },
            self.span,
        );
        Box::new(err).into()
    }

    pub fn expand_keys(&self) -> Result<(char, char), ParseError> {
        let lower = parse_key(self.lower.clone());
        let upper = parse_key(self.upper.clone());
        // if range attributes are unequal, complain
        if lower.attribute != upper.attribute {
            return Err(
                self.spanned_error("range bounds must have the same timing attributes".to_string())
            );
        }

        let lower: char = lower
            .key
            .parse()
            .expect("failed to parse lower bound as a character");
        let upper: char = upper
            .key
            .parse()
            .expect("failed to parse upper bound as a character");

        self.verify_range_bounds(lower, upper)?;

        Ok((lower, upper))
    }
    pub fn expand_commands(&self) -> Result<(char, char), ParseError> {
        // These unwraps must always work since the pest grammar picked up
        // the pairs due to the presence of the lower and upper bounds.
        // These should not be categorized as errors.
        let lower_bound = self
            .lower
            .as_str()
            .parse()
            .expect("failed to parse lower bound as a character");
        let upper_bound = self
            .upper
            .as_str()
            .parse()
            .expect("failed to parse upper bound as a character");

        self.verify_range_bounds(lower_bound, upper_bound)?;

        Ok((lower_bound, upper_bound))
    }
    fn verify_range_bounds(&self, lower_bound: char, upper_bound: char) -> Result<(), ParseError> {
        if !lower_bound.is_ascii() {
            return Err(self.spanned_error(format!(
                "shorthand lower bound `{0}` is not an ASCII character",
                lower_bound
            )));
        }
        if !upper_bound.is_ascii() {
            return Err(self.spanned_error(format!(
                "shorthand upper bound `{0}` is not an ASCII character",
                upper_bound
            )));
        }
        if lower_bound > upper_bound {
            return Err(self.spanned_error(format!(
                "shorthand lower bound `{}` is greater than upper bound `{}`",
                lower_bound, upper_bound
            )));
        }
        Ok(())
    }
}
