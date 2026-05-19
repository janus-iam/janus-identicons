use crate::RenderError;

pub const MAX_INPUT_LEN: usize = 256;

pub fn validate_input(input: &str) -> Result<(), RenderError> {
    if input.is_empty() {
        return Err(RenderError::EmptyInput);
    }
    if input.len() > MAX_INPUT_LEN {
        return Err(RenderError::InputTooLong);
    }
    if !input.chars().all(is_allowed_char) {
        return Err(RenderError::InvalidCharset);
    }
    Ok(())
}

fn is_allowed_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '@')
}

pub fn hash_input(input: &str) -> [u8; 32] {
    *blake3::hash(input.as_bytes()).as_bytes()
}
