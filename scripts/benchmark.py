#!/usr/bin/env python3
"""
Shimmy Performance Benchmark Tool

Measures actual GPU power consumption, memory usage, and performance metrics
for Shimmy vs other inference servers like Ollama.
"""

import json
import time
import subprocess
import psutil
import requests
import sys
import argparse
from pathlib import Path
from datetime import datetime
import threading
import csv

class ShimmyBenchmark:
    def __init__(self, shimmy_url="http://localhost:11434", model_name=None):
        self.shimmy_url = shimmy_url
        self.model_name = model_name
        self.results = {
            "timestamp": datetime.now().isoformat(),
            "system_info": self.get_system_info(),
            "measurements": []
        }
        self.monitoring = False
        
    def get_system_info(self):
        """Collect system information"""
        return {
            "cpu": {
                "count": psutil.cpu_count(),
                "freq": psutil.cpu_freq()._asdict() if psutil.cpu_freq() else None,
            },
            "memory": {
                "total": psutil.virtual_memory().total,
                "available": psutil.virtual_memory().available,
            },
            "gpu": self.get_gpu_info()
        }
    
    def get_gpu_info(self):
        """Get GPU information using nvidia-smi if available"""
        try:
            result = subprocess.run([
                "nvidia-smi", "--query-gpu=name,memory.total,power.max_limit",
                "--format=csv,noheader,nounits"
            ], capture_output=True, text=True, check=True)
            
            gpus = []
            for line in result.stdout.strip().split('\n'):
                if line:
                    parts = line.split(', ')
                    gpus.append({
                        "name": parts[0],
                        "memory_total_mb": int(parts[1]),
                        "power_limit_w": float(parts[2])
                    })
            return gpus
        except (subprocess.CalledProcessError, FileNotFoundError):
            return []
    
    def get_shimmy_process_info(self):
        """Find and get Shimmy process information"""
        for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
            try:
                if 'shimmy' in proc.info['name'].lower():
                    return psutil.Process(proc.info['pid'])
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue
        return None
    
    def monitor_resources(self, duration_seconds=60, interval=1):
        """Monitor system resources continuously"""
        measurements = []
        shimmy_proc = self.get_shimmy_process_info()
        
        if not shimmy_proc:
            print("Warning: Could not find Shimmy process for detailed monitoring")
        
        for i in range(duration_seconds):
            if not self.monitoring:
                break
                
            measurement = {
                "timestamp": time.time(),
                "cpu_percent": psutil.cpu_percent(interval=0.1),
                "memory_percent": psutil.virtual_memory().percent,
                "gpu": self.get_gpu_metrics()
            }
            
            if shimmy_proc:
                try:
                    shimmy_proc.cpu_percent()  # First call returns 0, so call it
                    time.sleep(0.1)
                    measurement["shimmy"] = {
                        "cpu_percent": shimmy_proc.cpu_percent(),
                        "memory_mb": shimmy_proc.memory_info().rss / (1024 * 1024),
                        "threads": shimmy_proc.num_threads()
                    }
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    measurement["shimmy"] = None
            
            measurements.append(measurement)
            time.sleep(interval)
        
        return measurements
    
    def get_gpu_metrics(self):
        """Get current GPU metrics"""
        try:
            result = subprocess.run([
                "nvidia-smi", "--query-gpu=utilization.gpu,memory.used,memory.total,power.draw",
                "--format=csv,noheader,nounits"
            ], capture_output=True, text=True, check=True)
            
            gpus = []
            for line in result.stdout.strip().split('\n'):
                if line:
                    parts = line.split(', ')
                    gpus.append({
                        "utilization_percent": int(parts[0]),
                        "memory_used_mb": int(parts[1]),
                        "memory_total_mb": int(parts[2]),
                        "power_draw_w": float(parts[3])
                    })
            return gpus
        except (subprocess.CalledProcessError, FileNotFoundError):
            return []
    
    def test_inference_performance(self, prompt="Hello, how are you?", num_requests=10):
        """Test inference performance with actual requests"""
        print(f"Testing inference performance with {num_requests} requests...")
        
        results = []
        
        # Start monitoring in background
        self.monitoring = True
        monitor_thread = threading.Thread(
            target=lambda: self.results.update({
                "resource_monitoring": self.monitor_resources(duration_seconds=60)
            })
        )
        monitor_thread.start()
        
        # Warm up
        try:
            self.make_request(prompt)
            time.sleep(1)
        except Exception as e:
            print(f"Warmup request failed: {e}")
        
        # Actual performance test
        for i in range(num_requests):
            print(f"Request {i+1}/{num_requests}")
            start_time = time.time()
            
            try:
                response = self.make_request(prompt)
                end_time = time.time()
                
                results.append({
                    "request_id": i + 1,
                    "response_time": end_time - start_time,
                    "success": True,
                    "tokens_generated": len(response.get("choices", [{}])[0].get("message", {}).get("content", "").split()),
                    "model_used": response.get("model", "unknown")
                })
                
            except Exception as e:
                end_time = time.time()
                results.append({
                    "request_id": i + 1,
                    "response_time": end_time - start_time,
                    "success": False,
                    "error": str(e),
                    "tokens_generated": 0
                })
            
            time.sleep(0.5)  # Small delay between requests
        
        # Stop monitoring
        self.monitoring = False
        monitor_thread.join(timeout=5)
        
        return results
    
    def make_request(self, prompt):
        """Make a request to Shimmy API"""
        if not self.model_name:
            # Get available models
            models_response = requests.get(f"{self.shimmy_url}/v1/models", timeout=10)
            models = models_response.json().get("data", [])
            if not models:
                raise Exception("No models available")
            self.model_name = models[0]["id"]
        
        payload = {
            "model": self.model_name,
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 100,
            "temperature": 0.7
        }
        
        response = requests.post(
            f"{self.shimmy_url}/v1/chat/completions",
            json=payload,
            timeout=30
        )
        response.raise_for_status()
        return response.json()
    
    def run_full_benchmark(self, output_file="benchmark_results.json"):
        """Run complete benchmark suite"""
        print("Starting Shimmy Performance Benchmark...")
        print(f"Target: {self.shimmy_url}")
        print(f"Model: {self.model_name or 'auto-detect'}")
        
        try:
            # Test if Shimmy is running
            health_response = requests.get(f"{self.shimmy_url}/health", timeout=5)
            print(f"✅ Shimmy is running (status: {health_response.status_code})")
        except Exception as e:
            print(f"❌ Cannot connect to Shimmy: {e}")
            return
        
        # Run performance tests
        performance_results = self.test_inference_performance()
        
        # Calculate statistics
        successful_requests = [r for r in performance_results if r["success"]]
        if successful_requests:
            response_times = [r["response_time"] for r in successful_requests]
            token_counts = [r["tokens_generated"] for r in successful_requests]
            
            stats = {
                "total_requests": len(performance_results),
                "successful_requests": len(successful_requests),
                "success_rate": len(successful_requests) / len(performance_results),
                "avg_response_time": sum(response_times) / len(response_times),
                "min_response_time": min(response_times),
                "max_response_time": max(response_times),
                "avg_tokens_per_second": sum(token_counts) / sum(response_times) if sum(response_times) > 0 else 0,
                "total_tokens_generated": sum(token_counts)
            }
        else:
            stats = {"error": "No successful requests"}
        
        self.results.update({
            "performance_results": performance_results,
            "statistics": stats
        })
        
        # Save results
        with open(output_file, 'w') as f:
            json.dump(self.results, f, indent=2)
        
        print(f"\n📊 Benchmark Results:")
        print(f"Success Rate: {stats.get('success_rate', 0)*100:.1f}%")
        print(f"Avg Response Time: {stats.get('avg_response_time', 0):.2f}s")
        print(f"Tokens/sec: {stats.get('avg_tokens_per_second', 0):.1f}")
        print(f"Results saved to: {output_file}")
        
        return self.results


def main():
    parser = argparse.ArgumentParser(description="Benchmark Shimmy performance")
    parser.add_argument("--url", default="http://localhost:11434", help="Shimmy server URL")
    parser.add_argument("--model", help="Model name to test")
    parser.add_argument("--requests", type=int, default=10, help="Number of test requests")
    parser.add_argument("--output", default="benchmark_results.json", help="Output file")
    
    args = parser.parse_args()
    
    benchmark = ShimmyBenchmark(shimmy_url=args.url, model_name=args.model)
    benchmark.run_full_benchmark(output_file=args.output)


if __name__ == "__main__":
    main()