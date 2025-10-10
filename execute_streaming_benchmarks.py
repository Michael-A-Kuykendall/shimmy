#!/usr/bin/env python3
"""
Comprehensive Streaming Benchmark Execution
Based on LOCAL_STREAMING_BENCHMARK_PROTOCOL.md
"""

import requests
import time
import json
import sys
from datetime import datetime
from typing import Dict, List

class StreamingBenchmarkRunner:
    def __init__(self, base_url="http://127.0.0.1:11435", model_name="deepseek-moe-16b-f16"):
        self.base_url = base_url
        self.model_name = model_name
        self.results = []
<<<<<<< HEAD

=======
        
>>>>>>> main
    def calculate_repetition_score(self, text: str) -> float:
        """Calculate repetition score using validated algorithm"""
        if not text or len(text.split()) < 3:
            return 0.0
<<<<<<< HEAD

=======
        
>>>>>>> main
        words = text.split()
        phrases = []
        for i in range(len(words) - 2):
            phrase = ' '.join(words[i:i+3])
            phrases.append(phrase)
<<<<<<< HEAD

        phrase_counts = {}
        for phrase in phrases:
            phrase_counts[phrase] = phrase_counts.get(phrase, 0) + 1

        repeated_phrases = sum(count - 1 for count in phrase_counts.values() if count > 1)
        phrase_repetition = repeated_phrases / len(phrases) if phrases else 0

        return phrase_repetition

    def execute_streaming_test(self, test_name: str, prompt: str, max_tokens: int, timeout: int = 300) -> Dict:
        """Execute a single streaming test with comprehensive metrics"""

        print(f"\nExecuting: {test_name}")
        print(f"   Prompt: \"{prompt[:50]}...\"")
        print(f"   Max tokens: {max_tokens}, Timeout: {timeout}s")

        start_time = time.time()
        first_token_time = None
        tokens = []

=======
        
        phrase_counts = {}
        for phrase in phrases:
            phrase_counts[phrase] = phrase_counts.get(phrase, 0) + 1
        
        repeated_phrases = sum(count - 1 for count in phrase_counts.values() if count > 1)
        phrase_repetition = repeated_phrases / len(phrases) if phrases else 0
        
        return phrase_repetition
    
    def execute_streaming_test(self, test_name: str, prompt: str, max_tokens: int, timeout: int = 300) -> Dict:
        """Execute a single streaming test with comprehensive metrics"""
        
        print(f"\nExecuting: {test_name}")
        print(f"   Prompt: \"{prompt[:50]}...\"")
        print(f"   Max tokens: {max_tokens}, Timeout: {timeout}s")
        
        start_time = time.time()
        first_token_time = None
        tokens = []
        
>>>>>>> main
        try:
            response = requests.post(
                f"{self.base_url}/api/generate",
                json={
                    "model": self.model_name,
                    "prompt": prompt,
                    "max_tokens": max_tokens,
                    "temperature": 0.3,  # Validated to prevent repetition
                    "stream": True
                },
                timeout=timeout,
                stream=True
            )
<<<<<<< HEAD

=======
            
>>>>>>> main
            if response.status_code != 200:
                return {
                    "test_name": test_name,
                    "status": "error",
                    "error": f"HTTP {response.status_code}",
                    "prompt": prompt
                }
<<<<<<< HEAD

            full_response = ""
            token_count = 0

            for line in response.iter_lines(decode_unicode=True):
                if line and line.startswith('data: '):
                    token_data = line[6:]  # Remove 'data: ' prefix

                    if token_data == '[DONE]':
                        break

=======
            
            full_response = ""
            token_count = 0
            
            for line in response.iter_lines(decode_unicode=True):
                if line and line.startswith('data: '):
                    token_data = line[6:]  # Remove 'data: ' prefix
                    
                    if token_data == '[DONE]':
                        break
                    
>>>>>>> main
                    if token_data.strip():
                        # First token timing
                        if first_token_time is None:
                            first_token_time = time.time()
<<<<<<< HEAD

                        full_response += token_data
                        token_count += 1

=======
                        
                        full_response += token_data
                        token_count += 1
                        
>>>>>>> main
                        # Show progress for longer tests
                        if token_count % 20 == 0:
                            elapsed = time.time() - start_time
                            current_rate = token_count / elapsed if elapsed > 0 else 0
                            print(f"   Progress: {token_count} tokens, {current_rate:.2f} tokens/sec")
<<<<<<< HEAD

            end_time = time.time()
            total_time = end_time - start_time
            first_token_latency = (first_token_time - start_time) if first_token_time else 0

=======
            
            end_time = time.time()
            total_time = end_time - start_time
            first_token_latency = (first_token_time - start_time) if first_token_time else 0
            
>>>>>>> main
            # Calculate metrics
            word_count = len(full_response.split())
            tokens_per_second = word_count / total_time if total_time > 0 else 0
            repetition_score = self.calculate_repetition_score(full_response)
<<<<<<< HEAD

=======
            
>>>>>>> main
            # Subjective quality assessment (simple heuristics)
            quality_score = 5  # Start with perfect
            if repetition_score > 0.3:
                quality_score -= 2
            if len(full_response.strip()) < 20:
                quality_score -= 2
            if not full_response.strip():
                quality_score = 1
            quality_score = max(1, quality_score)
<<<<<<< HEAD

=======
            
>>>>>>> main
            result = {
                "test_name": test_name,
                "status": "success",
                "prompt": prompt,
                "response": full_response,
                "metrics": {
                    "total_time": total_time,
                    "first_token_latency": first_token_latency,
                    "word_count": word_count,
                    "tokens_per_second": tokens_per_second,
                    "repetition_score": repetition_score,
                    "quality_score": quality_score,
                    "max_tokens_requested": max_tokens,
                    "response_length": len(full_response)
                }
            }
<<<<<<< HEAD

            print(f"   Completed: {word_count} words in {total_time:.1f}s ({tokens_per_second:.2f} tokens/sec)")
            print(f"   Quality: {quality_score}/5, Repetition: {repetition_score:.3f}")

            return result

=======
            
            print(f"   Completed: {word_count} words in {total_time:.1f}s ({tokens_per_second:.2f} tokens/sec)")
            print(f"   Quality: {quality_score}/5, Repetition: {repetition_score:.3f}")
            
            return result
            
>>>>>>> main
        except Exception as e:
            print(f"   Failed: {e}")
            return {
                "test_name": test_name,
                "status": "timeout/error",
                "error": str(e),
                "prompt": prompt
            }
<<<<<<< HEAD

    def run_benchmark_suite(self):
        """Execute comprehensive benchmark suite"""

=======
    
    def run_benchmark_suite(self):
        """Execute comprehensive benchmark suite"""
        
>>>>>>> main
        print("=" * 60)
        print(f"STREAMING BENCHMARK SUITE - {self.model_name}")
        print(f"Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        print("=" * 60)
<<<<<<< HEAD

=======
        
>>>>>>> main
        # Test suite based on LOCAL_STREAMING_BENCHMARK_PROTOCOL.md
        test_suite = [
            # Basic Functionality Tests
            {
                "name": "Simple Response",
                "prompt": "Hello, how are you?",
                "max_tokens": 50
            },
            {
                "name": "Code Generation",
                "prompt": "Write a Python function to calculate factorial",
                "max_tokens": 150
            },
            {
                "name": "Technical Explanation",
                "prompt": "Explain how binary search works",
                "max_tokens": 200
            },
<<<<<<< HEAD

=======
            
>>>>>>> main
            # Complex Reasoning Tasks
            {
                "name": "Multi-step Problem",
                "prompt": "You have 3-gallon and 5-gallon jugs. Measure exactly 4 gallons step-by-step",
                "max_tokens": 300
            },
            {
                "name": "System Design",
                "prompt": "Design a simple chat application architecture",
                "max_tokens": 400
            },
            {
                "name": "Algorithm Analysis",
                "prompt": "Compare bubble sort and quicksort algorithms",
                "max_tokens": 350
            },
<<<<<<< HEAD

            # Long-form Generation Tests
=======
            
            # Long-form Generation Tests  
>>>>>>> main
            {
                "name": "Creative Writing",
                "prompt": "Write a short story about AI discovering emotions",
                "max_tokens": 800
            },
            {
                "name": "Technical Documentation",
                "prompt": "Document a REST API for a library management system",
                "max_tokens": 1000
            },
            {
                "name": "Research Analysis",
                "prompt": "Analyze the benefits and challenges of renewable energy",
                "max_tokens": 600
            }
        ]
<<<<<<< HEAD

        # Execute all tests
        for i, test in enumerate(test_suite, 1):
            print(f"\nTest {i}/{len(test_suite)}")

            result = self.execute_streaming_test(
                test["name"],
                test["prompt"],
                test["max_tokens"]
            )

            self.results.append(result)

=======
        
        # Execute all tests
        for i, test in enumerate(test_suite, 1):
            print(f"\nTest {i}/{len(test_suite)}")
            
            result = self.execute_streaming_test(
                test["name"],
                test["prompt"], 
                test["max_tokens"]
            )
            
            self.results.append(result)
            
>>>>>>> main
            # Pause between tests
            if i < len(test_suite):
                print("   5-second pause...")
                time.sleep(5)
<<<<<<< HEAD

        # Generate summary
        self.generate_summary()

        # Save detailed results
        self.save_results()

    def generate_summary(self):
        """Generate benchmark summary"""

        print("\n" + "=" * 60)
        print("BENCHMARK SUMMARY")
        print("=" * 60)

        successful_tests = [r for r in self.results if r["status"] == "success"]

        if not successful_tests:
            print("No successful tests completed")
            return

=======
        
        # Generate summary
        self.generate_summary()
        
        # Save detailed results
        self.save_results()
    
    def generate_summary(self):
        """Generate benchmark summary"""
        
        print("\n" + "=" * 60)
        print("BENCHMARK SUMMARY")
        print("=" * 60)
        
        successful_tests = [r for r in self.results if r["status"] == "success"]
        
        if not successful_tests:
            print("No successful tests completed")
            return
        
>>>>>>> main
        # Calculate aggregate metrics
        avg_tokens_per_sec = sum(r["metrics"]["tokens_per_second"] for r in successful_tests) / len(successful_tests)
        avg_quality = sum(r["metrics"]["quality_score"] for r in successful_tests) / len(successful_tests)
        avg_repetition = sum(r["metrics"]["repetition_score"] for r in successful_tests) / len(successful_tests)
        avg_first_token = sum(r["metrics"]["first_token_latency"] for r in successful_tests) / len(successful_tests)
<<<<<<< HEAD

        success_rate = len(successful_tests) / len(self.results) * 100

=======
        
        success_rate = len(successful_tests) / len(self.results) * 100
        
>>>>>>> main
        print(f"Success Rate: {success_rate:.1f}% ({len(successful_tests)}/{len(self.results)})")
        print(f"Average Speed: {avg_tokens_per_sec:.2f} tokens/second")
        print(f"Average First Token: {avg_first_token:.2f} seconds")
        print(f"Average Quality: {avg_quality:.1f}/5")
        print(f"Average Repetition: {avg_repetition:.3f}")
<<<<<<< HEAD

=======
        
>>>>>>> main
        # Individual test results
        print(f"\nIndividual Test Results:")
        for result in self.results:
            if result["status"] == "success":
                metrics = result["metrics"]
                print(f"   {result['test_name']}: {metrics['tokens_per_second']:.2f} tok/s, quality {metrics['quality_score']}/5")
            else:
                print(f"   {result['test_name']}: FAILED {result.get('error', 'Unknown error')}")
<<<<<<< HEAD

=======
        
>>>>>>> main
        # Performance assessment
        print(f"\nPerformance Assessment:")
        if avg_tokens_per_sec >= 2.0:
            print("   Good performance for CPU offloading")
        elif avg_tokens_per_sec >= 1.0:
            print("   Acceptable performance for CPU offloading")
        else:
            print("   Performance below expectations")
<<<<<<< HEAD

=======
        
>>>>>>> main
        if avg_repetition < 0.1:
            print("   No repetition issues (temperature 0.3 working)")
        else:
            print("   Some repetition detected")
<<<<<<< HEAD

=======
        
>>>>>>> main
        if success_rate >= 90:
            print("   High reliability")
        else:
            print("   Some test failures detected")
<<<<<<< HEAD

=======
    
>>>>>>> main
    def save_results(self):
        """Save detailed results to file"""
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        filename = f"streaming_benchmark_{self.model_name}_{timestamp}.json"
<<<<<<< HEAD

=======
        
>>>>>>> main
        benchmark_data = {
            "model": self.model_name,
            "timestamp": datetime.now().isoformat(),
            "test_environment": {
                "temperature": 0.3,
                "streaming": True,
                "cpu_moe_offloading": True
            },
            "results": self.results
        }
<<<<<<< HEAD

        with open(filename, 'w') as f:
            json.dump(benchmark_data, f, indent=2)

=======
        
        with open(filename, 'w') as f:
            json.dump(benchmark_data, f, indent=2)
        
>>>>>>> main
        print(f"\nDetailed results saved to: {filename}")

def main():
    if len(sys.argv) > 1:
        model_name = sys.argv[1]
    else:
        model_name = "deepseek-moe-16b-f16"
<<<<<<< HEAD

=======
    
>>>>>>> main
    runner = StreamingBenchmarkRunner(model_name=model_name)
    runner.run_benchmark_suite()

if __name__ == "__main__":
<<<<<<< HEAD
    main()
=======
    main()
>>>>>>> main
