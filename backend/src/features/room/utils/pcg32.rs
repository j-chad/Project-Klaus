#![allow(clippy::unreadable_literal)]
// Constants for PCG32 algorithm defined by the PCG authors
const MULTIPLIER: u64 = 6364136223846793005;
const INCREMENT: u64 = 1442695040888963407;

// Output function XSH RR: xorshift high (bits), followed by a random rotate
// Constants are for 64-bit state, 32-bit output
const ROTATE: u32 = 59; // 64 - 5
const XSHIFT: u32 = 18; // (5 + 32) / 2
const SPARE: u32 = 27; // 64 - 32 - 5

// PCG-XSH-RR-32
pub struct Pcg32 {
    state: u64,
}

impl Pcg32 {
    pub fn new(seed: u64) -> Self {
        let mut rng = Self {
            state: seed.wrapping_add(INCREMENT),
        };

        rng.advance_state(); // Discard the first value to ensure better randomness
        rng
    }

    pub fn next_u32(&mut self) -> u32 {
        let old_state = self.state;
        self.advance_state();

        #[allow(clippy::cast_possible_truncation)]
        let xor_shifted = (((old_state >> XSHIFT) ^ old_state) >> SPARE) as u32;
        let rot = (old_state >> ROTATE) as u32;

        xor_shifted.rotate_right(rot)
    }

    #[allow(clippy::cast_possible_truncation)]
    /// Lemire's debiased integer multiplicative generator
    pub fn gen_range(&mut self, range: u32) -> u32 {
        let random_value = self.next_u32();
        let mut full_product = u64::from(random_value) * u64::from(range);
        let mut product_low_bits = full_product as u32;

        if product_low_bits < range {
            let threshold = 0u32.wrapping_sub(range) % range;
            while product_low_bits < threshold {
                let random_value = self.next_u32();
                full_product = u64::from(random_value) * u64::from(range);
                product_low_bits = full_product as u32;
            }
        }

        (full_product >> 32) as u32
    }

    fn advance_state(&mut self) {
        self.state = self.state.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
    }
}
