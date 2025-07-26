use rsa::BigUint;

fn combine_seed_components(components: &[&[u8]]) -> BigUint {
    components.iter().map(|&c| BigUint::from_bytes_be(c)).sum()
}
