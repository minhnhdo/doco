pub struct Range {
    /// ranges of the value, each represented as a lower bound and an upper
    /// bound, both inclusive.
    ranges: Vec<(i64, i64)>,
}

impl Range {
    pub fn from(lower: i64, upper: i64) -> Range {
        if lower <= upper {
            Range {
                ranges: vec![(lower, upper)],
            }
        } else {
            Range {
                ranges: vec![],
            }
        }
    }

    pub fn simplify(&self) -> Range {
        let len = self.ranges.len();
        if len <= 1 {
            return Range { ranges: self.ranges.clone() };
        }
        let &(mut a, mut b) = unsafe { self.ranges.get_unchecked(0) };
        let mut ranges = Vec::with_capacity(len);
        for &(c, d) in &self.ranges[1..] {
            if b < c {
                ranges.push((a, b));
                a = c;
                b = d;
            } else {
                b = d;
            }
        }
        ranges.push((a, b));
        Range{ranges}
    }

    pub fn intersect(&self, other: &Range) -> Range {
        let self_len = self.ranges.len();
        let other_len = other.ranges.len();
        if self_len == 0 {
            return Range { ranges: other.ranges.clone() };
        }
        if other_len == 0 {
            return Range { ranges: self.ranges.clone() };
        }
        let mut ranges = Vec::with_capacity(self_len + other_len);
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
                //    a b
                // ---+-+-
                //  c   d
                // -+---+-
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
        Range { ranges: ranges }
    }

    pub fn union(&self, other: &Range) -> Range {
        let self_len = self.ranges.len();
        let other_len = other.ranges.len();
        if self_len == 0 {
            return Range { ranges: other.ranges.clone() };
        }
        if other_len == 0 {
            return Range { ranges: self.ranges.clone() };
        }
        let mut ranges = Vec::with_capacity(self_len + other_len);
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
                ranges.push((c, d));
                other_index += 1;
            } else if a >= c && b > d {
                //     a  b        a  b
                // ----+--+-    ---+--+-
                //  c  d     OR  c  d
                // -+--+----    -+--+---
                ranges.push((c, b));
                other_index += 1;
                self_index += 1;
            } else if a >= c && b <= d {
                //    a b
                // ---+-+-
                //  c   d
                // -+---+-
                ranges.push((c, d));
                other_index += 1;
                self_index += 1;
            } else if b >= d {
                //  a     b      a    b
                // -+-----+-    -+----+-
                //    c  d   OR    c  d
                // ---+--+--    ---+--+-
                ranges.push((a, b));
                other_index += 1;
                self_index += 1;
            } else if b < c {
                //  a  b
                // -+--+-----
                //      c  d
                // -----+--+-
                ranges.push((a, b));
                self_index += 1;
            } else {
                //  a  b
                // -+--+-----
                //    c  d
                // ---+--+-
                ranges.push((a, d));
                other_index += 1;
                self_index += 1;
            }
        }
        ranges.extend(self.ranges[self_index..].iter());
        ranges.extend(other.ranges[other_index..].iter());
        Range { ranges: ranges }.simplify()
    }
}

#[cfg(test)]
mod test {
    use super::Range;

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
    fn test_disjoint_intersect() {
        let r1 = Range::from(1, 3);
        let r2 = Range::from(4, 6);
        let test1 = r1.intersect(&r2);
        assert_eq!(Vec::<(i64, i64)>::new(), test1.ranges);
        let test2 = r2.intersect(&r1);
        assert_eq!(Vec::<(i64, i64)>::new(), test2.ranges);
    }

    #[test]
    fn test_overlapping_intersect() {
        let r1 = Range::from(1, 4);
        let r2 = Range::from(3, 6);
        let test1 = r1.intersect(&r2);
        assert_eq!(vec![(3, 4)], test1.ranges);
        let test2 = r2.intersect(&r1);
        assert_eq!(vec![(3, 4)], test2.ranges);
    }

    #[test]
    fn test_contained_intersect() {
        let r1 = Range::from(1, 6);
        let r2 = Range::from(3, 4);
        let test1 = r1.intersect(&r2);
        assert_eq!(vec![(3, 4)], test1.ranges);
        let test2 = r2.intersect(&r1);
        assert_eq!(vec![(3, 4)], test2.ranges);
    }

    #[test]
    fn test_complex_intersect() {
        let r1 = Range::from(1, 4).union(&Range::from(5, 7));
        let r2 = Range::from(3, 6).union(&Range::from(7, 8));
        let test1 = r1.intersect(&r2);
        assert_eq!(vec![(3, 4), (5, 6), (7, 7)], test1.ranges);
        let test2 = r2.intersect(&r1);
        assert_eq!(vec![(3, 4), (5, 6), (7, 7)], test2.ranges);
    }
}
