#!/bin/bash
# Real Human Test: GPT-OSS with MoE CPU Offloading
# Let's see if this actually generates text!

echo "========================================="
echo "GPT-OSS MoE Test - Can it actually work?"
echo "========================================="
echo ""
echo "Model: GPT-OSS 20B Q4_K_M (11.6GB)"
echo "Hardware: RTX 3060 (4GB VRAM)"
echo "Test: Generate a simple response"
echo ""
echo "Starting generation..."
echo ""

NO_COLOR=1 SHIMMY_BASE_GGUF=./models/gpt-oss-20b-Q4_K_M.gguf \
./target/release/shimmy.exe --cpu-moe generate phi3-lora \
--prompt "Say hello and introduce yourself in one sentence." \
--max-tokens 50

echo ""
echo ""
echo "========================================="
echo "Test complete!"
echo "========================================="
