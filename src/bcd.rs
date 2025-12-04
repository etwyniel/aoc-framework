use std::{
    fmt::{Debug, Display},
    ops::{Add, Shl, Shr, Sub},
    str::FromStr,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bcd(u64);

impl Bcd {
    pub fn len(self) -> u32 {
        u64::BITS / 4 - self.0.leading_zeros() / 4
    }

    pub fn repeat(self, n: u32) -> Self {
        let len = self.len();
        self.repeat_len(len, n)
    }

    pub fn repeat_len(self, len: u32, n: u32) -> Self {
        let res = (0..n).fold(0, |acc, _| (acc << (4 * len)) | self.0);
        Bcd(res)
    }
}

impl From<Bcd> for u64 {
    fn from(value: Bcd) -> Self {
        (0..value.len())
            .rev()
            .fold(0, |acc, i| acc * 10 + ((value >> i).0 & 0xf))
    }
}

impl From<u64> for Bcd {
    fn from(mut value: u64) -> Self {
        let mut len = 0;
        let mut out = 0;
        while value > 0 {
            out = (out >> 4) | ((value % 10) << (u64::BITS - 4));
            value /= 10;
            len += 1;
        }
        Bcd(out >> (u64::BITS - len * 4))
    }
}

impl Debug for Bcd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&u64::from(*self), f)
    }
}

impl Display for Bcd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&u64::from(*self), f)
    }
}

impl FromStr for Bcd {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = 0;
        s.bytes().for_each(|b| {
            out = (out << 4) | u64::from(b - b'0');
        });
        Ok(Bcd(out))
    }
}

impl Shl<u32> for Bcd {
    type Output = Self;
    fn shl(self, rhs: u32) -> Self::Output {
        Bcd(self.0 << (4 * rhs))
    }
}

impl Shr<u32> for Bcd {
    type Output = Self;
    fn shr(self, rhs: u32) -> Self::Output {
        Bcd(self.0 >> (4 * rhs))
    }
}

impl Add<u32> for Bcd {
    type Output = Self;
    fn add(self, mut rhs: u32) -> Self::Output {
        let mut carry = 0;
        let mut out = self.0;
        let mut offset = 0;
        while rhs > 0 || carry > 0 {
            let res = ((out >> offset) & 0xf) + (rhs as u64 % 10) + carry;
            carry = res / 10;
            out = (out & !(0xf << offset)) | ((res % 10) << offset);
            rhs /= 10;
            offset += 4;
        }
        Bcd(out)
    }
}

impl Sub<u32> for Bcd {
    type Output = Self;
    fn sub(self, mut rhs: u32) -> Self::Output {
        let mut carry = 0;
        let mut out = self.0;
        let mut offset = 0;
        while rhs > 0 || carry > 0 {
            let res = ((out >> offset) & 0xf) as i64 + (rhs as i64 % 10) - carry;
            carry = (res < 0) as i64;
            out = (out & !(0xf << offset)) | ((res.rem_euclid(10) as u64) << offset);
            rhs /= 10;
            offset += 4;
        }
        Bcd(out)
    }
}
