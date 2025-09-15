#!/bin/bash
# Shimmy Lightweight Performance Benchmark
# Zero dependencies - uses only system tools

set -e

SHIMMY_URL="http://localhost:11434"
OUTPUT_FILE="benchmark_results.json"
NUM_REQUESTS=10
MODEL_NAME=""

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  --url URL       Shimmy server URL (default: http://localhost:11434)"
    echo "  --model NAME    Model name to test (auto-detect if not specified)"
    echo "  --requests N    Number of test requests (default: 10)"
    echo "  --output FILE   Output file (default: benchmark_results.json)"
    echo "  --help         Show this help"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --url)
            SHIMMY_URL="$2"
            shift 2
            ;;
        --model)
            MODEL_NAME="$2"
            shift 2
            ;;
        --requests)
            NUM_REQUESTS="$2"
            shift 2
            ;;
        --output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

echo "🚀 Shimmy Lightweight Benchmark"
echo "Target: $SHIMMY_URL"
echo "Requests: $NUM_REQUESTS"

# Check if Shimmy is running
if ! curl -s "$SHIMMY_URL/health" > /dev/null 2>&1; then
    echo "❌ Cannot connect to Shimmy at $SHIMMY_URL"
    exit 1
fi

echo "✅ Shimmy is running"

# Get model name if not specified
if [ -z "$MODEL_NAME" ]; then
    MODEL_NAME=$(curl -s "$SHIMMY_URL/v1/models" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
    if [ -z "$MODEL_NAME" ]; then
        echo "❌ No models found"
        exit 1
    fi
fi

echo "📦 Using model: $MODEL_NAME"

# Get system info
get_system_info() {
    echo "{"
    echo "  \"timestamp\": \"$(date -Iseconds)\","
    echo "  \"hostname\": \"$(hostname)\","
    echo "  \"os\": \"$(uname -s)\","
    echo "  \"arch\": \"$(uname -m)\","
    echo "  \"cpu_cores\": $(nproc 2>/dev/null || echo "null"),"
    
    # Memory info
    if command -v free >/dev/null 2>&1; then
        TOTAL_MEM=$(free -b | awk '/^Mem:/ {print $2}')
        echo "  \"memory_total_bytes\": $TOTAL_MEM,"
    fi
    
    # GPU info (if nvidia-smi available)
    if command -v nvidia-smi >/dev/null 2>&1; then
        GPU_NAME=$(nvidia-smi --query-gpu=name --format=csv,noheader | head -1)
        GPU_MEM=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits | head -1)
        echo "  \"gpu_name\": \"$GPU_NAME\","
        echo "  \"gpu_memory_mb\": $GPU_MEM,"
    fi
    
    echo "  \"shimmy_url\": \"$SHIMMY_URL\","
    echo "  \"model_name\": \"$MODEL_NAME\""
    echo "}"
}

# Performance test
run_performance_test() {
    echo "🧪 Running performance test..."
    
    local total_time=0
    local successful_requests=0
    local failed_requests=0
    local total_tokens=0
    
    echo "["
    
    for i in $(seq 1 $NUM_REQUESTS); do
        echo -n "Request $i/$NUM_REQUESTS... "
        
        start_time=$(date +%s.%N)
        
        # Make request with timeout
        response=$(timeout 30 curl -s -X POST "$SHIMMY_URL/v1/chat/completions" \
            -H "Content-Type: application/json" \
            -d "{
                \"model\": \"$MODEL_NAME\",
                \"messages\": [{\"role\": \"user\", \"content\": \"Hello, how are you?\"}],
                \"max_tokens\": 100,
                \"temperature\": 0.7
            }" 2>/dev/null)
        
        end_time=$(date +%s.%N)
        request_time=$(echo "$end_time - $start_time" | bc -l 2>/dev/null || echo "0")
        
        if [ $? -eq 0 ] && echo "$response" | grep -q "choices"; then
            # Success
            successful_requests=$((successful_requests + 1))
            tokens=$(echo "$response" | grep -o '"content":"[^"]*"' | wc -c)
            tokens=$((tokens / 10))  # Rough token estimate
            total_tokens=$((total_tokens + tokens))
            echo "✅ ${request_time}s"
            
            [ $i -gt 1 ] && echo ","
            echo "    {\"request\": $i, \"time\": $request_time, \"success\": true, \"tokens\": $tokens}"
        else
            # Failure
            failed_requests=$((failed_requests + 1))
            echo "❌ ${request_time}s"
            
            [ $i -gt 1 ] && echo ","
            echo "    {\"request\": $i, \"time\": $request_time, \"success\": false, \"tokens\": 0}"
        fi
        
        total_time=$(echo "$total_time + $request_time" | bc -l 2>/dev/null || echo "$total_time")
        
        # Small delay
        sleep 0.5
    done
    
    echo "]"
    
    # Calculate averages
    if [ $successful_requests -gt 0 ]; then
        avg_time=$(echo "scale=3; $total_time / $NUM_REQUESTS" | bc -l 2>/dev/null || echo "0")
        tokens_per_sec=$(echo "scale=1; $total_tokens / $total_time" | bc -l 2>/dev/null || echo "0")
        success_rate=$(echo "scale=2; $successful_requests * 100 / $NUM_REQUESTS" | bc -l 2>/dev/null || echo "0")
        
        echo ""
        echo "📊 Results:"
        echo "  Success Rate: ${success_rate}%"
        echo "  Avg Response Time: ${avg_time}s"
        echo "  Tokens/sec: $tokens_per_sec"
        echo "  Total Requests: $NUM_REQUESTS"
        echo "  Successful: $successful_requests"
        echo "  Failed: $failed_requests"
    fi
}

# Get baseline system metrics
get_baseline_metrics() {
    echo "📊 System Metrics:"
    
    # CPU usage
    if command -v top >/dev/null 2>&1; then
        CPU_USAGE=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
        echo "  CPU Usage: ${CPU_USAGE}%"
    fi
    
    # Memory usage
    if command -v free >/dev/null 2>&1; then
        MEM_USAGE=$(free | grep Mem | awk '{printf "%.1f", $3/$2 * 100.0}')
        echo "  Memory Usage: ${MEM_USAGE}%"
    fi
    
    # GPU metrics (if available)
    if command -v nvidia-smi >/dev/null 2>&1; then
        GPU_UTIL=$(nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader,nounits | head -1)
        GPU_MEM_USED=$(nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits | head -1)
        GPU_POWER=$(nvidia-smi --query-gpu=power.draw --format=csv,noheader,nounits | head -1)
        
        echo "  GPU Utilization: ${GPU_UTIL}%"
        echo "  GPU Memory Used: ${GPU_MEM_USED}MB"
        echo "  GPU Power Draw: ${GPU_POWER}W"
    fi
    
    # Shimmy process info
    SHIMMY_PID=$(pgrep shimmy 2>/dev/null || echo "")
    if [ -n "$SHIMMY_PID" ]; then
        if command -v ps >/dev/null 2>&1; then
            SHIMMY_MEM=$(ps -p "$SHIMMY_PID" -o rss= 2>/dev/null || echo "0")
            SHIMMY_MEM_MB=$(echo "scale=1; $SHIMMY_MEM / 1024" | bc -l 2>/dev/null || echo "0")
            echo "  Shimmy Memory: ${SHIMMY_MEM_MB}MB"
        fi
    fi
}

# Main execution
main() {
    echo ""
    get_baseline_metrics
    echo ""
    
    # Run the performance test
    RESULTS=$(run_performance_test)
    
    # Generate JSON output
    {
        get_system_info | head -n -1
        echo "  \"benchmark_results\": $RESULTS"
        echo "}"
    } > "$OUTPUT_FILE"
    
    echo ""
    echo "💾 Results saved to: $OUTPUT_FILE"
}

main