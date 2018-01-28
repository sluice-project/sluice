#![feature(test)]

extern crate test;

pub fn add_two(a: i32) -> i32 {
    a + 2
}

use test::Bencher;

#[bench]
fn bench_add_two(b: &mut Bencher) {
    b.iter(|| add_two(2));
}
