#![allow(unused)]

#[derive(Clone, Copy)]
pub struct Timer {
    delay: u8,
    step: u8,
}

impl Timer {
    /// Create a new timer with the given delay
    #[inline(always)]
    pub const fn new(delay: u8) -> Self {
        Self { delay, step: 0 }
    }

    /// Create a new timer with unspecified delay
    #[inline(always)]
    pub const fn infinite() -> Self {
        Self::new(u8::MAX)
    }

    /// Init timer at a specific step
    #[inline]
    pub const fn start_at(mut self, step: u8) -> Self {
        self.step = step;
        self
    }

    /// Read the current counter value
    #[inline]
    pub const fn get(&self) -> u8 {
        self.step
    }

    /// Return the configured delay
    #[inline]
    pub const fn delay(&self) -> u8 {
        self.delay
    }

    /// Increase by one but do nothing else
    #[inline]
    pub const fn tick(&mut self) {
        self.step = self.step.saturating_add(1);
    }

    /// Returns true if the timer is due
    #[inline]
    pub const fn is_due(&self) -> bool {
        self.step >= self.delay
    }

    /// Set the timer to be due
    #[inline]
    pub const fn set_due(&mut self) {
        self.step = u8::MAX;
    }

    /// Reset the timer interval to start over
    #[inline]
    pub const fn reset(&mut self) {
        self.step = 0;
    }

    /// Increase by one, if due reset timer and return true
    pub const fn step(&mut self) -> bool {
        self.tick();
        if self.is_due() {
            self.reset();
            true
        } else {
            false
        }
    }
}
