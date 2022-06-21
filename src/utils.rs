use crate::prelude::*;

pub trait IteratorUtils: Iterator {
    fn collect_svec(self) -> SVec<Self::Item>
    where
        Self: Sized,
    {
        self.collect()
    }

    fn branch<It, OutputIterA, OutputIterB>(
        self,
        condition: bool,
        a_iter: impl FnOnce(Self) -> OutputIterA,
        b_iter: impl FnOnce(Self) -> OutputIterB,
    ) -> BranchIteratorOutput<It, OutputIterA, OutputIterB>
    where
        Self: Sized,
        OutputIterA: Iterator<Item = It>,
        OutputIterB: Iterator<Item = It>,
    {
        let (a_iter, b_iter) = if condition {
            (Some(a_iter(self)), None)
        } else {
            (None, Some(b_iter(self)))
        };

        BranchIterator {
            iterator: a_iter
                .into_iter()
                .flatten()
                .chain(b_iter.into_iter().flatten()),
        }
    }
}

/// Rotates the given iterator by shifting all elements `shift` positions
/// forward. Any elements that would be out of bounds are instead put at the
/// beginning.
///
/// This method requires passing the `len` as a separate parameter. This is
/// often known beforehand or can be found by calling .size_hint() for an
/// ExactSizeIterator.
pub fn rotate_iter<T>(
    it: impl Iterator<Item = T> + Clone,
    shift: usize,
    len: usize,
) -> impl Iterator<Item = T> {
    it.cycle().dropping(shift).take(len)
}

impl<T: ?Sized> IteratorUtils for T where T: Iterator {}

pub trait SliceUtils<T> {
    /// Same as .iter().copied(), but doesn't trigger rustfmt line breaks
    fn iter_cpy(&self) -> std::iter::Copied<std::slice::Iter<'_, T>>;
}

impl<T: Copy> SliceUtils<T> for [T] {
    fn iter_cpy(&self) -> std::iter::Copied<std::slice::Iter<'_, T>> {
        self.iter().copied()
    }
}

pub struct BranchIterator<I, It>
where
    I: Iterator<Item = It>,
{
    iterator: I,
}

impl<I, It> Iterator for BranchIterator<I, It>
where
    I: Iterator<Item = It>,
{
    type Item = It;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

type BranchIteratorOutput<It, OutA, OutB> = BranchIterator<
    std::iter::Chain<
        std::iter::Flatten<std::option::IntoIter<OutA>>,
        std::iter::Flatten<std::option::IntoIter<OutB>>,
    >,
    It,
>;

#[test]
pub fn test() {
    fn tuple_windows(circular: bool) -> impl Iterator<Item = (i32, i32)> {
        vec![1, 2, 3, 4, 5].into_iter().branch(
            circular,
            |it| it.circular_tuple_windows(),
            |it| it.tuple_windows(),
        )
    }

    assert_eq!(
        tuple_windows(false).collect_vec(),
        &[(1, 2), (2, 3), (3, 4), (4, 5)]
    );
    assert_eq!(
        tuple_windows(true).collect_vec(),
        &[(1, 2), (2, 3), (3, 4), (4, 5), (5, 1)]
    );
}

/// Transmutes a vector of `T`s into a vector of `U`s.
///
/// # Safety
/// This is only safe when `T` and `U` have the same size, plus all the
/// additional safety considerations required when calling `transmute::<T,U>`
pub unsafe fn transmute_vec<T, U>(v: Vec<T>) -> Vec<U> {
    let mut v = std::mem::ManuallyDrop::new(v);
    let ptr = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();

    Vec::from_raw_parts(ptr as *mut U, len, cap)
}
