#!/usr/bin/env python3
"""
Analyze quantization test results and extract performance metrics
"""
import json
import re
import os
from pathlib import Path
from collections import defaultdict

RESULTS_DIR = Path("./quantization-test-results")

def parse_result_file(filepath):
    """Extract metrics from a test result JSON file"""
    with open(filepath, 'r') as f:
        content = f.read()
<<<<<<< HEAD
<<<<<<< HEAD

=======
    
>>>>>>> main
=======
    
>>>>>>> main
    metrics = {
        'model': None,
        'config': None,
        'run': None,
        'model_size_mb': None,
        'vram_mb': None,
        'load_time_s': None,
        'generated_tokens': 0,
        'generation_time_s': None,
        'tokens_per_second': None,
        'output_text': None
    }
<<<<<<< HEAD
<<<<<<< HEAD

=======
    
>>>>>>> main
=======
    
>>>>>>> main
    # Extract from filename
    filename = filepath.stem
    parts = filename.rsplit('-run', 1)
    if len(parts) == 2:
        metrics['model'] = parts[0].replace('-cpu-offload', '').replace('-baseline', '')
        metrics['config'] = 'cpu-offload' if '-cpu-offload-' in filename else 'baseline'
        metrics['run'] = int(parts[1])
<<<<<<< HEAD
<<<<<<< HEAD

=======
    
>>>>>>> main
=======
    
>>>>>>> main
    # Extract model size from llama.cpp output
    model_size_match = re.search(r'llama_model_load.*?(\d+(?:\.\d+)?)\s*(?:MiB|GiB)', content)
    if model_size_match:
        size = float(model_size_match.group(1))
        unit = model_size_match.group(0)
        if 'GiB' in unit:
            size *= 1024
        metrics['model_size_mb'] = size
<<<<<<< HEAD
<<<<<<< HEAD

    # Extract VRAM usage (CUDA0 buffer sizes only - avoid counting per-layer allocations)
    # We want: model buffer + KV cache buffer + compute buffer
    vram_total = 0

=======
=======
>>>>>>> main
    
    # Extract VRAM usage (CUDA0 buffer sizes only - avoid counting per-layer allocations)
    # We want: model buffer + KV cache buffer + compute buffer
    vram_total = 0
    
<<<<<<< HEAD
>>>>>>> main
=======
>>>>>>> main
    # Model buffer
    model_buf = re.search(r'CUDA0 model buffer size\s*=\s*(\d+(?:\.\d+)?)\s*MiB', content)
    if model_buf:
        vram_total += float(model_buf.group(1))
<<<<<<< HEAD
<<<<<<< HEAD

=======
    
>>>>>>> main
=======
    
>>>>>>> main
    # KV cache buffer
    kv_buf = re.search(r'CUDA0 KV buffer size\s*=\s*(\d+(?:\.\d+)?)\s*MiB', content)
    if kv_buf:
        vram_total += float(kv_buf.group(1))
<<<<<<< HEAD
<<<<<<< HEAD

=======
    
>>>>>>> main
=======
    
>>>>>>> main
    # Compute buffer
    compute_buf = re.search(r'CUDA0 compute buffer size\s*=\s*(\d+(?:\.\d+)?)\s*MiB', content)
    if compute_buf:
        vram_total += float(compute_buf.group(1))
<<<<<<< HEAD
<<<<<<< HEAD

    if vram_total > 0:
        metrics['vram_mb'] = vram_total

=======
=======
>>>>>>> main
    
    if vram_total > 0:
        metrics['vram_mb'] = vram_total
    
<<<<<<< HEAD
>>>>>>> main
=======
>>>>>>> main
    # Extract generation metrics
    # Look for token generation in output
    output_match = re.search(r'graph splits.*?\n(.+?)$', content, re.DOTALL)
    if output_match:
        output_text = output_match.group(1).strip()
        # Count tokens (rough estimate: ~4 chars per token)
        metrics['output_text'] = output_text[:200]  # First 200 chars
        metrics['generated_tokens'] = len(output_text.split())
<<<<<<< HEAD
<<<<<<< HEAD

    # Try to estimate TPS from timing if available
    # This is rough - llama.cpp doesn't always output timing

=======
=======
>>>>>>> main
    
    # Try to estimate TPS from timing if available
    # This is rough - llama.cpp doesn't always output timing
    
<<<<<<< HEAD
>>>>>>> main
=======
>>>>>>> main
    return metrics

def main():
    results = []
<<<<<<< HEAD
<<<<<<< HEAD

=======
    
>>>>>>> main
=======
    
>>>>>>> main
    # Parse all result files
    for filepath in sorted(RESULTS_DIR.glob("*.json")):
        if filepath.name == "SUMMARY.md":
            continue
        metrics = parse_result_file(filepath)
        results.append(metrics)
        print(f"Parsed: {filepath.name}")
<<<<<<< HEAD
<<<<<<< HEAD

=======
    
>>>>>>> main
=======
    
>>>>>>> main
    # Group by model and config
    grouped = defaultdict(lambda: defaultdict(list))
    for r in results:
        if r['model']:
            grouped[r['model']][r['config']].append(r)
<<<<<<< HEAD
<<<<<<< HEAD

=======
    
>>>>>>> main
=======
    
>>>>>>> main
    # Calculate averages
    print("\n" + "="*80)
    print("QUANTIZATION TEST RESULTS SUMMARY")
    print("="*80)
<<<<<<< HEAD
<<<<<<< HEAD

=======
    
>>>>>>> main
=======
    
>>>>>>> main
    for model in sorted(grouped.keys()):
        print(f"\n{'='*80}")
        print(f"MODEL: {model}")
        print(f"{'='*80}")
<<<<<<< HEAD
<<<<<<< HEAD

=======
        
>>>>>>> main
=======
        
>>>>>>> main
        for config in ['baseline', 'cpu-offload']:
            runs = grouped[model][config]
            if not runs:
                continue
<<<<<<< HEAD
<<<<<<< HEAD

            print(f"\n  {config.upper()}:")

            # Calculate averages
            avg_vram = sum(r['vram_mb'] for r in runs if r['vram_mb']) / len([r for r in runs if r['vram_mb']]) if any(r['vram_mb'] for r in runs) else 0
            avg_tokens = sum(r['generated_tokens'] for r in runs) / len(runs)

            print(f"    Runs: {len(runs)}")
            print(f"    Avg VRAM: {avg_vram:.1f} MB ({avg_vram/1024:.2f} GB)")
            print(f"    Avg tokens generated: {avg_tokens:.0f}")

            # Show sample output
            if runs[0]['output_text']:
                print(f"    Sample output: {runs[0]['output_text'][:100]}...")

=======
=======
>>>>>>> main
            
            print(f"\n  {config.upper()}:")
            
            # Calculate averages
            avg_vram = sum(r['vram_mb'] for r in runs if r['vram_mb']) / len([r for r in runs if r['vram_mb']]) if any(r['vram_mb'] for r in runs) else 0
            avg_tokens = sum(r['generated_tokens'] for r in runs) / len(runs)
            
            print(f"    Runs: {len(runs)}")
            print(f"    Avg VRAM: {avg_vram:.1f} MB ({avg_vram/1024:.2f} GB)")
            print(f"    Avg tokens generated: {avg_tokens:.0f}")
            
            # Show sample output
            if runs[0]['output_text']:
                print(f"    Sample output: {runs[0]['output_text'][:100]}...")
    
<<<<<<< HEAD
>>>>>>> main
=======
>>>>>>> main
    # Save detailed results
    output_file = RESULTS_DIR / "analysis.json"
    with open(output_file, 'w') as f:
        json.dump({
            'summary': {model: {config: {
                'runs': len(runs),
                'avg_vram_mb': sum(r['vram_mb'] for r in runs if r['vram_mb']) / len([r for r in runs if r['vram_mb']]) if any(r['vram_mb'] for r in runs) else 0,
                'avg_tokens': sum(r['generated_tokens'] for r in runs) / len(runs)
            } for config, runs in configs.items()} for model, configs in grouped.items()},
            'detailed_results': results
        }, f, indent=2)
<<<<<<< HEAD
<<<<<<< HEAD

=======
    
>>>>>>> main
=======
    
>>>>>>> main
    print(f"\n{'='*80}")
    print(f"Detailed analysis saved to: {output_file}")
    print(f"{'='*80}\n")

if __name__ == "__main__":
    main()
