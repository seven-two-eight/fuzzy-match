//! A naive fuzzy matching algorithm designed for correcting mis-spelled names.

use std::collections::HashMap;

/// Computes case-insensitive cosine similarity between two strings 
/// using byte-based unigram, bigram, trigram featuers.
/// # Examples
/// ```
/// use std::collections::HashMap;
/// use std::iter::FromIterator;
/// 
/// let target  = "abcd";
/// let queries = ["", "abcd", "efg", "a", "c", "ce", "cb", "bc", "cd"];
/// let scores = HashMap::<&str, f32>::from_iter( queries
///     .iter()
///     .map( |&query| (query, fuzzy_match::trigram::score(&target, query)) )
/// );
/// 
/// // Empty string always gets zero score, even with itself.
/// assert!(0.0 == scores[""] && 0.0 == fuzzy_match::trigram::score("", ""));
/// 
/// // "efg" gets zero score because nothing matches.
/// assert!(0.0 == scores["efg"]);
/// 
/// // Perfect match gets the heighest 1.0 score.
/// assert!((1.0 - scores["abcd"]) < 1e-5);
/// 
/// // Score is also between 0.0 and 1.0.
/// assert!(0.0 <= scores["a"] && scores["a"] <= 1.0);
/// 
/// // "a" and "c" get the same score because both match one character.
/// assert!(scores["a"]  == scores["c"]);
/// 
/// // "a" gets higher score than "ce" even though both match one character,
/// // because "ce" is penalized for the extra 'e'.
/// assert!(scores["a"]  > scores["ce"]);
/// 
/// // "a" gets lower score than "cb" because "cb" matches two characters.
/// assert!(scores["a"]  <  scores["cb"]);
/// // "cb" gets lower score than "bc" because "bc" not only matches two characters,
/// // but also is in the right order.
/// assert!(scores["cb"] <  scores["bc"]);
/// 
/// // "bc" and "cd" receives same score because position being matched doesn't weight. 
/// assert!(scores["bc"] == scores["cd"]);
/// ```
pub fn score(s: &str, t: &str) -> f32 {
    let s = s.to_lowercase();
    let t = t.to_lowercase();
    let fs = features(&s);
    let ft = features(&t);
    fs.into_iter()
        .filter_map ( |(us, vs)| ft.get(us).map(|vt| vs*vt) )
        .sum()
}

fn features(s: &str) -> HashMap<&[u8], f32> {
    let s = s.as_bytes();
    let mut fs = HashMap::new();
    for k in 1..3 {
        if k > s.len() {
            break;
        }
        for w in s.windows(k) {
            *fs.entry(w).or_insert(0.0) += 1.0;
        }
    }
    let n = fs.values().map(|&v| v*v).sum::<f32>().sqrt();
    for (_, v) in fs.iter_mut() {
        *v /= n;
    }
    fs
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity() {
        assert!((1.0-score("abcd", "abcd")) < 1e-5);
    }
    #[test]
    fn empty_string() {
        assert_eq!(0.0, score("", ""));
        assert_eq!(0.0, score("", "abc"));
        assert_eq!(0.0, score("abc", ""));
        assert_eq!(0.0, score("abc", "ef"));
    }
    #[test]
    fn symmetry() {
        let s1 = "abc";
        let s2 = "bcef";
        assert_eq!(score(s1, s2), score(s2, s1));
    }
    #[test]
    fn unigram() {
        let a4  = "aaaa";
        assert!(score("abc", a4) < score("aba", a4));
        assert_eq!(score("abac", a4), score("baca", a4));
    }
    #[test]
    fn bigram() {
        let a4 = "abcd";
        assert!(score("baaa", a4) < score("aaab", a4));
    }
    #[test]
    fn trigram() {
        let a4 = "abcd";
        assert!(score("ecba", a4) < score("eabc", a4));
    }
}
