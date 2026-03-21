#[derive(Clone, Copy)]
pub(crate) enum ParseType {
    U64,
    Usize,
    I64,
    F64,
}

impl ParseType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::U64 | Self::Usize | Self::I64 => "integer",
            Self::F64 => "number",
        }
    }
}

pub(crate) struct ParseError {
    pub(crate) name: String,
    pub(crate) expected: ParseType,
}

pub(crate) fn parse_required_u64(value: &str, name: &str) -> Result<u64, ParseError> {
    value.trim().parse::<u64>().map_err(|_| ParseError {
        name: name.to_string(),
        expected: ParseType::U64,
    })
}

pub(crate) fn parse_required_usize(value: &str, name: &str) -> Result<usize, ParseError> {
    value.trim().parse::<usize>().map_err(|_| ParseError {
        name: name.to_string(),
        expected: ParseType::Usize,
    })
}

pub(crate) fn parse_required_i64(value: &str, name: &str) -> Result<i64, ParseError> {
    value.trim().parse::<i64>().map_err(|_| ParseError {
        name: name.to_string(),
        expected: ParseType::I64,
    })
}

pub(crate) fn parse_optional_u64(value: &str, name: &str) -> Result<Option<u64>, ParseError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed.parse::<u64>().map(Some).map_err(|_| ParseError {
        name: name.to_string(),
        expected: ParseType::U64,
    })
}

pub(crate) fn parse_optional_usize(value: &str, name: &str) -> Result<Option<usize>, ParseError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed.parse::<usize>().map(Some).map_err(|_| ParseError {
        name: name.to_string(),
        expected: ParseType::Usize,
    })
}

pub(crate) fn parse_optional_f64(value: &str, name: &str) -> Result<Option<f64>, ParseError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed.parse::<f64>().map(Some).map_err(|_| ParseError {
        name: name.to_string(),
        expected: ParseType::F64,
    })
}
