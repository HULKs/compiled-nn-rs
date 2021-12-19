use std::{
    ffi::CString,
    path::Path,
    slice::{from_raw_parts, from_raw_parts_mut},
};

use compiled_nn_bindings;

pub struct CompiledNN {
    core: compiled_nn_bindings::CompiledNN,
}

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
    pub fn compile<P>(&mut self, filename: P)
    where
        P: AsRef<Path>,
    {
        let filename =
            CString::new(filename.as_ref().to_str().unwrap()).expect("CString::new failed");
        unsafe { self.core.compile(filename.as_ptr()) }
    }

    pub fn input(&mut self, index: usize) -> &mut [f32] {
        unsafe {
            let input = self.core.input(index as u64);
            let input_size = self.core.inputSize(index as u64);
            from_raw_parts_mut(input, input_size as usize)
        }
    }

    pub fn output(&mut self, index: usize) -> &[f32] {
        unsafe {
            let output = self.core.output(index as u64);
            let output_size = self.core.outputSize(index as u64);
            from_raw_parts(output, output_size as usize)
        }
    }

    pub fn apply(&mut self) {
        unsafe { self.core.apply() }
    }
}
