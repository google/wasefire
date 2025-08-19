#![feature(get_disjoint_mut_helpers)]
#![allow(clippy::reversed_empty_ranges)]
#![allow(clippy::single_range_in_vec_init)]

use std::ops::{Bound, RangeBounds};

use wasefire_slice_cell::Error::{Borrow, Range};
use wasefire_slice_cell::{Error, SliceCell};

fn flip_start(range: impl RangeBounds<usize>) -> (Bound<usize>, Bound<usize>) {
    let Bound::Included(x) = range.start_bound().cloned() else { unreachable!() };
    (Bound::Excluded(x), range.end_bound().cloned())
}

fn start_index(range: &impl RangeBounds<usize>) -> usize {
    match range.start_bound().cloned() {
        Bound::Included(x) => x,
        Bound::Excluded(x) => x.checked_add(1).unwrap(),
        Bound::Unbounded => 0,
    }
}

#[test]
#[cfg(feature = "_internal")]
fn range() {
    use std::collections::BTreeSet;
    use std::ops::{Bound, RangeBounds};

    use wasefire_slice_cell::range_check;

    const L: usize = usize::MAX - 1;
    const M: usize = usize::MAX;

    fn squash(xs: impl IntoIterator<Item = Option<usize>>) -> Vec<usize> {
        xs.into_iter().flatten().collect::<BTreeSet<usize>>().into_iter().collect()
    }

    let mut tests = Vec::<(usize, Bound<usize>, Bound<usize>)>::new();
    for len in [0, 8, M] {
        let bounds_inc = squash([Some(0), Some(len), len.checked_add(1), Some(M)]);
        let bounds_dec = squash([Some(0), len.checked_sub(1), Some(len), Some(M)]);
        tests.push((len, Bound::Unbounded, Bound::Unbounded));
        for &end in &bounds_inc {
            tests.push((len, Bound::Unbounded, Bound::Excluded(end)));
        }
        for &start in &bounds_inc {
            tests.push((len, Bound::Included(start), Bound::Unbounded));
        }
        for &start in &bounds_inc {
            for &end in &bounds_inc {
                tests.push((len, Bound::Included(start), Bound::Excluded(end)));
            }
        }
        for &end in &bounds_dec {
            tests.push((len, Bound::Unbounded, Bound::Included(end)));
        }
        for &start in &bounds_inc {
            for &end in &bounds_dec {
                tests.push((len, Bound::Included(start), Bound::Included(end)));
            }
        }
        for &start in &bounds_dec {
            tests.push((len, Bound::Excluded(start), Bound::Unbounded));
        }
        for &start in &bounds_dec {
            for &end in &bounds_inc {
                tests.push((len, Bound::Excluded(start), Bound::Excluded(end)));
            }
        }
        for &start in &bounds_dec {
            for &end in &bounds_dec {
                tests.push((len, Bound::Excluded(start), Bound::Included(end)));
            }
        }
    }
    tests.reverse();
    macro_rules! test {
        ($len:expr, ! $range:expr, $expected:expr) => {
            test!($len, flip_start($range), $expected)
        };
        ($len:expr, $range:expr, $expected:expr) => {
            assert_eq!(
                ($len, $range.start_bound().cloned(), $range.end_bound().cloned()),
                tests.pop().unwrap()
            );
            assert_eq!(range_check($len, $range), $expected);
        };
    }

    // There are a few weird but expected results. Those are only for empty ranges, which are
    // ignored by the implementation. We still test them for completeness.
    test!(0, .., Ok(0 .. 0));
    test!(0, .. 0, Ok(0 .. 0));
    test!(0, .. 1, Err(Range));
    test!(0, .. M, Err(Range));
    test!(0, 0 .., Ok(0 .. 0));
    test!(0, 1 .., Err(Range));
    test!(0, M .., Err(Range));
    test!(0, 0 .. 0, Ok(0 .. 0));
    test!(0, 0 .. 1, Err(Range));
    test!(0, 0 .. M, Err(Range));
    test!(0, 1 .. 0, Err(Range));
    test!(0, 1 .. 1, Err(Range));
    test!(0, 1 .. M, Err(Range));
    test!(0, M .. 0, Err(Range));
    test!(0, M .. 1, Err(Range));
    test!(0, M .. M, Err(Range));
    test!(0, ..= 0, Err(Range));
    test!(0, ..= M, Err(Range));
    test!(0, 0 ..= 0, Err(Range));
    test!(0, 0 ..= M, Err(Range));
    test!(0, 1 ..= 0, Err(Range));
    test!(0, 1 ..= M, Err(Range));
    test!(0, M ..= 0, Err(Range));
    test!(0, M ..= M, Err(Range));
    test!(0, !0 .., Err(Range));
    test!(0, !M .., Err(Range));
    test!(0, !0 .. 0, Err(Range));
    test!(0, !0 .. 1, Err(Range));
    test!(0, !0 .. M, Err(Range));
    test!(0, !M .. 0, Err(Range));
    test!(0, !M .. 1, Err(Range));
    test!(0, !M .. M, Err(Range));
    test!(0, !0 ..= 0, Err(Range));
    test!(0, !0 ..= M, Err(Range));
    test!(0, !M ..= 0, Err(Range));
    test!(0, !M ..= M, Err(Range));
    test!(8, .., Ok(0 .. 8));
    test!(8, .. 0, Ok(0 .. 0));
    test!(8, .. 8, Ok(0 .. 8));
    test!(8, .. 9, Err(Range));
    test!(8, .. M, Err(Range));
    test!(8, 0 .., Ok(0 .. 8));
    test!(8, 8 .., Ok(8 .. 8));
    test!(8, 9 .., Err(Range));
    test!(8, M .., Err(Range));
    test!(8, 0 .. 0, Ok(0 .. 0));
    test!(8, 0 .. 8, Ok(0 .. 8));
    test!(8, 0 .. 9, Err(Range));
    test!(8, 0 .. M, Err(Range));
    test!(8, 8 .. 0, Err(Range));
    test!(8, 8 .. 8, Ok(8 .. 8));
    test!(8, 8 .. 9, Err(Range));
    test!(8, 8 .. M, Err(Range));
    test!(8, 9 .. 0, Err(Range));
    test!(8, 9 .. 8, Err(Range));
    test!(8, 9 .. 9, Err(Range));
    test!(8, 9 .. M, Err(Range));
    test!(8, M .. 0, Err(Range));
    test!(8, M .. 8, Err(Range));
    test!(8, M .. 9, Err(Range));
    test!(8, M .. M, Err(Range));
    test!(8, ..= 0, Ok(0 .. 1));
    test!(8, ..= 7, Ok(0 .. 8));
    test!(8, ..= 8, Err(Range));
    test!(8, ..= M, Err(Range));
    test!(8, 0 ..= 0, Ok(0 .. 1));
    test!(8, 0 ..= 7, Ok(0 .. 8));
    test!(8, 0 ..= 8, Err(Range));
    test!(8, 0 ..= M, Err(Range));
    test!(8, 8 ..= 0, Err(Range));
    test!(8, 8 ..= 7, Ok(8 .. 8)); // weird but expected
    test!(8, 8 ..= 8, Err(Range));
    test!(8, 8 ..= M, Err(Range));
    test!(8, 9 ..= 0, Err(Range));
    test!(8, 9 ..= 7, Err(Range));
    test!(8, 9 ..= 8, Err(Range));
    test!(8, 9 ..= M, Err(Range));
    test!(8, M ..= 0, Err(Range));
    test!(8, M ..= 7, Err(Range));
    test!(8, M ..= 8, Err(Range));
    test!(8, M ..= M, Err(Range));
    test!(8, !0 .., Ok(1 .. 8));
    test!(8, !7 .., Ok(8 .. 8));
    test!(8, !8 .., Err(Range));
    test!(8, !M .., Err(Range));
    test!(8, !0 .. 0, Err(Range));
    test!(8, !0 .. 8, Ok(1 .. 8));
    test!(8, !0 .. 9, Err(Range));
    test!(8, !0 .. M, Err(Range));
    test!(8, !7 .. 0, Err(Range));
    test!(8, !7 .. 8, Ok(8 .. 8));
    test!(8, !7 .. 9, Err(Range));
    test!(8, !7 .. M, Err(Range));
    test!(8, !8 .. 0, Err(Range));
    test!(8, !8 .. 8, Err(Range));
    test!(8, !8 .. 9, Err(Range));
    test!(8, !8 .. M, Err(Range));
    test!(8, !M .. 0, Err(Range));
    test!(8, !M .. 8, Err(Range));
    test!(8, !M .. 9, Err(Range));
    test!(8, !M .. M, Err(Range));
    test!(8, !0 ..= 0, Ok(1 .. 1)); // weird but expected
    test!(8, !0 ..= 7, Ok(1 .. 8));
    test!(8, !0 ..= 8, Err(Range));
    test!(8, !0 ..= M, Err(Range));
    test!(8, !7 ..= 0, Err(Range));
    test!(8, !7 ..= 7, Ok(8 .. 8)); // weird but expected
    test!(8, !7 ..= 8, Err(Range));
    test!(8, !7 ..= M, Err(Range));
    test!(8, !8 ..= 0, Err(Range));
    test!(8, !8 ..= 7, Err(Range));
    test!(8, !8 ..= 8, Err(Range));
    test!(8, !8 ..= M, Err(Range));
    test!(8, !M ..= 0, Err(Range));
    test!(8, !M ..= 7, Err(Range));
    test!(8, !M ..= 8, Err(Range));
    test!(8, !M ..= M, Err(Range));
    test!(M, .., Ok(0 .. M));
    test!(M, .. 0, Ok(0 .. 0));
    test!(M, .. M, Ok(0 .. M));
    test!(M, 0 .., Ok(0 .. M));
    test!(M, M .., Ok(M .. M));
    test!(M, 0 .. 0, Ok(0 .. 0));
    test!(M, 0 .. M, Ok(0 .. M));
    test!(M, M .. 0, Err(Range));
    test!(M, M .. M, Ok(M .. M));
    test!(M, ..= 0, Ok(0 .. 1));
    test!(M, ..= L, Ok(0 .. M));
    test!(M, ..= M, Err(Range));
    test!(M, 0 ..= 0, Ok(0 .. 1));
    test!(M, 0 ..= L, Ok(0 .. M));
    test!(M, 0 ..= M, Err(Range));
    test!(M, M ..= 0, Err(Range));
    test!(M, M ..= L, Ok(M .. M)); // weird but expected
    test!(M, M ..= M, Err(Range));
    test!(M, !0 .., Ok(1 .. M));
    test!(M, !L .., Ok(M .. M));
    test!(M, !M .., Err(Range));
    test!(M, !0 .. 0, Err(Range));
    test!(M, !0 .. M, Ok(1 .. M));
    test!(M, !L .. 0, Err(Range));
    test!(M, !L .. M, Ok(M .. M));
    test!(M, !M .. 0, Err(Range));
    test!(M, !M .. M, Err(Range));
    test!(M, !0 ..= 0, Ok(1 .. 1));
    test!(M, !0 ..= L, Ok(1 .. M));
    test!(M, !0 ..= M, Err(Range));
    test!(M, !L ..= 0, Err(Range));
    test!(M, !L ..= L, Ok(M .. M)); // weird but expected
    test!(M, !L ..= M, Err(Range));
    test!(M, !M ..= 0, Err(Range));
    test!(M, !M ..= L, Err(Range));
    test!(M, !M ..= M, Err(Range));

    assert_eq!(tests.pop(), None);
}

#[test]
#[cfg(feature = "_internal")]
fn borrow() {
    use core::slice::GetDisjointMutIndex;
    use std::ops::Range;

    use wasefire_slice_cell::{Access, State, borrow_check, state_invariant};

    struct Test<'a> {
        actual_ranges: &'a [Range<usize>],
        expected_ranges: &'a [Range<usize>],
        mask: u32,
        actual: State,
        access: Access,
        expected: State,
        ok: Option<bool>,
    }
    impl<'a> Test<'a> {
        fn run(
            actual_ranges: &[Range<usize>], access_range: Range<usize>,
            expected_ranges: &[Range<usize>], mask: u32,
        ) {
            let mut test = Test {
                actual_ranges,
                expected_ranges,
                mask,
                actual: Vec::new(),
                access: Access { exclusive: false, range: access_range },
                expected: Vec::new(),
                ok: None,
            };
            test.access.exclusive = test.next_mask();
            while let Some(actual_range) = test.next_actual_range() {
                let actual_access = Access { exclusive: test.next_mask(), range: actual_range };
                test.actual.push(actual_access);
                let actual_access = test.actual.last().unwrap().clone();
                if test.access.range.is_overlapping(&actual_access.range) {
                    if test.ok.is_none() {
                        test.ok = Some(!test.access.exclusive);
                        test.push_expected();
                    }
                    *test.ok.as_mut().unwrap() &= !actual_access.exclusive;
                    continue;
                }
                if test.ok.is_none() && test.access.range.end <= actual_access.range.start {
                    test.ok = Some(true);
                    test.push_expected();
                    assert_eq!(test.access.range, test.expected.last().unwrap().range);
                }
                assert_eq!(actual_access.range, test.next_expected_range());
                test.expected.push(actual_access);
            }
            if !test.expected_ranges.is_empty() {
                test.push_expected();
            }
            assert!(test.expected_ranges.is_empty());
            assert_eq!(test.mask, 0);
            let Test { mut actual, access, expected, ok, .. } = test;
            let result = if ok.unwrap_or(true) { Ok(()) } else { Err(Borrow) };
            eprintln!("- {actual:?}");
            eprintln!("  + {access:?} = {result:?}");
            eprintln!("  {expected:?}");
            assert!(state_invariant(&actual));
            assert_eq!(borrow_check(&mut actual, access), result);
            assert!(state_invariant(&actual));
            if result.is_ok() {
                assert_eq!(actual, expected);
            }
        }

        fn next_mask(&mut self) -> bool {
            let result = self.mask & 1 == 1;
            self.mask >>= 1;
            result
        }

        fn next_actual_range(&mut self) -> Option<Range<usize>> {
            let (first, tail) = self.actual_ranges.split_first()?;
            self.actual_ranges = tail;
            Some(first.clone())
        }

        fn next_expected_range(&mut self) -> Range<usize> {
            let (first, tail) = self.expected_ranges.split_first().unwrap();
            self.expected_ranges = tail;
            first.clone()
        }

        fn push_expected(&mut self) {
            let range = self.next_expected_range();
            self.expected.push(Access { range, ..self.access });
        }
    }
    fn test(
        actual_ranges: &[Range<usize>], access_range: Range<usize>,
        expected_ranges: &[Range<usize>],
    ) {
        eprintln!("* {actual_ranges:?}");
        eprintln!("  + {access_range:?}");
        eprintln!("  {expected_ranges:?}");
        for mask in 0 .. 1 << (1 + actual_ranges.len()) {
            Test::run(actual_ranges, access_range.clone(), expected_ranges, mask);
        }
    }

    // empty
    test(&[], 0 .. 1, &[0 .. 1]);
    test(&[], 10 .. 20, &[10 .. 20]);
    // no overlap last
    test(&[0 .. 1], 1 .. 2, &[0 .. 1, 1 .. 2]);
    test(&[0 .. 1], 10 .. 20, &[0 .. 1, 10 .. 20]);
    test(&[0 .. 1, 3 .. 7], 10 .. 20, &[0 .. 1, 3 .. 7, 10 .. 20]);
    test(&[0 .. 1, 1 .. 10], 10 .. 20, &[0 .. 1, 1 .. 10, 10 .. 20]);
    // no overlap first
    test(&[10 .. 20], 0 .. 1, &[0 .. 1, 10 .. 20]);
    test(&[10 .. 20], 5 .. 10, &[5 .. 10, 10 .. 20]);
    test(&[10 .. 20, 30 .. 40], 0 .. 1, &[0 .. 1, 10 .. 20, 30 .. 40]);
    test(&[10 .. 20, 20 .. 30], 5 .. 10, &[5 .. 10, 10 .. 20, 20 .. 30]);
    // no overlap middle
    test(&[0 .. 1, 10 .. 20], 3 .. 7, &[0 .. 1, 3 .. 7, 10 .. 20]);
    test(&[0 .. 1, 10 .. 20], 1 .. 7, &[0 .. 1, 1 .. 7, 10 .. 20]);
    test(&[0 .. 1, 10 .. 20], 3 .. 10, &[0 .. 1, 3 .. 10, 10 .. 20]);
    test(&[0 .. 1, 10 .. 20], 1 .. 10, &[0 .. 1, 1 .. 10, 10 .. 20]);
    // overlap inside inside
    test(&[10 .. 20], 13 .. 17, &[10 .. 20]);
    test(&[10 .. 20, 30 .. 40], 13 .. 17, &[10 .. 20, 30 .. 40]);
    test(&[0 .. 1, 10 .. 20], 13 .. 17, &[0 .. 1, 10 .. 20]);
    test(&[0 .. 1, 10 .. 20, 30 .. 40], 13 .. 17, &[0 .. 1, 10 .. 20, 30 .. 40]);
    // overlap inside inside multiple
    test(&[10 .. 14, 16 .. 20], 13 .. 17, &[10 .. 20]);
    test(&[10 .. 14, 16 .. 20, 30 .. 40], 13 .. 17, &[10 .. 20, 30 .. 40]);
    test(&[0 .. 1, 10 .. 14, 16 .. 20], 13 .. 17, &[0 .. 1, 10 .. 20]);
    test(&[0 .. 1, 10 .. 14, 16 .. 20, 30 .. 40], 13 .. 17, &[0 .. 1, 10 .. 20, 30 .. 40]);
    // overlap outside outside
    test(&[13 .. 17], 10 .. 20, &[10 .. 20]);
    test(&[13 .. 17, 30 .. 40], 10 .. 17, &[10 .. 17, 30 .. 40]);
    test(&[0 .. 1, 13 .. 17], 13 .. 20, &[0 .. 1, 13 .. 20]);
    test(&[0 .. 1, 13 .. 17, 30 .. 40], 13 .. 17, &[0 .. 1, 13 .. 17, 30 .. 40]);
    // overlap outside outside multiple
    test(&[13 .. 14, 15 .. 17], 10 .. 20, &[10 .. 20]);
    test(&[13 .. 14, 15 .. 17, 30 .. 40], 10 .. 20, &[10 .. 20, 30 .. 40]);
    test(&[0 .. 1, 13 .. 14, 15 .. 17], 10 .. 20, &[0 .. 1, 10 .. 20]);
    test(&[0 .. 1, 13 .. 14, 15 .. 17, 30 .. 40], 10 .. 20, &[0 .. 1, 10 .. 20, 30 .. 40]);
    // overlap inside outside
    test(&[10 .. 20], 13 .. 27, &[10 .. 27]);
    test(&[10 .. 20, 30 .. 40], 13 .. 27, &[10 .. 27, 30 .. 40]);
    test(&[0 .. 1, 10 .. 20], 13 .. 27, &[0 .. 1, 10 .. 27]);
    test(&[0 .. 1, 10 .. 20, 30 .. 40], 13 .. 27, &[0 .. 1, 10 .. 27, 30 .. 40]);
    // overlap inside outside multiple
    test(&[10 .. 15, 20 .. 25], 13 .. 27, &[10 .. 27]);
    test(&[10 .. 15, 20 .. 25, 30 .. 40], 13 .. 27, &[10 .. 27, 30 .. 40]);
    test(&[0 .. 1, 10 .. 15, 20 .. 25], 13 .. 27, &[0 .. 1, 10 .. 27]);
    test(&[0 .. 1, 10 .. 15, 20 .. 25, 30 .. 40], 13 .. 27, &[0 .. 1, 10 .. 27, 30 .. 40]);
    // overlap outside inside
    test(&[10 .. 20], 3 .. 17, &[3 .. 20]);
    test(&[10 .. 20, 30 .. 40], 3 .. 17, &[3 .. 20, 30 .. 40]);
    test(&[0 .. 1, 10 .. 20], 3 .. 17, &[0 .. 1, 3 .. 20]);
    test(&[0 .. 1, 10 .. 20, 30 .. 40], 3 .. 17, &[0 .. 1, 3 .. 20, 30 .. 40]);
    // overlap outside inside multiple
    test(&[10 .. 13, 15 .. 20], 3 .. 17, &[3 .. 20]);
    test(&[10 .. 13, 15 .. 20, 30 .. 40], 3 .. 17, &[3 .. 20, 30 .. 40]);
    test(&[0 .. 1, 10 .. 13, 15 .. 20], 3 .. 17, &[0 .. 1, 3 .. 20]);
    test(&[0 .. 1, 10 .. 13, 15 .. 20, 30 .. 40], 3 .. 17, &[0 .. 1, 3 .. 20, 30 .. 40]);
}

#[test]
fn api() {
    fn add<T: std::fmt::Debug>(
        xs: &mut Vec<(i32, T)>, i: usize, x: Result<T, Error>, r: Result<(), Error>,
    ) {
        match (x, r) {
            (Ok(x), Ok(())) => xs.push((i as i32 + 1, x)),
            (Err(x), Err(y)) => assert_eq!(x, y),
            (x, r) => panic!("{x:?} does not match {r:?}"),
        }
    }
    let mut data;
    let mut view;
    let mut shared;
    let mut exclusive;
    macro_rules! test {
        ($len:literal, $([$($cmd:tt)*])*) => {
            eprintln!("* {}:{}:", file!(), line!());
            #[allow(unused_assignments)]
            { shared = Vec::new(); exclusive = Vec::new(); }
            data = (1 ..= $len).collect::<Vec<i32>>();
            view = SliceCell::new(&mut data);
            $(
                eprintln!("- {}", stringify!([$($cmd)*]));
                test!($($cmd)*);
            )*
            eprintln!("- cleanup");
            test!(reset);
        };
        (reset) => {
            // First round of exclusive access.
            for (i, xs) in exclusive.iter_mut() {
                for (x, y) in xs.iter_mut().zip(*i ..) {
                    assert_eq!(*x, y);
                    *x = -y;
                }

            }
            // Round of shared access.
            for (i, xs) in shared.iter() {
                for (x, y) in xs.iter().zip(*i ..) {
                    assert_eq!(*x, y);
                }

            }
            // Second round of exclusive access.
            for (i, xs) in exclusive.iter_mut() {
                for (x, y) in xs.iter_mut().zip(*i ..) {
                    assert_eq!(*x, -y);
                    *x = y;
                }

            }
            #[allow(unused_assignments)]
            { shared = Vec::new(); exclusive = Vec::new(); }
            view.reset()
        };
        (& $i:literal $(= $err:ident)?) => {
            add(&mut shared, $i, view.get($i).map(std::slice::from_ref), test!(? $($err)?))
        };
        (&mut $i:literal $(= $err:ident)?) => {
            add(&mut exclusive, $i, view.get_mut($i).map(std::slice::from_mut), test!(? $($err)?))
        };
        (& $($mut:ident)? ! $range:tt $(= $err:ident)?) => {
            test!(& $($mut)? (flip_start $range) $(= $err)?)
        };
        (& $range:tt $(= $err:ident)?) => {
            add(&mut shared, start_index(&$range), view.get_range $range, test!(? $($err)?))
        };
        (&mut $range:tt $(= $err:ident)?) => {
            add(&mut exclusive, start_index(&$range), view.get_range_mut $range, test!(? $($err)?))
        };
        (?) => { Ok(()) };
        (? $x:ident) => { Err($x) };
    }

    test!(0, [&0 = Range][&mut 0 = Range]);
    test!(0, [&(0 .. 0)][&mut (0 .. 0)]);
    test!(0, [&(0 .. 1) = Range][&mut (0 .. 1) = Range]);

    test!(1, [&0][&0][&1 = Range]);
    test!(1, [&(0 .. 1)][&(0 ..= 0)][&(1 .. 1)][&(1 .. 2) = Range][&(1 .. 0) = Range]);
    test!(1, [&mut 0][&mut 0 = Borrow][&mut 1 = Range][reset][&mut 0]);
    test!(1, [&mut 0][reset][&mut (0 .. 1)]);

    test!(8, [&mut (0 ..= 2)][&(3 ..= 5)][&(4 ..= 6)][&mut !(6 .. 8)]);
    test!(8, [&mut (0 .. 5)][&mut 4 = Borrow][&mut 5][&mut (5 .. 8) = Borrow]);
}
