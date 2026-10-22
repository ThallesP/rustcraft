//! org/joml/Matrix4fStack.java (JOML 1.10.9)
//!
//! A `Matrix4f` with a fixed-size stack of saved copies of itself, which is what the game's
//! model-view stack is (`RenderSystem.getModelViewStack()`). It extends `Matrix4f`, so it holds
//! the matrix it is and derefs to it. The no-argument constructor is only for deserializing, and
//! isn't ported.

use std::ops::{Deref, DerefMut};

use crate::org::joml::matrix4f::Matrix4f;

#[derive(Clone, Debug)]
pub struct Matrix4fStack {
    matrix: Matrix4f,
    mats: Vec<Matrix4f>,
    curr: usize,
}

impl Matrix4fStack {
    /// A stack of size 1 is just the matrix itself.
    pub fn new(stack_size: i32) -> Matrix4fStack {
        if stack_size < 1 {
            panic!("stackSize must be >= 1");
        }
        Matrix4fStack {
            matrix: Matrix4f::new(),
            mats: vec![Matrix4f::new(); (stack_size - 1) as usize],
            curr: 0,
        }
    }

    pub fn clear(&mut self) -> &mut Self {
        self.curr = 0;
        self.matrix.identity();
        self
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

impl Deref for Matrix4fStack {
    type Target = Matrix4f;

    fn deref(&self) -> &Matrix4f {
        &self.matrix
    }
}

impl DerefMut for Matrix4fStack {
    fn deref_mut(&mut self) -> &mut Matrix4f {
        &mut self.matrix
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::Matrix4fStack;
    use crate::golden::{Decode, Encode, Golden, Thrown, catch};
    use crate::org::joml::matrix4f::Matrix4f;
    use crate::org::joml::quaternionf::Quaternionf;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("org.joml.Matrix4fStack");
        golden.check("Matrix4fStack", |size: i32, program: Value| {
            run(size, &program)
        });
        golden.finish();
    }

    // Mirrors Matrix4fStackSuite in oracle/golden/: the stack's matrix after each op, and a
    // panicking op (or constructor) ends the program like a thrown exception.
    fn run(size: i32, program: &Value) -> Vec<Box<dyn Encode>> {
        let Ok(mut stack) = catch(|| Matrix4fStack::new(size)) else {
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

    fn apply(stack: &mut Matrix4fStack, op: &[Value]) {
        let float = |index: usize| f32::decode(&op[index]);
        match op[0].as_str().expect("ops start with their name") {
            "pushMatrix" => {
                stack.push_matrix();
            }
            "popMatrix" => {
                stack.pop_matrix();
            }
            "clear" => {
                stack.clear();
            }
            "identity" => {
                stack.identity();
            }
            "translate" => {
                stack.translate(float(1), float(2), float(3));
            }
            "scale" => {
                stack.scale(float(1), float(2), float(3));
            }
            "rotateY" => {
                stack.rotate_y(float(1));
            }
            "rotate" => {
                stack.rotate(&Quaternionf::decode(&op[1]));
            }
            "mul" => {
                stack.mul(&Matrix4f::decode(&op[1]));
            }
            "set" => {
                stack.set(&Matrix4f::decode(&op[1]));
            }
            name => panic!("unknown op {name}"),
        }
    }
}
