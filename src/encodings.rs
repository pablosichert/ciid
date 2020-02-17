pub fn to_sortable_base_16(data: &[u8]) -> String {
    let mut result = String::new();

    for byte in data {
        let upper = (('a' as u8) + (byte >> 4)) as char;
        let lower = (('a' as u8) + (byte & 0b00001111)) as char;

        result.push(upper);
        result.push(lower);
    }

    return result;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty() {
        let data = vec![];
        let encoded = to_sortable_base_16(&data);
        assert_eq!(encoded, "");
    }

    #[test]
    fn test_all_single_byte_inputs() {
        let encoded = (0..=255)
            .into_iter()
            .map(|input| to_sortable_base_16(&vec![input]))
            .collect::<Vec<_>>();

        assert_eq!(
            encoded,
            vec![
                "aa", "ab", "ac", "ad", "ae", "af", "ag", "ah", "ai", "aj", "ak", "al", "am", "an",
                "ao", "ap", "ba", "bb", "bc", "bd", "be", "bf", "bg", "bh", "bi", "bj", "bk", "bl",
                "bm", "bn", "bo", "bp", "ca", "cb", "cc", "cd", "ce", "cf", "cg", "ch", "ci", "cj",
                "ck", "cl", "cm", "cn", "co", "cp", "da", "db", "dc", "dd", "de", "df", "dg", "dh",
                "di", "dj", "dk", "dl", "dm", "dn", "do", "dp", "ea", "eb", "ec", "ed", "ee", "ef",
                "eg", "eh", "ei", "ej", "ek", "el", "em", "en", "eo", "ep", "fa", "fb", "fc", "fd",
                "fe", "ff", "fg", "fh", "fi", "fj", "fk", "fl", "fm", "fn", "fo", "fp", "ga", "gb",
                "gc", "gd", "ge", "gf", "gg", "gh", "gi", "gj", "gk", "gl", "gm", "gn", "go", "gp",
                "ha", "hb", "hc", "hd", "he", "hf", "hg", "hh", "hi", "hj", "hk", "hl", "hm", "hn",
                "ho", "hp", "ia", "ib", "ic", "id", "ie", "if", "ig", "ih", "ii", "ij", "ik", "il",
                "im", "in", "io", "ip", "ja", "jb", "jc", "jd", "je", "jf", "jg", "jh", "ji", "jj",
                "jk", "jl", "jm", "jn", "jo", "jp", "ka", "kb", "kc", "kd", "ke", "kf", "kg", "kh",
                "ki", "kj", "kk", "kl", "km", "kn", "ko", "kp", "la", "lb", "lc", "ld", "le", "lf",
                "lg", "lh", "li", "lj", "lk", "ll", "lm", "ln", "lo", "lp", "ma", "mb", "mc", "md",
                "me", "mf", "mg", "mh", "mi", "mj", "mk", "ml", "mm", "mn", "mo", "mp", "na", "nb",
                "nc", "nd", "ne", "nf", "ng", "nh", "ni", "nj", "nk", "nl", "nm", "nn", "no", "np",
                "oa", "ob", "oc", "od", "oe", "of", "og", "oh", "oi", "oj", "ok", "ol", "om", "on",
                "oo", "op", "pa", "pb", "pc", "pd", "pe", "pf", "pg", "ph", "pi", "pj", "pk", "pl",
                "pm", "pn", "po", "pp"
            ]
        );
    }

    #[test]
    fn test_sorted() {
        let encoded = (0..=255)
            .into_iter()
            .map(|input| to_sortable_base_16(&vec![input]))
            .collect::<Vec<_>>();

        let mut sorted = encoded.clone();
        sorted.sort();

        assert_eq!(encoded, sorted);
    }
}
