use crate::ecc::{stat};

mod ecc {
    use std::{fs, io, time};
    use std::any::type_name;
    use std::collections::BTreeSet;
    use rand::Rng;

    type Bits = Vec<u8>;

    fn rand_corrupt(data: &Bits, corruptions: usize) -> Bits {
        let mut ret = data.clone();
        let ret_len = ret.len();
        let mut rng = rand::thread_rng();
        let mut ids = BTreeSet::from_iter(0..ret.len());
        let mut get_random_id = || {
            loop {
                let i = rng.gen::<usize>() % ret_len;
                if ids.remove(&i) {
                    return i;
                }
            };
        };
        for _ in 0..corruptions {
            let bit = ret.get_mut(get_random_id()).unwrap();
            *bit = (!*bit) & 1;
        }
        ret
    }

    fn diff(lhs: &Bits, rhs: &Bits) -> Vec<usize> {
        (0..lhs.len().min(rhs.len()))
            .filter(|i| lhs[*i] != rhs[*i])
            .collect()
    }

    fn read_bits(filename: &str) -> Result<Bits, io::Error> {
        let mut ret = Bits::new();
        for mut byte in fs::read(filename)? {
            ret.extend((0..8).map(|_| {
                let ret = byte % 2;
                byte /= 2;
                ret
            }).rev());
        }
        Ok(ret)
    }

    fn kb(len: usize) -> f64 { len as f64 / (1 << 13) as f64 }

    pub fn stat<ECC: ErrorCorrectingCode>(test_file: &str, max_corruptions: usize) -> Result<(), io::Error> {
        let original = read_bits(test_file)?;
        let encoded = ECC::encode(&original);
        println!("ECC TYPE\t--\t{}", type_name::<ECC>());
        println!("\tORIGINAL LEN\t{:?} ({:.2}KB)", original.len(), kb(original.len()));
        println!("\tENCODED LEN  \t{:?} ({:.2}%)", encoded.len(), original.len() as f64 * 100.0 / encoded.len() as f64);

        for corruptions in 1..=max_corruptions {
            let mut decode_time = 0.0;
            let max_diff = (0..1000).map(|_| {
                let corrupted = rand_corrupt(&encoded, corruptions);
                let now = time::Instant::now();
                let decoded = ECC::decode(&corrupted);
                decode_time += now.elapsed().as_secs_f64();
                (diff(&corrupted, &original).len(), diff(&decoded, &original).len())
            }).max().unwrap();
            println!("({}) DIFF CORRUPTED {:?}\tDECODED {}\t({:.2}s)", corruptions, max_diff.0, max_diff.1, decode_time);
        }
        Ok(())
    }

    pub trait ErrorCorrectingCode {
        fn encode(data: &Bits) -> Bits;
        fn decode(data: &Bits) -> Bits;
    }

    pub struct VoteECC<const VOTES: usize> {}

    impl<const VOTES: usize> ErrorCorrectingCode for VoteECC<VOTES> {
        fn encode(data: &Bits) -> Bits {
            data.iter().cloned().cycle().take(data.len() * VOTES).collect()
        }

        fn decode(data: &Bits) -> Bits {
            (0..(data.len() / VOTES))
                .map(|i|
                    ((0..VOTES)
                        .map(|n| data[i + data.len() / VOTES * n] as usize)
                        .sum::<usize>() as f64 / VOTES as f64
                    ).round() as u8
                )
                .collect()
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    stat::<ecc::VoteECC<3>>("testdata/hello", 5)?;

    Ok(())
}
