// Copyright (c) 2022, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

#[allow(unused)]
/// Find k-means for a sorted slice of integers that can be summed in `i64`.
pub fn kmeans<T, const N: usize>(data: &[T]) -> [T; N]
where
  T: Copy,
  T: Into<i64>,
  T: PartialEq,
  T: PartialOrd,
  i64: TryInto<T>,
  <i64 as std::convert::TryInto<T>>::Error: std::fmt::Debug,
{
  let mut low = {
    let mut i = 0..N;
    [(); N].map(|_| (i.next().unwrap() * (data.len() - 1)) / (N - 1))
  };
  let mut centroids = low.map(|i| unsafe { *data.get_unchecked(i) });
  let mut high = low;
  let mut sum = [0i64; N];
  high[N - 1] = data.len();
  sum[N - 1] = centroids[N - 1].into();

  let data_to = |n: usize| unsafe { data.get_unchecked(..n) }.iter();
  let data_from = |n: usize| unsafe { data.get_unchecked(n..) }.iter();

  // Constrain complexity to O(n log n)
  let limit = 2 * (usize::BITS - data.len().leading_zeros());
  for _ in 0..limit {
    for (i, (threshold, (low, high))) in
      (centroids.iter().skip(1).zip(&centroids))
        .map(|(&c1, &c2)| unsafe {
          ((c1.into() + c2.into() + 1) >> 1).try_into().unwrap_unchecked()
        })
        .zip(low.iter_mut().skip(1).zip(&mut high))
        .enumerate()
    {
      let mut n = *high;
      let mut s = sum[i];
      for &d in data_to(n).rev().take_while(|&d| *d > threshold) {
        s -= d.into();
        n -= 1;
      }
      for &d in data_from(n).take_while(|&d| *d <= threshold) {
        s += d.into();
        n += 1;
      }
      *high = n;
      sum[i] = s;

      let mut n = *low;
      let mut s = sum[i + 1];
      for &d in data_from(n).take_while(|&d| *d < threshold) {
        s -= d.into();
        n += 1;
      }
      for &d in data_to(n).rev().take_while(|&d| *d >= threshold) {
        s += d.into();
        n -= 1;
      }
      *low = n;
      sum[i + 1] = s;
    }
    let mut changed = false;
    for (c, (sum, (high, low))) in
      centroids.iter_mut().zip(sum.iter().zip(high.iter().zip(low)))
    {
      let count = (high - low) as i64;
      if count == 0 {
        continue;
      }
      let new_centroid = unsafe {
        ((sum + (count >> 1)).saturating_div(count))
          .try_into()
          .unwrap_unchecked()
      };
      changed |= *c != new_centroid;
      *c = new_centroid;
    }
    if !changed {
      break;
    }
  }

  centroids
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn three_means() {
    let mut data = [1, 2, 3, 10, 11, 12, 20, 21, 22];
    data.sort_unstable();
    let centroids = kmeans(&data);
    assert_eq!(centroids, [2, 11, 21]);
  }

  #[test]
  fn four_means() {
    let mut data = [30, 31, 32, 1, 2, 3, 10, 11, 12, 20, 21, 22];
    data.sort_unstable();
    let centroids = kmeans(&data);
    assert_eq!(centroids, [2, 11, 21, 31]);
  }
}
