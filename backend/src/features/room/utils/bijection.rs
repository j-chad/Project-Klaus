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

pub fn get_assignment(
    seed: u64,
    query_santa_id: &str,
    mut santa_ids: Vec<String>,
    mut target_names: Vec<String>,
) -> Option<String> {
    // Sort the lists to ensure deterministic behavior
    santa_ids.sort();
    target_names.sort();

    let mut available: Vec<_> = (0..santa_ids.len()).collect();
    let mut rng = Pcg32::new(seed);

    for santa_id in santa_ids {
        #[allow(clippy::cast_possible_truncation)]
        // if we get into a situation where the number of
        // santa_ids is larger than u32::MAX then we have bigger problems
        let pick_index = rng.gen_range(available.len() as u32);
        let target_index = available.remove(pick_index as usize);

        if santa_id == query_santa_id {
            return Some(target_names[target_index].clone());
        }
    }

    // If we reach here, it means the query_santa_id was not found
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_assignment() {
        let seed = 2;
        let santa_ids = vec![
            "santa1".to_string(),
            "santa2".to_string(),
            "santa3".to_string(),
            "santa4".to_string(),
        ];
        let target_names = vec![
            "target1".to_string(),
            "target2".to_string(),
            "target3".to_string(),
            "target4".to_string(),
        ];

        let assignment1 = get_assignment(seed, "santa1", santa_ids.clone(), target_names.clone());
        assert_eq!(assignment1, Some("target4".to_string()));

        let assignment2 = get_assignment(seed, "santa2", santa_ids.clone(), target_names.clone());
        assert_eq!(assignment2, Some("target2".to_string()));

        let assignment3 = get_assignment(seed, "santa3", santa_ids.clone(), target_names.clone());
        assert_eq!(assignment3, Some("target3".to_string()));

        let assignment4 = get_assignment(seed, "santa4", santa_ids.clone(), target_names.clone());
        assert_eq!(assignment4, Some("target1".to_string()));
    }
}
