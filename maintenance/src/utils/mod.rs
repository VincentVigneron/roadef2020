use std::ops::{Add, Sub};

#[inline(always)]
pub fn add_vec_in_place<T: Copy + Add<Output = T>>(x: &mut [T], y: &[T]) {
    x.iter_mut().zip(y.iter()).for_each(|(x, &y)| *x = *x + y);
}

#[allow(dead_code)]
#[inline(always)]
pub fn sub_vec_in_place<T: Copy + Sub<Output = T>>(x: &mut [T], y: &[T]) {
    x.iter_mut().zip(y.iter()).for_each(|(x, &y)| *x = *x - y);
}

#[inline(always)]
pub fn mean_vec(dst: &mut [f64], sums: &[f64], divisors: &[usize]) {
    dst.iter_mut()
        .zip(sums.iter().zip(divisors.iter()))
        .for_each(|(mean, (sum, &nb))| *mean = sum / (nb as f64));
}

#[inline]
pub fn nth_element(values: &[f64], n: usize) -> Option<f64> {
    if n >= values.len() {
        return None;
    } else if values.len() == 1 {
        return Some(values[0]);
    } else if n == 0 {
        return values
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .copied();
    } else if n == values.len() - 1 {
        return values
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .copied();
    }

    let pivot = values[0];
    let values = &values[1..];
    let (mut smaller, mut greater): (Vec<_>, Vec<_>) = values.iter().partition(|&v| *v < pivot);

    // NOTE(vincent) : n is the position and (n+1) is the number of element.
    match smaller.len() {
        len if len > n => nth_element_in_place(&mut smaller[..], n),
        len if len == n => Some(pivot),
        _ => nth_element_in_place(&mut greater[..], n - (smaller.len() + 1)),
    }
}

#[inline]
pub fn nth_element_in_place(values: &mut [f64], n: usize) -> Option<f64> {
    if n >= values.len() {
        return None;
    } else if values.len() == 1 {
        return Some(values[0]);
    } else if n == 0 {
        return values
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .copied();
    } else if n == values.len() - 1 {
        return values
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .copied();
    }

    let pivot = values[0];
    let values = &mut values[1..];
    let i = values.iter_mut().partition_in_place(|&v| v < pivot);

    // NOTE(vincent) : n is the position and (n+1) is the number of element.
    match i {
        i if i > n => nth_element_in_place(&mut values[..i], n),
        i if i == n => Some(pivot),
        _ => nth_element_in_place(&mut values[i..], n - (i + 1)),
    }
}
