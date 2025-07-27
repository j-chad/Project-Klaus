use crate::features::room::utils::pcg32::Pcg32;
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

pub fn get_target_for_santa_id(
    mut santa_ids: Vec<String>,
    mut target_names: Vec<String>,
    query_santa_id: &str,
    seed: u64,
) -> Option<String> {
    // Sort the lists to ensure deterministic behavior
    santa_ids.sort();
    target_names.sort();

    let mut available: Vec<_> = (0..santa_ids.len()).collect();
    let mut rng = Pcg32::new(seed);

    for _ in 0..santa_ids.len() {
        #[allow(clippy::cast_possible_truncation)]
        // if we get into a situation where the number of
        // santa_ids is larger than u32::MAX then we have bigger problems
        let pick_index = rng.gen_range(available.len() as u32);
        let pick = available.remove(pick_index as usize);

        if santa_ids[pick] == query_santa_id {
            return Some(target_names[pick].clone());
        }
    }

    // If we reach here, it means the query_santa_id was not found
    None
}
