use std::cmp;

#[derive(Clone, Debug, PartialEq)]
pub struct Range {
    /// ranges of the value, each represented as a lower bound and an upper
    /// bound, both inclusive.
    ranges: Vec<(i64, i64)>,
}

impl Range {
    pub fn from(lower: i64, upper: i64) -> Range {
        Range {
            ranges: if lower <= upper {
                vec![(lower, upper)]
            } else {
                vec![]
            },
        }
    }

    pub fn difference(&self, other: &Range) -> Range {
        let self_len = self.ranges.len();
        let other_len = other.ranges.len();
        if self_len == 0 {
            return Range { ranges: Vec::new() };
        }
        if other_len == 0 {
            return Range {
                ranges: self.ranges.clone(),
            };
        }
        let mut ranges = Vec::with_capacity(cmp::max(self_len, other_len));
        let mut a = 0;
        let mut b = 0;
        let mut update_a_b = true;
        let mut next_self_index = 0;
        let mut other_index = 0;
        while (next_self_index < self_len || (next_self_index == self_len && !update_a_b))
            && other_index < other_len
        {
            if update_a_b {
                unsafe {
                    let &(l, u) = self.ranges.get_unchecked(next_self_index);
                    a = l;
                    b = u;
                }
                next_self_index += 1;
                update_a_b = false;
            }
            let &(c, d) = unsafe { other.ranges.get_unchecked(other_index) };
            if a > d {
                //       a  b
                // ------+--+-
                //  c  d
                // -+--+------
                other_index += 1;
            } else if a == d {
                //     a  b
                // ----+--+-
                //  c  d
                // -+--+----
                a = d + 1;
                update_a_b = a > b;
            } else if a >= c && b > d {
                //    a  b
                // ---+--+-
                //  c  d
                // -+--+---
                a = d + 1;
                update_a_b = a > b;
                other_index += 1;
            } else if a >= c && b <= d {
                //    a b
                // ---+-+-
                //  c   d
                // -+---+-
                update_a_b = true;
            } else if b >= d {
                //  a     b      a    b
                // -+-----+-    -+----+--
                //    c  d   OR    c  d
                // ---+--+--    ---+--+--
                if a <= c - 1 {
                    ranges.push((a, c - 1));
                }
                a = d + 1;
                update_a_b = a > b;
                other_index += 1;
            } else if b < c {
                //  a  b
                // -+--+-----
                //      c  d
                // -----+--+-
                ranges.push((a, b));
                update_a_b = true;
            } else {
                //  a  b
                // -+--+---
                //    c  d
                // ---+--+-
                if a <= c - 1 {
                    ranges.push((a, c - 1));
                }
                update_a_b = true;
            }
        }
        if !update_a_b {
            ranges.push((a, b));
        }
        if next_self_index < self_len {
            ranges.extend(self.ranges[next_self_index..].iter());
        }
        Range { ranges }
    }

    pub fn intersect(&self, other: &Range) -> Range {
        let self_len = self.ranges.len();
        let other_len = other.ranges.len();
        if self_len == 0 || other_len == 0 {
            return Range { ranges: Vec::new() };
        }
        let mut ranges = Vec::with_capacity(cmp::max(self_len, other_len));
        let mut self_index = 0;
        let mut other_index = 0;
        while self_index < self_len && other_index < other_len {
            let &(a, b) = unsafe { self.ranges.get_unchecked(self_index) };
            let &(c, d) = unsafe { other.ranges.get_unchecked(other_index) };
            if a > d {
                //       a  b
                // ------+--+-
                //  c  d
                // -+--+------
                other_index += 1;
            } else if a >= c && b > d {
                //     a  b        a  b
                // ----+--+-    ---+--+-
                //  c  d     OR  c  d
                // -+--+----    -+--+---
                ranges.push((a, d));
                other_index += 1;
            } else if a >= c && b <= d {
                //    a b        a b
                // ---+-+-    ---+-+-
                //  c   d  OR  c    d
                // -+---+-    -+----+-
                ranges.push((a, b));
                self_index += 1;
            } else if b >= d {
                //  a     b      a    b
                // -+-----+-    -+----+-
                //    c  d   OR    c  d
                // ---+--+--    ---+--+-
                ranges.push((c, d));
                other_index += 1;
            } else if b < c {
                //  a  b
                // -+--+-----
                //      c  d
                // -----+--+-
                self_index += 1;
            } else {
                //  a  b
                // -+--+-----
                //    c  d
                // ---+--+-
                ranges.push((c, b));
                self_index += 1;
            }
        }
        Range { ranges }
    }

    pub fn union(&self, other: &Range) -> Range {
        let self_len = self.ranges.len();
        let other_len = other.ranges.len();
        if self_len == 0 {
            return Range {
                ranges: other.ranges.clone(),
            };
        }
        if other_len == 0 {
            return Range {
                ranges: self.ranges.clone(),
            };
        }
        let mut ranges = Vec::with_capacity(self_len + other_len);
        let mut self_index = 0;
        let mut other_index = 0;
        while self_index < self_len && other_index < other_len {
            let &(a, b) = unsafe { self.ranges.get_unchecked(self_index) };
            let &(c, d) = unsafe { other.ranges.get_unchecked(other_index) };
            if a <= c {
                ranges.push((a, b));
                self_index += 1;
            } else {
                ranges.push((c, d));
                other_index += 1;
            }
        }
        ranges.extend(self.ranges[self_index..].iter());
        ranges.extend(other.ranges[other_index..].iter());
        // simplify
        assert!(ranges.len() >= 2);
        let &(mut a, mut b) = unsafe { ranges.get_unchecked(0) };
        let mut write_head = 0;
        for read_head in 1..ranges.len() {
            let &(c, d) = unsafe { ranges.get_unchecked(read_head) };
            if b < c {
                unsafe {
                    *ranges.get_unchecked_mut(write_head) = (a, b);
                }
                write_head += 1;
                a = c;
                b = d;
            } else if b < d {
                b = d;
            }
        }
        unsafe {
            *ranges.get_unchecked_mut(write_head) = (a, b);
        }
        ranges.truncate(write_head + 1);
        Range { ranges }
    }
}

#[cfg(test)]
mod test {
    use super::Range;

    #[test]
    fn test_empty_difference() {
        let r1 = Range::from(3, 1); // empty range
        let r2 = Range::from(4, 6);
        let test1 = r1.difference(&r2);
        assert_eq!(Vec::<(i64, i64)>::new(), test1.ranges);
        let test2 = r2.difference(&r1);
        assert_eq!(vec![(4, 6)], test2.ranges);
    }

    #[test]
    fn test_disjoint_difference() {
        let r1 = Range::from(1, 3);
        let r2 = Range::from(4, 6);
        let test1 = r1.difference(&r2);
        assert_eq!(vec![(1, 3)], test1.ranges);
        let test2 = r2.difference(&r1);
        assert_eq!(vec![(4, 6)], test2.ranges);
    }

    #[test]
    fn test_overlapping_difference() {
        let r1 = Range::from(1, 4);
        let r2 = Range::from(3, 6);
        let test1 = r1.difference(&r2);
        assert_eq!(vec![(1, 2)], test1.ranges);
        let test2 = r2.difference(&r1);
        assert_eq!(vec![(5, 6)], test2.ranges);
    }

    #[test]
    fn test_complex_overlapping_difference() {
        let r1 = Range::from(1, 10);
        let r2 = Range::from(2, 4)
            .union(&Range::from(5, 7))
            .union(&Range::from(8, 9));
        let test1 = r1.difference(&r2);
        assert_eq!(vec![(1, 1), (10, 10)], test1.ranges);
        let test2 = r2.difference(&r1);
        assert_eq!(Vec::<(i64, i64)>::new(), test2.ranges);
    }

    #[test]
    fn test_contained_difference() {
        let r1 = Range::from(1, 6);
        let r2 = Range::from(3, 4);
        let test1 = r1.difference(&r2);
        assert_eq!(vec![(1, 2), (5, 6)], test1.ranges);
        let test2 = r2.difference(&r1);
        assert_eq!(Vec::<(i64, i64)>::new(), test2.ranges);
    }

    #[test]
    fn test_complex_difference() {
        let r1 = Range::from(1, 4).union(&Range::from(5, 7));
        let r2 = Range::from(3, 6).union(&Range::from(7, 8));
        let test1 = r1.difference(&r2);
        assert_eq!(vec![(1, 2)], test1.ranges);
        let test2 = r2.difference(&r1);
        assert_eq!(vec![(8, 8)], test2.ranges);
    }

    #[test]
    fn test_empty_union() {
        let r1 = Range::from(3, 1); // empty range
        let r2 = Range::from(4, 6);
        let test1 = r1.union(&r2);
        assert_eq!(vec![(4, 6)], test1.ranges);
        let test2 = r2.union(&r1);
        assert_eq!(vec![(4, 6)], test2.ranges);
    }

    #[test]
    fn test_disjoint_union() {
        let r1 = Range::from(1, 3);
        let r2 = Range::from(4, 6);
        let test1 = r1.union(&r2);
        assert_eq!(vec![(1, 3), (4, 6)], test1.ranges);
        let test2 = r2.union(&r1);
        assert_eq!(vec![(1, 3), (4, 6)], test2.ranges);
    }

    #[test]
    fn test_overlapping_union() {
        let r1 = Range::from(1, 4);
        let r2 = Range::from(3, 6);
        let test1 = r1.union(&r2);
        assert_eq!(vec![(1, 6)], test1.ranges);
        let test2 = r2.union(&r1);
        assert_eq!(vec![(1, 6)], test2.ranges);
    }

    #[test]
    fn test_complex_overlapping_union() {
        let r1 = Range::from(1, 10);
        let r2 = Range::from(2, 4)
            .union(&Range::from(5, 7))
            .union(&Range::from(8, 9));
        let test1 = r1.union(&r2);
        assert_eq!(vec![(1, 10)], test1.ranges);
        let test2 = r2.union(&r1);
        assert_eq!(vec![(1, 10)], test2.ranges);
    }

    #[test]
    fn test_contained_union() {
        let r1 = Range::from(1, 6);
        let r2 = Range::from(3, 4);
        let test1 = r1.union(&r2);
        assert_eq!(vec![(1, 6)], test1.ranges);
        let test2 = r2.union(&r1);
        assert_eq!(vec![(1, 6)], test2.ranges);
    }

    #[test]
    fn test_complex_union() {
        let r1 = Range::from(1, 4).union(&Range::from(5, 7));
        let r2 = Range::from(3, 6).union(&Range::from(7, 8));
        let test1 = r1.union(&r2);
        assert_eq!(vec![(1, 8)], test1.ranges);
        let test2 = r2.union(&r1);
        assert_eq!(vec![(1, 8)], test2.ranges);
    }

    #[test]
    fn test_empty_intersection() {
        let r1 = Range::from(3, 1); // empty range
        let r2 = Range::from(4, 6);
        let test1 = r1.intersect(&r2);
        assert_eq!(Vec::<(i64, i64)>::new(), test1.ranges);
        let test2 = r2.intersect(&r1);
        assert_eq!(Vec::<(i64, i64)>::new(), test2.ranges);
    }

    #[test]
    fn test_disjoint_intersection() {
        let r1 = Range::from(1, 3);
        let r2 = Range::from(4, 6);
        let test1 = r1.intersect(&r2);
        assert_eq!(Vec::<(i64, i64)>::new(), test1.ranges);
        let test2 = r2.intersect(&r1);
        assert_eq!(Vec::<(i64, i64)>::new(), test2.ranges);
    }

    #[test]
    fn test_overlapping_intersection() {
        let r1 = Range::from(1, 4);
        let r2 = Range::from(3, 6);
        let test1 = r1.intersect(&r2);
        assert_eq!(vec![(3, 4)], test1.ranges);
        let test2 = r2.intersect(&r1);
        assert_eq!(vec![(3, 4)], test2.ranges);
    }

    #[test]
    fn test_complex_overlapping_intersection() {
        let r1 = Range::from(1, 6).union(&Range::from(7, 9));
        let r2 = Range::from(2, 4)
            .union(&Range::from(5, 8))
            .union(&Range::from(9, 10));
        let test1 = r1.intersect(&r2);
        assert_eq!(vec![(2, 4), (5, 6), (7, 8), (9, 9)], test1.ranges);
        let test2 = r2.intersect(&r1);
        assert_eq!(vec![(2, 4), (5, 6), (7, 8), (9, 9)], test2.ranges);
    }

    #[test]
    fn test_contained_intersection() {
        let r1 = Range::from(1, 6);
        let r2 = Range::from(3, 4);
        let test1 = r1.intersect(&r2);
        assert_eq!(vec![(3, 4)], test1.ranges);
        let test2 = r2.intersect(&r1);
        assert_eq!(vec![(3, 4)], test2.ranges);
    }

    #[test]
    fn test_complex_intersection() {
        let r1 = Range::from(1, 4).union(&Range::from(5, 7));
        let r2 = Range::from(3, 6).union(&Range::from(7, 8));
        let test1 = r1.intersect(&r2);
        assert_eq!(vec![(3, 4), (5, 6), (7, 7)], test1.ranges);
        let test2 = r2.intersect(&r1);
        assert_eq!(vec![(3, 4), (5, 6), (7, 7)], test2.ranges);
    }
}
