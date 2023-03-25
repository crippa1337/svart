# A script to convert the weights and biases from the marlinflow json format to the bin format used by the engine.
# This script was heavily inspired by Dede, the author of Carp.


import sys
import json
import struct

FEATURES = 768
HIDDEN = 256
QA = 255
QB = 64
QAB = QA * QB

def write_bytes(array, path):
    with open(path, 'wb') as file:
        for num in array:
            file.write(struct.pack('<h', num))

def convert_weight(json_weight, stride, length, q, transpose):
    weights = [0 for _ in range(length)]

    for i, row in enumerate(json_weight):
        for j, weight in enumerate(row):
            if transpose:
                index = j * stride + i
            else:
                index = i * stride + j
            
            weights[index] = int(weight * q)

    return weights

def convert_bias(json_bias, q):
    biases = []

    for bias in json_bias:
        value = int(bias * q)
        biases.append(value)
    
    return biases

def main():
    if len(sys.argv) != 2:
        print("Usage: python marlinflow_to_bin.py <json_file>")
        sys.exit(1)

    json_file = sys.argv[1]
    with open(json_file, 'r') as file:
        data = json.load(file)

    for key, value in data.items():
        if key == "ft.weight":
            weights = convert_weight(value, HIDDEN, HIDDEN * FEATURES, QA, True)
            write_bytes(weights, "../src/engine/nnue/net/feature_weights.bin")
        elif key == "ft.bias":
            biases = convert_bias(value, QA)
            write_bytes(biases, "../src/engine/nnue/net/feature_bias.bin")
        elif key == "out.weight":
            weights = convert_weight(value, HIDDEN * 2, HIDDEN * 2, QB, False)
            write_bytes(weights, "../src/engine/nnue/net/output_weights.bin")
        elif key == "out.bias":
            biases = convert_bias(value, QAB)    
            write_bytes(biases, "../src/engine/nnue/net/output_bias.bin")
        
if __name__ == "__main__":
    main()