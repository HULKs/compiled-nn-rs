use std::{
    ffi::CString,
    path::Path,
    slice::{from_raw_parts, from_raw_parts_mut},
};

use compiled_nn_bindings;

pub struct CompiledNN {
    core: compiled_nn_bindings::CompiledNN,
}

unsafe impl Send for CompiledNN {}

impl Default for CompiledNN {
    fn default() -> Self {
        Self {
            core: unsafe { compiled_nn_bindings::CompiledNN::new() },
        }
    }
}

impl Drop for CompiledNN {
    fn drop(&mut self) {
        unsafe { self.core.destruct() }
    }
}

impl CompiledNN {
    pub fn compile(&mut self, filename: impl AsRef<Path>) {
        let filename =
            CString::new(filename.as_ref().to_str().unwrap()).expect("CString::new failed");
        unsafe { self.core.compile(filename.as_ptr()) }
    }

    pub fn input(&self, index: usize) -> Tensor {
        unsafe {
            let input = self.core.input(index as u64);
            Tensor {
                data: from_raw_parts(input.data, input.data_size as usize),
                dimensions: from_raw_parts(input.dimensions, input.dimensions_size as usize),
            }
        }
    }

    pub fn input_mut(&mut self, index: usize) -> TensorMut {
        unsafe {
            let input = self.core.input_mut(index as u64);
            TensorMut {
                data: from_raw_parts_mut(input.data, input.data_size as usize),
                dimensions: from_raw_parts(input.dimensions, input.dimensions_size as usize),
            }
        }
    }

    pub fn output(&self, index: usize) -> Tensor {
        unsafe {
            let output = self.core.output(index as u64);
            Tensor {
                data: from_raw_parts(output.data, output.data_size as usize),
                dimensions: from_raw_parts(output.dimensions, output.dimensions_size as usize),
            }
        }
    }

    pub fn output_mut(&mut self, index: usize) -> TensorMut {
        unsafe {
            let output = self.core.output_mut(index as u64);
            TensorMut {
                data: from_raw_parts_mut(output.data, output.data_size as usize),
                dimensions: from_raw_parts(output.dimensions, output.dimensions_size as usize),
            }
        }
    }

    pub fn apply(&mut self) {
        unsafe { self.core.apply() }
    }
}

#[derive(Debug)]
pub struct Tensor<'a> {
    pub data: &'a [f32],
    pub dimensions: &'a [u32],
}

#[derive(Debug)]
pub struct TensorMut<'a> {
    pub data: &'a mut [f32],
    pub dimensions: &'a [u32],
}
