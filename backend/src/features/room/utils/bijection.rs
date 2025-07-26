use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use rsa::BigUint;

pub fn combine_seed_components(components: &[String]) -> Result<BigUint, base64::DecodeError> {
    let mut sum = BigUint::from(0u32);

    for component in components {
        let bytes = BASE64_STANDARD.decode(component.as_bytes())?;
        sum += BigUint::from_bytes_be(&bytes);
    }

    Ok(sum)
}
