use crate::Table;

#[inline]
pub(crate) fn next_match(hash: &mut u64, table: &Table, buf: &[u8], mask: u64) -> Option<usize> {
    for (i, b) in buf.iter().enumerate() {
        *hash = (*hash << 1).wrapping_add(table[*b as usize]);

        if *hash & mask == 0 {
            return Some(i + 1);
        }
    }

    None
}

#[inline]
pub fn next_match2(hash: &mut u64, table: &Table, table_ls: &Table, buf: &[u8], mask: u64) -> Option<usize> {
    if mask >> 63 == 1 {
        return next_match(hash, table, buf, mask);
    }
    let mask_ls = mask << 1;
    let len = buf.len();
    for i in 0..len / 2 {
        let hash_orig = *hash;
        let b1 = unsafe { table_ls[*buf.get_unchecked(i * 2) as usize] };
        let b2 = unsafe { table[*buf.get_unchecked(i * 2 + 1) as usize] };
        *hash = (*hash << 2).wrapping_add(b1);
        if *hash & mask_ls == 0 {
            *hash = (hash_orig << 1).wrapping_add(unsafe { table[*buf.get_unchecked(i * 2) as usize] });
            return Some(i * 2 + 1);
        }
        *hash = hash.wrapping_add(b2);
        if *hash & mask == 0 {
            return Some(i * 2 + 2);
        }
    }
    if len % 2 == 1 {
        *hash = (*hash << 1).wrapping_add(unsafe { table[*buf.get_unchecked(len - 1) as usize] });

        if *hash & mask == 0 {
            return Some(len);
        }
    }

    None
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
                let result_accelx = next_match2(&mut hash2, &DEFAULT_TABLE, &DEFAULT_TABLE_LS, &bytes[offset..], mask);

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
        next_match2(hash, &crate::DEFAULT_TABLE, &crate::DEFAULT_TABLE_LS, buf, mask)
    })
}
