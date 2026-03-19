pub(crate) fn parse_required_u64(value: &str, name: &str) -> Result<u64, String> {
    value
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("{} must be a u64", name))
}

pub(crate) fn parse_required_usize(value: &str, name: &str) -> Result<usize, String> {
    value
        .trim()
        .parse::<usize>()
        .map_err(|_| format!("{} must be a usize", name))
}

pub(crate) fn parse_required_i64(value: &str, name: &str) -> Result<i64, String> {
    value
        .trim()
        .parse::<i64>()
        .map_err(|_| format!("{} must be an i64", name))
}

pub(crate) fn parse_optional_u64(value: &str, name: &str) -> Result<Option<u64>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed
        .parse::<u64>()
        .map(Some)
        .map_err(|_| format!("{} must be a u64", name))
}

pub(crate) fn parse_optional_usize(value: &str, name: &str) -> Result<Option<usize>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed
        .parse::<usize>()
        .map(Some)
        .map_err(|_| format!("{} must be a usize", name))
}

pub(crate) fn parse_optional_f64(value: &str, name: &str) -> Result<Option<f64>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed
        .parse::<f64>()
        .map(Some)
        .map_err(|_| format!("{} must be a f64", name))
}
