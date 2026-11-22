//! Générateur aléatoire déterministe (xoshiro256**), sans dépendance.
//!
//! Même graine ⇒ même suite de nombres. Conséquences :
//! - une partie est reproductible (débogage, rejeu) ;
//! - annuler puis refaire une pioche redonne la même carte (pas de triche).

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        // Initialisation par SplitMix64, recommandée pour xoshiro.
        let mut x = seed;
        let mut next = || {
            x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = x;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        };
        Rng {
            s: [next(), next(), next(), next()],
        }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// Entier uniforme dans `0..n` (n > 0), sans biais (méthode de Lemire).
    #[inline]
    pub fn below(&mut self, n: u32) -> u32 {
        assert!(n > 0, "below(0) n'a pas de sens");
        let n64 = n as u64;
        loop {
            let m = (self.next_u64() >> 32) * n64;
            let low = m as u32;
            if low >= n || low >= (u32::MAX - n + 1) % n {
                return (m >> 32) as u32;
            }
        }
    }

    /// Mélange de Fisher-Yates (utilisé pour mélanger les decks).
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u32 + 1) as usize;
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meme_graine_meme_suite() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn graines_differentes_suites_differentes() {
        assert_ne!(Rng::new(1).next_u64(), Rng::new(2).next_u64());
    }

    #[test]
    fn below_reste_dans_les_bornes() {
        let mut r = Rng::new(7);
        for n in 1..50 {
            for _ in 0..200 {
                assert!(r.below(n) < n);
            }
        }
    }

    #[test]
    fn melange_conserve_les_elements() {
        let mut r = Rng::new(3);
        let mut v: Vec<u32> = (0..40).collect();
        r.shuffle(&mut v);
        let mut trie = v.clone();
        trie.sort();
        assert_eq!(trie, (0..40).collect::<Vec<_>>());
    }
}
