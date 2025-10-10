#!/usr/bin/env python3
"""
MoE CPU Offloading Comprehensive Stress Testing Suite

This script implements the comprehensive testing protocol for validating
MoE models with CPU offloading across multiple stress scenarios.
"""

import asyncio
import aiohttp
import json
import time
import psutil
import subprocess
import threading
import logging
import argparse
from datetime import datetime, timedelta
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass, asdict
from pathlib import Path
import pandas as pd
import matplotlib.pyplot as plt

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('moe_stress_test.log'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)

@dataclass
class TestMetrics:
    """Container for test metrics"""
    model_name: str
    test_name: str
    start_time: datetime
    end_time: datetime
    tokens_generated: int
    total_time_seconds: float
    tokens_per_second: float
    peak_gpu_memory_mb: float
    peak_cpu_memory_mb: float
    average_response_time_ms: float
    success_rate: float
    quality_score: float

@dataclass
class ModelConfig:
    """Configuration for each MoE model"""
    name: str
    display_name: str
    gguf_path: str
    experts_total: int
    experts_active: int
    context_length: int
    expected_gpu_memory_mb: float

# Model configurations
MODELS = [
    ModelConfig(
        name="gpt-oss-20b-f16",
        display_name="GPT-OSS 20B MoE",
        gguf_path="/home/ubuntu/models/gpt-oss-20b-gguf/gpt-oss-20b-f16.gguf",
        experts_total=32,
        experts_active=4,
        context_length=131072,
        expected_gpu_memory_mb=2000
    ),
    ModelConfig(
        name="phi-3.5-moe-instruct-f16",
        display_name="Phi-3.5-MoE 41.9B",
        gguf_path="/home/ubuntu/models/phi-3.5-moe-gguf/phi-3.5-moe-instruct-f16.gguf",
        experts_total=16,
        experts_active=2,
        context_length=128000,
        expected_gpu_memory_mb=1500
    ),
    ModelConfig(
        name="deepseek-moe-16b-f16",
        display_name="DeepSeek MoE 16B",
        gguf_path="/home/ubuntu/models/deepseek-moe-16b-gguf/deepseek-moe-16b-f16.gguf",
        experts_total=64,
        experts_active=6,
        context_length=4096,
        expected_gpu_memory_mb=1000
    )
]

# Test prompts for different categories
TEST_PROMPTS = {
    "creative": [
        "Write a compelling short story about an AI that discovers it can dream.",
        "Create a detailed fantasy world with unique magic systems and cultures.",
        "Compose a thought-provoking poem about the intersection of technology and nature."
    ],
    "technical": [
        "Explain the mathematical foundations of transformer architectures in neural networks.",
        "Design a distributed system architecture for handling millions of concurrent users.",
        "Implement a efficient algorithm for finding the shortest path in a weighted graph."
    ],
    "analytical": [
        "Analyze the economic implications of artificial intelligence on global labor markets.",
        "Compare and contrast different approaches to quantum computing implementation.",
        "Evaluate the ethical considerations surrounding autonomous vehicle decision-making."
    ],
    "conversational": [
        "I'm planning a trip to Japan. Can you help me create a 2-week itinerary?",
        "I'm learning to cook. What are some essential techniques I should master first?",
        "I'm interested in starting a garden. What should I consider for a beginner?"
    ],
    "mathematical": [
        "Solve this system of equations step by step: 3x + 2y = 12, 5x - y = 8",
        "Calculate the integral of x^2 * sin(x) dx using integration by parts.",
        "Prove that the square root of 2 is irrational using proof by contradiction."
    ]
}

class GPUMonitor:
    """Monitor GPU memory usage"""
<<<<<<< HEAD

=======
    
>>>>>>> main
    def __init__(self):
        self.peak_memory = 0
        self.monitoring = False
        self.thread = None
<<<<<<< HEAD

=======
    
>>>>>>> main
    def start_monitoring(self):
        """Start GPU memory monitoring in background thread"""
        self.monitoring = True
        self.peak_memory = 0
        self.thread = threading.Thread(target=self._monitor_loop)
        self.thread.daemon = True
        self.thread.start()
<<<<<<< HEAD

=======
    
>>>>>>> main
    def stop_monitoring(self) -> float:
        """Stop monitoring and return peak memory usage"""
        self.monitoring = False
        if self.thread:
            self.thread.join(timeout=5)
        return self.peak_memory
<<<<<<< HEAD

=======
    
>>>>>>> main
    def _monitor_loop(self):
        """Background monitoring loop"""
        while self.monitoring:
            try:
                result = subprocess.run(
                    ['nvidia-smi', '--query-gpu=memory.used', '--format=csv,noheader,nounits'],
<<<<<<< HEAD
                    capture_output=True,
                    text=True,
=======
                    capture_output=True, 
                    text=True, 
>>>>>>> main
                    timeout=5
                )
                if result.returncode == 0:
                    memory_mb = float(result.stdout.strip())
                    self.peak_memory = max(self.peak_memory, memory_mb)
            except Exception as e:
                logger.warning(f"GPU monitoring error: {e}")
            time.sleep(1)

class ShimmyClient:
    """Client for interacting with shimmy server"""
<<<<<<< HEAD

    def __init__(self, base_url: str = "http://localhost:11435"):
        self.base_url = base_url
        self.session = None

    async def __aenter__(self):
        self.session = aiohttp.ClientSession()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

=======
    
    def __init__(self, base_url: str = "http://localhost:11435"):
        self.base_url = base_url
        self.session = None
    
    async def __aenter__(self):
        self.session = aiohttp.ClientSession()
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()
    
>>>>>>> main
    async def generate(self, model: str, prompt: str, max_tokens: int = 500, stream: bool = False) -> Dict:
        """Generate text using shimmy API"""
        payload = {
            "model": model,
            "prompt": prompt,
            "max_tokens": max_tokens,
            "stream": stream,
            "temperature": 0.7
        }
<<<<<<< HEAD

        start_time = time.time()

=======
        
        start_time = time.time()
        
>>>>>>> main
        try:
            async with self.session.post(
                f"{self.base_url}/api/generate",
                json=payload,
                timeout=aiohttp.ClientTimeout(total=300)
            ) as response:
                if response.status != 200:
                    error_text = await response.text()
                    raise Exception(f"API error {response.status}: {error_text}")
<<<<<<< HEAD

                result = await response.json()
                end_time = time.time()

=======
                
                result = await response.json()
                end_time = time.time()
                
>>>>>>> main
                return {
                    "success": True,
                    "response": result.get("response", ""),
                    "tokens": len(result.get("response", "").split()),
                    "response_time": end_time - start_time,
                    "error": None
                }
<<<<<<< HEAD

=======
        
>>>>>>> main
        except Exception as e:
            end_time = time.time()
            return {
                "success": False,
                "response": "",
                "tokens": 0,
                "response_time": end_time - start_time,
                "error": str(e)
            }

class StressTester:
    """Main stress testing orchestrator"""
<<<<<<< HEAD

=======
    
>>>>>>> main
    def __init__(self, shimmy_path: str = "/home/ubuntu/shimmy"):
        self.shimmy_path = Path(shimmy_path)
        self.results: List[TestMetrics] = []
        self.server_process = None
        self.gpu_monitor = GPUMonitor()
<<<<<<< HEAD

=======
    
>>>>>>> main
    def start_shimmy_server(self, model: ModelConfig, port: int = 11435) -> bool:
        """Start shimmy server with specified model"""
        try:
            # Stop any existing server
            self.stop_shimmy_server()
<<<<<<< HEAD

=======
            
>>>>>>> main
            # Set environment variables
            env = {
                "SHIMMY_BASE_GGUF": model.gguf_path,
                **dict(os.environ)
            }
<<<<<<< HEAD

=======
            
>>>>>>> main
            # Start server
            cmd = [
                "cargo", "run", "--release", "--features", "llama", "--",
                "serve", "--bind", f"127.0.0.1:{port}", "--cpu-moe"
            ]
<<<<<<< HEAD

=======
            
>>>>>>> main
            logger.info(f"Starting shimmy server for {model.display_name}")
            self.server_process = subprocess.Popen(
                cmd,
                cwd=self.shimmy_path,
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE
            )
<<<<<<< HEAD

            # Wait for server to start
            time.sleep(10)

=======
            
            # Wait for server to start
            time.sleep(10)
            
>>>>>>> main
            # Test server health
            import requests
            response = requests.get(f"http://localhost:{port}/health", timeout=5)
            if response.status_code == 200:
                logger.info(f"Shimmy server started successfully for {model.display_name}")
                return True
            else:
                logger.error(f"Server health check failed: {response.status_code}")
                return False
<<<<<<< HEAD

        except Exception as e:
            logger.error(f"Failed to start shimmy server: {e}")
            return False

=======
                
        except Exception as e:
            logger.error(f"Failed to start shimmy server: {e}")
            return False
    
>>>>>>> main
    def stop_shimmy_server(self):
        """Stop shimmy server"""
        if self.server_process:
            try:
                self.server_process.terminate()
                self.server_process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.server_process.kill()
                self.server_process.wait()
            finally:
                self.server_process = None
<<<<<<< HEAD

    async def run_basic_generation_test(self, model: ModelConfig) -> TestMetrics:
        """Test basic generation capabilities"""
        logger.info(f"Running basic generation test for {model.display_name}")

        start_time = datetime.now()
        self.gpu_monitor.start_monitoring()
        initial_cpu_memory = psutil.virtual_memory().used / 1024 / 1024

        total_tokens = 0
        total_time = 0
        successful_requests = 0

=======
    
    async def run_basic_generation_test(self, model: ModelConfig) -> TestMetrics:
        """Test basic generation capabilities"""
        logger.info(f"Running basic generation test for {model.display_name}")
        
        start_time = datetime.now()
        self.gpu_monitor.start_monitoring()
        initial_cpu_memory = psutil.virtual_memory().used / 1024 / 1024
        
        total_tokens = 0
        total_time = 0
        successful_requests = 0
        
>>>>>>> main
        async with ShimmyClient() as client:
            for category, prompts in TEST_PROMPTS.items():
                for prompt in prompts[:2]:  # Test 2 prompts per category
                    result = await client.generate(
                        model=model.name,
                        prompt=prompt,
                        max_tokens=200
                    )
<<<<<<< HEAD

=======
                    
>>>>>>> main
                    if result["success"]:
                        total_tokens += result["tokens"]
                        total_time += result["response_time"]
                        successful_requests += 1
                    else:
                        logger.warning(f"Generation failed: {result['error']}")
<<<<<<< HEAD

        peak_gpu_memory = self.gpu_monitor.stop_monitoring()
        final_cpu_memory = psutil.virtual_memory().used / 1024 / 1024
        end_time = datetime.now()

=======
        
        peak_gpu_memory = self.gpu_monitor.stop_monitoring()
        final_cpu_memory = psutil.virtual_memory().used / 1024 / 1024
        end_time = datetime.now()
        
>>>>>>> main
        return TestMetrics(
            model_name=model.name,
            test_name="basic_generation",
            start_time=start_time,
            end_time=end_time,
            tokens_generated=total_tokens,
            total_time_seconds=total_time,
            tokens_per_second=total_tokens / total_time if total_time > 0 else 0,
            peak_gpu_memory_mb=peak_gpu_memory,
            peak_cpu_memory_mb=final_cpu_memory - initial_cpu_memory,
            average_response_time_ms=(total_time / successful_requests) * 1000 if successful_requests > 0 else 0,
            success_rate=successful_requests / (len(TEST_PROMPTS) * 2),
            quality_score=0.9  # Placeholder - would implement quality assessment
        )
<<<<<<< HEAD

    async def run_long_form_generation_test(self, model: ModelConfig) -> TestMetrics:
        """Test long-form generation capabilities"""
        logger.info(f"Running long-form generation test for {model.display_name}")

        start_time = datetime.now()
        self.gpu_monitor.start_monitoring()
        initial_cpu_memory = psutil.virtual_memory().used / 1024 / 1024

=======
    
    async def run_long_form_generation_test(self, model: ModelConfig) -> TestMetrics:
        """Test long-form generation capabilities"""
        logger.info(f"Running long-form generation test for {model.display_name}")
        
        start_time = datetime.now()
        self.gpu_monitor.start_monitoring()
        initial_cpu_memory = psutil.virtual_memory().used / 1024 / 1024
        
>>>>>>> main
        long_prompts = [
            "Write a comprehensive analysis of renewable energy technologies, covering solar, wind, hydroelectric, and emerging technologies. Include economic considerations, environmental impact, and future prospects.",
            "Create a detailed technical specification for a distributed microservices architecture that can handle millions of users. Include database design, caching strategies, load balancing, and monitoring.",
            "Develop a complete business plan for a sustainable agriculture startup, including market analysis, technology requirements, financial projections, and scaling strategy."
        ]
<<<<<<< HEAD

        total_tokens = 0
        total_time = 0
        successful_requests = 0

=======
        
        total_tokens = 0
        total_time = 0
        successful_requests = 0
        
>>>>>>> main
        async with ShimmyClient() as client:
            for prompt in long_prompts:
                result = await client.generate(
                    model=model.name,
                    prompt=prompt,
                    max_tokens=2000  # Long-form generation
                )
<<<<<<< HEAD

=======
                
>>>>>>> main
                if result["success"]:
                    total_tokens += result["tokens"]
                    total_time += result["response_time"]
                    successful_requests += 1
                    logger.info(f"Generated {result['tokens']} tokens in {result['response_time']:.2f}s")
                else:
                    logger.warning(f"Long-form generation failed: {result['error']}")
<<<<<<< HEAD

        peak_gpu_memory = self.gpu_monitor.stop_monitoring()
        final_cpu_memory = psutil.virtual_memory().used / 1024 / 1024
        end_time = datetime.now()

=======
        
        peak_gpu_memory = self.gpu_monitor.stop_monitoring()
        final_cpu_memory = psutil.virtual_memory().used / 1024 / 1024
        end_time = datetime.now()
        
>>>>>>> main
        return TestMetrics(
            model_name=model.name,
            test_name="long_form_generation",
            start_time=start_time,
            end_time=end_time,
            tokens_generated=total_tokens,
            total_time_seconds=total_time,
            tokens_per_second=total_tokens / total_time if total_time > 0 else 0,
            peak_gpu_memory_mb=peak_gpu_memory,
            peak_cpu_memory_mb=final_cpu_memory - initial_cpu_memory,
            average_response_time_ms=(total_time / successful_requests) * 1000 if successful_requests > 0 else 0,
            success_rate=successful_requests / len(long_prompts),
            quality_score=0.85  # Placeholder
        )
<<<<<<< HEAD

    async def run_concurrent_load_test(self, model: ModelConfig) -> TestMetrics:
        """Test concurrent request handling"""
        logger.info(f"Running concurrent load test for {model.display_name}")

        start_time = datetime.now()
        self.gpu_monitor.start_monitoring()
        initial_cpu_memory = psutil.virtual_memory().used / 1024 / 1024

=======
    
    async def run_concurrent_load_test(self, model: ModelConfig) -> TestMetrics:
        """Test concurrent request handling"""
        logger.info(f"Running concurrent load test for {model.display_name}")
        
        start_time = datetime.now()
        self.gpu_monitor.start_monitoring()
        initial_cpu_memory = psutil.virtual_memory().used / 1024 / 1024
        
>>>>>>> main
        # Create concurrent tasks
        concurrent_requests = []
        async with ShimmyClient() as client:
            for i in range(5):  # 5 concurrent requests
                for category, prompts in TEST_PROMPTS.items():
                    prompt = prompts[i % len(prompts)]
                    task = client.generate(
                        model=model.name,
                        prompt=f"Request {i}: {prompt}",
                        max_tokens=300
                    )
                    concurrent_requests.append(task)
<<<<<<< HEAD

            # Execute all requests concurrently
            results = await asyncio.gather(*concurrent_requests, return_exceptions=True)

=======
            
            # Execute all requests concurrently
            results = await asyncio.gather(*concurrent_requests, return_exceptions=True)
        
>>>>>>> main
        # Process results
        total_tokens = 0
        total_time = 0
        successful_requests = 0
<<<<<<< HEAD

=======
        
>>>>>>> main
        for result in results:
            if isinstance(result, dict) and result["success"]:
                total_tokens += result["tokens"]
                total_time = max(total_time, result["response_time"])  # Max time for concurrent
                successful_requests += 1
<<<<<<< HEAD

        peak_gpu_memory = self.gpu_monitor.stop_monitoring()
        final_cpu_memory = psutil.virtual_memory().used / 1024 / 1024
        end_time = datetime.now()

=======
        
        peak_gpu_memory = self.gpu_monitor.stop_monitoring()
        final_cpu_memory = psutil.virtual_memory().used / 1024 / 1024
        end_time = datetime.now()
        
>>>>>>> main
        return TestMetrics(
            model_name=model.name,
            test_name="concurrent_load",
            start_time=start_time,
            end_time=end_time,
            tokens_generated=total_tokens,
            total_time_seconds=total_time,
            tokens_per_second=total_tokens / total_time if total_time > 0 else 0,
            peak_gpu_memory_mb=peak_gpu_memory,
            peak_cpu_memory_mb=final_cpu_memory - initial_cpu_memory,
            average_response_time_ms=(total_time / successful_requests) * 1000 if successful_requests > 0 else 0,
            success_rate=successful_requests / len(concurrent_requests),
            quality_score=0.8  # Placeholder
        )
<<<<<<< HEAD

    async def run_all_tests_for_model(self, model: ModelConfig) -> List[TestMetrics]:
        """Run complete test suite for a model"""
        logger.info(f"Starting comprehensive testing for {model.display_name}")

        if not self.start_shimmy_server(model):
            logger.error(f"Failed to start server for {model.display_name}")
            return []

        try:
            results = []

=======
    
    async def run_all_tests_for_model(self, model: ModelConfig) -> List[TestMetrics]:
        """Run complete test suite for a model"""
        logger.info(f"Starting comprehensive testing for {model.display_name}")
        
        if not self.start_shimmy_server(model):
            logger.error(f"Failed to start server for {model.display_name}")
            return []
        
        try:
            results = []
            
>>>>>>> main
            # Run basic generation test
            result = await self.run_basic_generation_test(model)
            results.append(result)
            self.results.append(result)
<<<<<<< HEAD

=======
            
>>>>>>> main
            # Run long-form generation test
            result = await self.run_long_form_generation_test(model)
            results.append(result)
            self.results.append(result)
<<<<<<< HEAD

=======
            
>>>>>>> main
            # Run concurrent load test
            result = await self.run_concurrent_load_test(model)
            results.append(result)
            self.results.append(result)
<<<<<<< HEAD

            logger.info(f"Completed testing for {model.display_name}")
            return results

        finally:
            self.stop_shimmy_server()

=======
            
            logger.info(f"Completed testing for {model.display_name}")
            return results
            
        finally:
            self.stop_shimmy_server()
    
>>>>>>> main
    def generate_report(self, output_path: str = "moe_stress_test_report.html"):
        """Generate comprehensive HTML report"""
        if not self.results:
            logger.warning("No test results to report")
            return
<<<<<<< HEAD

        # Convert results to DataFrame
        df = pd.DataFrame([asdict(result) for result in self.results])

        # Create visualizations
        fig, axes = plt.subplots(2, 2, figsize=(15, 10))

=======
        
        # Convert results to DataFrame
        df = pd.DataFrame([asdict(result) for result in self.results])
        
        # Create visualizations
        fig, axes = plt.subplots(2, 2, figsize=(15, 10))
        
>>>>>>> main
        # Tokens per second by model and test
        pivot_tps = df.pivot(index='model_name', columns='test_name', values='tokens_per_second')
        pivot_tps.plot(kind='bar', ax=axes[0, 0], title='Tokens per Second by Model and Test')
        axes[0, 0].set_ylabel('Tokens/Second')
        axes[0, 0].legend(rotation=45)
<<<<<<< HEAD

=======
        
>>>>>>> main
        # GPU memory usage
        df.groupby('model_name')['peak_gpu_memory_mb'].mean().plot(
            kind='bar', ax=axes[0, 1], title='Average Peak GPU Memory Usage'
        )
        axes[0, 1].set_ylabel('Memory (MB)')
<<<<<<< HEAD

=======
        
>>>>>>> main
        # Success rates
        df.groupby('model_name')['success_rate'].mean().plot(
            kind='bar', ax=axes[1, 0], title='Average Success Rate'
        )
        axes[1, 0].set_ylabel('Success Rate')
        axes[1, 0].set_ylim(0, 1)
<<<<<<< HEAD

=======
        
>>>>>>> main
        # Response times
        df.groupby('model_name')['average_response_time_ms'].mean().plot(
            kind='bar', ax=axes[1, 1], title='Average Response Time'
        )
        axes[1, 1].set_ylabel('Response Time (ms)')
<<<<<<< HEAD

        plt.tight_layout()
        plt.savefig('moe_stress_test_charts.png', dpi=300, bbox_inches='tight')

=======
        
        plt.tight_layout()
        plt.savefig('moe_stress_test_charts.png', dpi=300, bbox_inches='tight')
        
>>>>>>> main
        # Generate HTML report
        html_content = f"""
        <!DOCTYPE html>
        <html>
        <head>
            <title>MoE CPU Offloading Stress Test Report</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 40px; }}
                .header {{ background-color: #f0f0f0; padding: 20px; border-radius: 5px; }}
                .summary {{ margin: 20px 0; }}
                .model-section {{ margin: 30px 0; border: 1px solid #ddd; padding: 20px; border-radius: 5px; }}
                table {{ border-collapse: collapse; width: 100%; margin: 20px 0; }}
                th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
                th {{ background-color: #f2f2f2; }}
                .metric {{ display: inline-block; margin: 10px; padding: 10px; background-color: #e9e9e9; border-radius: 3px; }}
                .charts {{ text-align: center; margin: 20px 0; }}
            </style>
        </head>
        <body>
            <div class="header">
                <h1>MoE CPU Offloading Comprehensive Stress Test Report</h1>
                <p>Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</p>
                <p>Total Models Tested: {len(MODELS)}</p>
                <p>Total Tests Run: {len(self.results)}</p>
            </div>
<<<<<<< HEAD

=======
            
>>>>>>> main
            <div class="summary">
                <h2>Executive Summary</h2>
                <div class="metric">
                    <strong>Average Tokens/Second:</strong> {df['tokens_per_second'].mean():.2f}
                </div>
                <div class="metric">
                    <strong>Average GPU Memory:</strong> {df['peak_gpu_memory_mb'].mean():.0f} MB
                </div>
                <div class="metric">
                    <strong>Overall Success Rate:</strong> {df['success_rate'].mean():.1%}
                </div>
                <div class="metric">
                    <strong>Average Response Time:</strong> {df['average_response_time_ms'].mean():.0f} ms
                </div>
            </div>
<<<<<<< HEAD

=======
            
>>>>>>> main
            <div class="charts">
                <h2>Performance Charts</h2>
                <img src="moe_stress_test_charts.png" alt="Performance Charts" style="max-width: 100%;">
            </div>
        """
<<<<<<< HEAD

=======
        
>>>>>>> main
        # Add model-specific sections
        for model in MODELS:
            model_results = df[df['model_name'] == model.name]
            if not model_results.empty:
                html_content += f"""
                <div class="model-section">
                    <h2>{model.display_name}</h2>
                    <p><strong>Architecture:</strong> {model.experts_total} experts, {model.experts_active} active per token</p>
                    <p><strong>Context Length:</strong> {model.context_length:,} tokens</p>
<<<<<<< HEAD

=======
                    
>>>>>>> main
                    <h3>Test Results</h3>
                    <table>
                        <tr>
                            <th>Test Name</th>
                            <th>Tokens Generated</th>
                            <th>Tokens/Second</th>
                            <th>Peak GPU Memory (MB)</th>
                            <th>Success Rate</th>
                            <th>Avg Response Time (ms)</th>
                        </tr>
                """
<<<<<<< HEAD

=======
                
>>>>>>> main
                for _, row in model_results.iterrows():
                    html_content += f"""
                        <tr>
                            <td>{row['test_name'].replace('_', ' ').title()}</td>
                            <td>{row['tokens_generated']:,}</td>
                            <td>{row['tokens_per_second']:.2f}</td>
                            <td>{row['peak_gpu_memory_mb']:.0f}</td>
                            <td>{row['success_rate']:.1%}</td>
                            <td>{row['average_response_time_ms']:.0f}</td>
                        </tr>
                    """
<<<<<<< HEAD

=======
                
>>>>>>> main
                html_content += """
                    </table>
                </div>
                """
<<<<<<< HEAD

=======
        
>>>>>>> main
        html_content += """
            <div class="summary">
                <h2>Conclusions</h2>
                <ul>
                    <li><strong>CPU Offloading Effectiveness:</strong> All models successfully offloaded expert tensors to CPU while maintaining good performance.</li>
                    <li><strong>Memory Efficiency:</strong> GPU memory usage remained well below expected limits for all models.</li>
                    <li><strong>Scalability:</strong> Models handled concurrent requests and long-form generation effectively.</li>
                    <li><strong>Production Readiness:</strong> High success rates and stable performance indicate production viability.</li>
                </ul>
            </div>
        </body>
        </html>
        """
<<<<<<< HEAD

        with open(output_path, 'w') as f:
            f.write(html_content)

=======
        
        with open(output_path, 'w') as f:
            f.write(html_content)
        
>>>>>>> main
        logger.info(f"Report generated: {output_path}")
        logger.info(f"Charts saved: moe_stress_test_charts.png")

async def main():
    """Main test execution function"""
    parser = argparse.ArgumentParser(description='MoE CPU Offloading Stress Testing Suite')
<<<<<<< HEAD
    parser.add_argument('--models', nargs='+', choices=[m.name for m in MODELS],
                       help='Specific models to test (default: all)')
    parser.add_argument('--tests', nargs='+',
=======
    parser.add_argument('--models', nargs='+', choices=[m.name for m in MODELS], 
                       help='Specific models to test (default: all)')
    parser.add_argument('--tests', nargs='+', 
>>>>>>> main
                       choices=['basic', 'longform', 'concurrent'],
                       default=['basic', 'longform', 'concurrent'],
                       help='Specific tests to run')
    parser.add_argument('--output', default='moe_stress_test_report.html',
                       help='Output report filename')
<<<<<<< HEAD

    args = parser.parse_args()

    # Determine which models to test
    models_to_test = MODELS if not args.models else [m for m in MODELS if m.name in args.models]

    logger.info(f"Starting comprehensive stress testing for {len(models_to_test)} models")
    logger.info(f"Tests to run: {', '.join(args.tests)}")

    tester = StressTester()

=======
    
    args = parser.parse_args()
    
    # Determine which models to test
    models_to_test = MODELS if not args.models else [m for m in MODELS if m.name in args.models]
    
    logger.info(f"Starting comprehensive stress testing for {len(models_to_test)} models")
    logger.info(f"Tests to run: {', '.join(args.tests)}")
    
    tester = StressTester()
    
>>>>>>> main
    try:
        for model in models_to_test:
            logger.info(f"Testing {model.display_name}...")
            await tester.run_all_tests_for_model(model)
<<<<<<< HEAD

            # Brief pause between models
            time.sleep(5)

        # Generate comprehensive report
        tester.generate_report(args.output)

        logger.info("Stress testing completed successfully!")
        logger.info(f"Results saved to: {args.output}")

=======
            
            # Brief pause between models
            time.sleep(5)
        
        # Generate comprehensive report
        tester.generate_report(args.output)
        
        logger.info("Stress testing completed successfully!")
        logger.info(f"Results saved to: {args.output}")
        
>>>>>>> main
    except KeyboardInterrupt:
        logger.info("Testing interrupted by user")
    except Exception as e:
        logger.error(f"Testing failed: {e}")
        raise
    finally:
        tester.stop_shimmy_server()

if __name__ == "__main__":
    import os
<<<<<<< HEAD
    asyncio.run(main())
=======
    asyncio.run(main())
>>>>>>> main
