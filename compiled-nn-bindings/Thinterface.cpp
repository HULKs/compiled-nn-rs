#include "Thinterface.h"
#include "CompiledNN.h"
#include "Model.h"
#include <string>

CompiledNN::CompiledNN()
    : core{reinterpret_cast<void *>(new NeuralNetwork::CompiledNN)} {}

CompiledNN::~CompiledNN() {
  delete reinterpret_cast<NeuralNetwork::CompiledNN *>(core);
}

void CompiledNN::compile(const char *filename) {
  reinterpret_cast<NeuralNetwork::CompiledNN *>(core)->compile(filename);
}

float *CompiledNN::input(std::size_t index) {
  return reinterpret_cast<NeuralNetwork::CompiledNN *>(core)
      ->input(index)
      .data();
}

float *CompiledNN::output(std::size_t index) {
  return reinterpret_cast<NeuralNetwork::CompiledNN *>(core)
      ->output(index)
      .data();
}

unsigned long CompiledNN::inputSize(unsigned long index) {
  return reinterpret_cast<NeuralNetwork::CompiledNN *>(core)
      ->input(index)
      .size();
}

unsigned long CompiledNN::outputSize(unsigned long index) {
  return reinterpret_cast<NeuralNetwork::CompiledNN *>(core)
      ->output(index)
      .size();
}

void CompiledNN::apply() {
  return reinterpret_cast<NeuralNetwork::CompiledNN *>(core)->apply();
}
