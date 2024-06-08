#![allow(dead_code)]
use computegraph::*;

#[derive(Debug)]
pub struct TestNodeConstant {
    value: usize,
}

impl TestNodeConstant {
    pub const fn new(value: usize) -> Self {
        Self { value }
    }
}

#[node(TestNodeConstant)]
fn run(&self) -> usize {
    self.value
}

#[derive(Debug)]
pub struct TestNodeAddition {}

impl TestNodeAddition {
    pub const fn new() -> Self {
        Self {}
    }
}

#[node(TestNodeAddition)]
fn run(&self, a: &usize, b: &usize) -> usize {
    *a + *b
}

#[derive(Debug)]
pub struct TestNodeNumToString {}

impl TestNodeNumToString {
    pub const fn new() -> Self {
        Self {}
    }
}

#[node(TestNodeNumToString)]
fn run(&self, input: &usize) -> String {
    input.to_string()
}
