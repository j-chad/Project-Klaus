#![allow(clippy::unreadable_literal)]
// Constants for PCG32 algorithm defined by the PCG authors
const MULTIPLIER: u64 = 6364136223846793005;
const INCREMENT: u64 = 1442695040888963407;

// PCG-XSH-RR-32
pub struct Pcg32 {
    state: u64,
}

impl Pcg32 {
    pub fn new(seed: u64) -> Self {
        let mut rng = Self {
            state: seed.wrapping_add(INCREMENT),
        };

        rng.next_u32(); // Discard the first value to ensure better randomness
        rng
    }

    pub fn next_u32(&mut self) -> u32 {
        let old_state = self.state;
        self.next_state();

        #[allow(clippy::cast_possible_truncation)]
        let xor_shifted = (((old_state >> 18) ^ old_state) >> 27) as u32;
        let rot = (old_state >> 59) as u32;

        xor_shifted.rotate_right(rot)
    }

    fn next_state(&mut self) {
        self.state = self.state.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
    }
}
