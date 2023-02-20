use std::{fmt::Display, iter::zip};

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum LV {
    H,
    L,
    X,
    Z,
}

impl LV {
    pub fn and(self, other: LV) -> LV {
        match (self, other) {
            (LV::L, _) => LV::L,
            (_, LV::L) => LV::L,
            (LV::Z, LV::H) => LV::X,
            (a, LV::H) => a,
            (LV::H, LV::Z) => LV::X,
            (LV::H, b) => b,
            (LV::X, LV::X) => LV::X,
            (LV::X, LV::Z) => LV::X,
            (LV::Z, LV::X) => LV::X,
            (LV::Z, LV::Z) => LV::X,
        }
    }
    pub fn or(self, other: LV) -> LV {
        match (self, other) {
            (LV::H, _) => LV::H,
            (_, LV::H) => LV::H,
            (LV::L, LV::Z) => LV::X,
            (LV::L, b) => b,
            (LV::Z, LV::L) => LV::X,
            (a, LV::L) => a,
            (LV::X, LV::X) => LV::X,
            (LV::Z, LV::X) => LV::X,
            (LV::X, LV::Z) => LV::X,
            (LV::Z, LV::Z) => LV::X,
        }
    }
    pub fn not(&self) -> LV {
        match self {
            LV::H => LV::L,
            LV::L => LV::H,
            LV::X => LV::X,
            LV::Z => LV::X,
        }
    }
}

impl Display for LV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            LV::H => write!(f, "{}", "1"),
            LV::L => write!(f, "{}", "0"),
            LV::X => write!(f, "{}", "X"),
            LV::Z => write!(f, "{}", "Z"),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Bits {
    value: Vec<LV>,
}

impl Display for Bits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in &self.value {
            write!(f, "{}", e).expect("For some unusual reason we can't print the bits to the console i am very sorry about that :(");
        }
        Ok(())
    }
}

impl Bits {
    pub fn new(n: usize) -> Bits {
        let value = (0..n).map(|_| LV::Z).collect();
        Bits { value }
    }

    pub fn set(&mut self, idx: usize, value: LV) {
        self.value[idx] = value;
    }

    pub fn get(&self, idx: usize) -> LV {
        self.value[idx]
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn and(&self, other: &Bits) -> Bits {
        if self.len() != other.len() {
            panic!("Can't compare bits of different sizes");
        }
        let value = zip(&self.value, &other.value)
            .map(|(a, b)| a.and(*b))
            .collect();
        Bits { value }
    }
    pub fn or(&self, other: &Bits) -> Bits {
        if self.len() != other.len() {
            panic!("Can't compare bits of different sizes");
        }
        let value = zip(&self.value, &other.value)
            .map(|(a, b)| a.or(*b))
            .collect();
        Bits { value }
    }
    pub fn not(&self) -> Bits {
        let value = self.value.iter().map(|e| e.not()).collect();
        Bits { value }
    }

    pub fn get_range(&self, start: usize, end: usize) -> Result<Bits, ()> {
        if start >= self.len() || end >= self.len() || start > end {
            return Err(());
        }
        let value = (start..end).map(|i| self.get(i)).collect();
        Ok(Bits { value })
    }
}
