#pragma once

struct CompiledNN {
  CompiledNN();
  CompiledNN(const CompiledNN &model) = delete;
  CompiledNN(CompiledNN &&model) = delete;
  CompiledNN &operator=(const CompiledNN &model) = delete;
  CompiledNN &operator=(CompiledNN &&model) = delete;
  ~CompiledNN();

  void compile(const char* filename);

  float* input(unsigned long index);
  float* output(unsigned long index);

  unsigned long inputSize(unsigned long index);
  unsigned long outputSize(unsigned long index);

  void apply();

private:
  void *core{nullptr};
};
