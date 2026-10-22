//! org/joml/Matrix3x2fStack.java (JOML 1.10.9)
//!
//! A `Matrix3x2f` with a fixed-size stack of saved copies of itself, which is what the GUI's
//! pose is (`GuiGraphicsExtractor.pose()`, of size 16). It extends `Matrix3x2f`, so it holds the
//! matrix it is and derefs to it. The no-argument constructor is only for deserializing, and
//! `clear` isn't called by the client, so neither is ported.

use std::ops::{Deref, DerefMut};

use crate::org::joml::matrix3x2f::Matrix3x2f;

#[derive(Clone, Debug)]
pub struct Matrix3x2fStack {
    matrix: Matrix3x2f,
    mats: Vec<Matrix3x2f>,
    curr: usize,
}

impl Matrix3x2fStack {
    /// A stack of size 1 is just the matrix itself.
    pub fn new(stack_size: i32) -> Matrix3x2fStack {
        if stack_size < 1 {
            panic!("stackSize must be >= 1");
        }
        Matrix3x2fStack {
            matrix: Matrix3x2f::new(),
            mats: vec![Matrix3x2f::new(); (stack_size - 1) as usize],
            curr: 0,
        }
    }

    pub fn push_matrix(&mut self) -> &mut Self {
        if self.curr == self.mats.len() {
            panic!("max stack size of {} reached", self.curr + 1);
        }
        self.mats[self.curr].set(&self.matrix);
        self.curr += 1;
        self
    }

    pub fn pop_matrix(&mut self) -> &mut Self {
        if self.curr == 0 {
            panic!("already at the bottom of the stack");
        }
        self.curr -= 1;
        self.matrix.set(&self.mats[self.curr]);
        self
    }
}

impl Deref for Matrix3x2fStack {
    type Target = Matrix3x2f;

    fn deref(&self) -> &Matrix3x2f {
        &self.matrix
    }
}

impl DerefMut for Matrix3x2fStack {
    fn deref_mut(&mut self) -> &mut Matrix3x2f {
        &mut self.matrix
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::Matrix3x2fStack;
    use crate::golden::{Decode, Encode, Golden, Thrown, catch};

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("org.joml.Matrix3x2fStack");
        golden.check("Matrix3x2fStack", |size: i32, program: Value| {
            run(size, &program)
        });
        golden.finish();
    }

    // Mirrors Matrix3x2fStackSuite in oracle/golden/: the stack's matrix after each op, and a
    // panicking op (or constructor) ends the program like a thrown exception.
    fn run(size: i32, program: &Value) -> Vec<Box<dyn Encode>> {
        let Ok(mut stack) = catch(|| Matrix3x2fStack::new(size)) else {
            return vec![Box::new(Thrown)];
        };
        let mut steps: Vec<Box<dyn Encode>> = Vec::new();
        for op in program.as_array().expect("a program is an array of ops") {
            let op = op.as_array().expect("an op is an array");
            if catch(|| apply(&mut stack, op)).is_err() {
                steps.push(Box::new(Thrown));
                break;
            }
            steps.push(Box::new(*stack));
        }
        steps
    }

    fn apply(stack: &mut Matrix3x2fStack, op: &[Value]) {
        let float = |index: usize| f32::decode(&op[index]);
        match op[0].as_str().expect("ops start with their name") {
            "pushMatrix" => {
                stack.push_matrix();
            }
            "popMatrix" => {
                stack.pop_matrix();
            }
            "translate" => {
                stack.translate(float(1), float(2));
            }
            "scale" => {
                stack.scale(float(1), float(2));
            }
            "rotate" => {
                stack.rotate(float(1));
            }
            name => panic!("unknown op {name}"),
        }
    }
}
