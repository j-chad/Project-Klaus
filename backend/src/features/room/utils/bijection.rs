use base64::Engine;
use base64::prelude::BASE64_STANDARD;

pub fn combine_seed_components(components: &[String]) -> Result<u64, base64::DecodeError> {
    let mut sum = 0u64;

    for component in components {
        let bytes = BASE64_STANDARD.decode(component.as_bytes())?;
        sum = bytes
            .iter()
            .fold(sum, |acc, &byte| acc.wrapping_add(u64::from(byte)));
    }

    Ok(sum)
}
