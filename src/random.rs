use rand_core::RngCore;

pub struct Random<R: RngCore> {
    ascon: ascon::State,
    hwrng: R,
}

impl<R: RngCore> Random<R> {
    pub fn new(hwrng: R) -> Self {
        let ascon = ascon::State::default();
        let mut random = Random { ascon, hwrng };
        random.absorb();
        random
    }

    pub fn absorb(&mut self) {
        let input = self.hwrng.next_u64();
        self.ascon[0] ^= input;
        self.ascon.permute_6();
    }

    pub fn squeeze(&mut self) -> u64 {
        let num = self.ascon[0];
        self.absorb();
        num
    }
}
