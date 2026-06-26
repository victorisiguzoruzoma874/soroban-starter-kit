import json
import os
import sys

def check_wasm_sizes(baseline_file, growth_factor):
    with open(baseline_file, 'r') as f:
        baseline_sizes = json.load(f)

    failed = False
    for wasm_file in os.listdir('target/wasm32-unknown-unknown/release'):
        if wasm_file.endswith('.wasm'):
            contract_name = wasm_file.replace('.wasm', '')
            if contract_name in baseline_sizes:
                baseline_size = baseline_sizes[contract_name]
                current_size = os.path.getsize(f'target/wasm32-unknown-unknown/release/{wasm_file}')
                max_size = baseline_size * float(growth_factor)
                print(f'{contract_name}: baseline={baseline_size}, current={current_size}, max={max_size}')
                if current_size > max_size:
                    print(f'ERROR: {contract_name} has grown by more than {growth_factor}%')
                    failed = True

    if failed:
        sys.exit(1)

if __name__ == '__main__':
    check_wasm_sizes(sys.argv[1], sys.argv[2])