use std::arch::asm;

use crate::Table;

#[inline]
fn add_sh(a: u64, b: u64) -> u64 {
    let out: u64;
    unsafe {
        #[cfg(target_arch = "x86_64")]
        asm!("lea {0}, [{1} + {2} * 2]", lateout(reg) out, in(reg) b, in(reg) a);
        #[cfg(target_arch = "aarch64")]
        asm!("add {0}, {1}, {2}, lsl #1", lateout(reg) out, in(reg) b, in(reg) a);
    }
    out
}

#[inline]
fn add_sh2(a: u64, b: u64) -> u64 {
    let out: u64;
    unsafe {
        #[cfg(target_arch = "x86_64")]
        asm!("lea {0}, [{1} + {2} * 4]", lateout(reg) out, in(reg) b, in(reg) a);
        #[cfg(target_arch = "aarch64")]
        asm!("add {0}, {1}, {2}, lsl #2", lateout(reg) out, in(reg) b, in(reg) a);
    }
    out
}

#[inline]
pub fn next_match(hash: &mut u64, table: &Table, buf: &[u8], mask: u64) -> Option<usize> {
    let mut hash_ = *hash;
    for (i, b) in buf.iter().enumerate() {
        hash_ = add_sh(hash_, table[*b as usize]);

        if hash_ & mask == 0 {
            *hash = hash_;
            return Some(i + 1);
        }
    }

    *hash = hash_;
    None
}

#[inline]
pub fn next_match2(hash: &mut u64, table: &Table, buf: &[u8], mask: u64) -> Option<usize> {
    let mut h = *hash;
    let len = buf.len();
    let (chunks, remainder) = buf.as_chunks::<4>();
    for (i, &[b1, b2, b3, b4]) in chunks.iter().enumerate() {
        let hash_orig = h;
        let b1 = table[b1 as usize];
        let b2 = table[b2 as usize];
        let b3 = table[b3 as usize];
        let b4 = table[b4 as usize];

        h = add_sh(hash_orig, b1);
        let b1b2 = add_sh(b1, b2);
        if h & mask == 0 {
            *hash = h;
            return Some(i * 4 + 1);
        }
        h = add_sh2(hash_orig, b1b2);
        if h & mask == 0 {
            *hash = h;
            return Some(i * 4 + 2);
        }

        let hash_orig = h;

        h = add_sh(hash_orig, b3);
        let b1b2 = add_sh(b3, b4);
        if h & mask == 0 {
            *hash = h;
            return Some(i * 4 + 3);
        }
        h = add_sh2(hash_orig, b1b2);
        if h & mask == 0 {
            *hash = h;
            return Some(i * 4 + 4);
        }
    }
    *hash = h;
    next_match(hash, table, remainder, mask).map(|i| i + chunks.len() * 4)
}

#[cfg(test)]
mod tests {
    use crate::{DEFAULT_TABLE, DEFAULT_TABLE_LS};

    use super::next_match2;

    quickcheck::quickcheck! {
        fn check_against_scalar(seed: u64, mask: u64) -> bool {
            let mut bytes = [0u8; 10240];
            let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
            rand::RngCore::fill_bytes(&mut rng, &mut bytes);

            let mut hash1 = 0;
            let mut hash2 = 0;

            let mut offset = 0;
            while offset < 10240 {
                let result_scalar = crate::scalar::next_match(&mut hash1, &DEFAULT_TABLE, &bytes[offset..], mask);
                let result_accelx = next_match2(&mut hash2, &DEFAULT_TABLE, &bytes[offset..], mask);

                match (result_scalar, result_accelx) {
                    (Some(a), Some(b)) => {
                        if a != b {
                            return false;
                        }
                        offset += a;
                    }
                    (None, None) => {
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }

            true
        }
    }
}

#[cfg(feature = "bench")]
#[bench]
fn throughput(b: &mut test::Bencher) {
    crate::bench::throughput(b, |hash, buf, mask| {
        next_match(hash, &crate::DEFAULT_TABLE, buf, mask)
    })
}

#[cfg(feature = "bench")]
#[bench]
fn throughput2(b: &mut test::Bencher) {
    crate::bench::throughput(b, |hash, buf, mask| {
        next_match2(hash, &crate::DEFAULT_TABLE, buf, mask)
    })
}
