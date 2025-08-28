import gradio as gr
import requests
import json
from datetime import datetime

def get_latest_release():
    """Fetch latest release from GitHub API"""
    try:
        # Update this to your actual GitHub repository
        response = requests.get('https://api.github.com/repos/Michael-A-Kuykendall/shimmy/releases/latest')
        if response.status_code == 404:
            # If no releases yet, return fake data for demo
            return {
                'tag_name': 'v0.1.0',
                'published_at': '2025-01-27T00:00:00Z',
                'assets': []
            }
        return response.json()
    except:
        return None

def format_file_size(bytes_size):
    """Convert bytes to MB"""
    return f"{bytes_size / 1024 / 1024:.1f} MB"

def create_shimmy_page():
    release = get_latest_release()
    
    if not release:
        return """
        <div style="text-align: center; padding: 40px;">
            <h1>Shimmy: Universal LLM Shim</h1>
            <p>Unable to fetch latest release. Please visit <a href="https://github.com/Michael-A-Kuykendall/shimmy/releases">GitHub Releases</a></p>
        </div>
        """
    
    # Build download page
    page_html = f"""
    <div style="max-width: 1200px; margin: 0 auto; padding: 20px; font-family: Arial, sans-serif;">
        <div style="text-align: center; margin-bottom: 40px;">
            <h1 style="font-size: 3.5rem; background: linear-gradient(45deg, #dc2626, #3b82f6); -webkit-background-clip: text; -webkit-text-fill-color: transparent; margin-bottom: 20px;">
                🔄 Shimmy
            </h1>
            <h2 style="font-size: 2rem; color: #374151; margin-bottom: 20px;">
                Universal LLM Shim
            </h2>
            <p style="font-size: 1.4rem; background: linear-gradient(45deg, #dc2626, #3b82f6); -webkit-background-clip: text; -webkit-text-fill-color: transparent; font-weight: bold; margin-bottom: 20px;">
                Stop converting your models. Start using them.
            </p>
            <p style="font-size: 1.2rem; color: #666; margin-bottom: 10px;">
                Latest Version: <strong style="color: #dc2626;">{release['tag_name']}</strong>
            </p>
            <p style="color: #888;">
                Released: {datetime.fromisoformat(release['published_at'].replace('Z', '+00:00')).strftime('%B %d, %Y') if release['published_at'] else 'Coming Soon'}
            </p>
            <p style="font-size: 1.1rem; color: #555; max-width: 600px; margin: 20px auto;">
                Run ANY model format through Ollama-compatible API. PEFT, GGUF, Candle - all in one universal shim.
            </p>
        </div>
        
        <div style="background: #fee2e2; border: 2px solid #fca5a5; border-radius: 12px; padding: 30px; margin-bottom: 40px;">
            <h2 style="text-align: center; color: #dc2626; margin-bottom: 20px;">🎯 The Problem</h2>
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px;">
                <div style="text-align: center;">
                    <div style="font-size: 2rem; margin-bottom: 10px;">❌</div>
                    <h4 style="color: #dc2626;">Lossy GGUF Conversion</h4>
                    <p style="color: #7f1d1d;">Ollama forces conversion that loses model precision</p>
                </div>
                <div style="text-align: center;">
                    <div style="font-size: 2rem; margin-bottom: 10px;">❌</div>
                    <h4 style="color: #dc2626;">Format Fragmentation</h4>
                    <p style="color: #7f1d1d;">Each format needs different tooling and APIs</p>
                </div>
                <div style="text-align: center;">
                    <div style="font-size: 2rem; margin-bottom: 10px;">❌</div>
                    <h4 style="color: #dc2626;">PEFT Abandonment</h4>
                    <p style="color: #7f1d1d;">Your fine-tuned models sit unused</p>
                </div>
            </div>
        </div>
        
        <div style="background: #dbeafe; border: 2px solid #93c5fd; border-radius: 12px; padding: 30px; margin-bottom: 40px;">
            <h2 style="text-align: center; color: #1d4ed8; margin-bottom: 20px;">✨ The Shimmy Solution</h2>
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px;">
                <div style="text-align: center;">
                    <div style="font-size: 2rem; margin-bottom: 10px;">🦀</div>
                    <h4 style="color: #1d4ed8;">Pure Rust Performance</h4>
                    <p style="color: #1e3a8a;">Zero-cost abstractions, memory safety</p>
                </div>
                <div style="text-align: center;">
                    <div style="font-size: 2rem; margin-bottom: 10px;">🔄</div>
                    <h4 style="color: #1d4ed8;">Universal Backend</h4>
                    <p style="color: #1e3a8a;">PEFT, GGUF, Candle through one API</p>
                </div>
                <div style="text-align: center;">
                    <div style="font-size: 2rem; margin-bottom: 10px;">⚡</div>
                    <h4 style="color: #1d4ed8;">Ollama Compatible</h4>
                    <p style="color: #1e3a8a;">Drop-in replacement for existing workflows</p>
                </div>
            </div>
        </div>
        
        <div style="background: #1f2937; color: white; border-radius: 12px; padding: 30px; margin-bottom: 40px;">
            <h2 style="text-align: center; margin-bottom: 30px;">🏗️ Architecture</h2>
            <div style="background: #374151; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
                <pre style="color: #f9fafb; margin: 0; text-align: center; font-size: 0.9rem;">
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Ollama API    │◄──►│      SHIMMY      │◄──►│  Your Models   │
│   (Standard)    │    │  (Universal!)    │    │   (Any Format)  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                             │
                    ┌─────────┼─────────┐
                    │         │         │
               ┌────▼───┐ ┌───▼────┐ ┌──▼─────┐
               │  GGUF  │ │  PEFT  │ │ Candle │
               │(llama) │ │ (HF)   │ │(future)│
               └────────┘ └────────┘ └────────┘
                </pre>
            </div>
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px;">
                <div style="background: #374151; padding: 15px; border-radius: 8px; text-align: center;">
                    <h4 style="color: #10b981; margin-bottom: 10px;">✅ HuggingFace/PEFT</h4>
                    <p style="color: #d1d5db; font-size: 0.9rem;">Use your fine-tuned models directly</p>
                </div>
                <div style="background: #374151; padding: 15px; border-radius: 8px; text-align: center;">
                    <h4 style="color: #10b981; margin-bottom: 10px;">✅ llama.cpp/GGUF</h4>
                    <p style="color: #d1d5db; font-size: 0.9rem;">Fast quantized inference</p>
                </div>
                <div style="background: #374151; padding: 15px; border-radius: 8px; text-align: center;">
                    <h4 style="color: #f59e0b; margin-bottom: 10px;">🔄 Candle</h4>
                    <p style="color: #d1d5db; font-size: 0.9rem;">Pure Rust (coming soon)</p>
                </div>
            </div>
        </div>
    """
    
    # Add download section if releases are available
    if release and release.get('assets'):
        page_html += f"""
        <div style="margin-bottom: 40px;">
            <h2 style="text-align: center; color: #1f2937; margin-bottom: 30px;">📥 Download Shimmy</h2>
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px;">
        """
        
        for asset in release['assets']:
            platform = "Windows" if "windows" in asset['name'].lower() or "win" in asset['name'].lower() else \
                      "macOS" if "darwin" in asset['name'].lower() or "macos" in asset['name'].lower() else \
                      "Linux"
            
            icon = "🖥️" if platform == "Windows" else "🍎" if platform == "macOS" else "🐧"
            
            page_html += f"""
                <div style="border: 2px solid #e5e7eb; border-radius: 12px; padding: 20px; background: white; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                    <div style="display: flex; align-items: center; margin-bottom: 15px;">
                        <span style="font-size: 2rem; margin-right: 10px;">{icon}</span>
                        <div>
                            <h3 style="margin: 0; color: #1f2937;">{platform}</h3>
                            <p style="margin: 5px 0 0 0; color: #6b7280; font-size: 0.9rem;">{format_file_size(asset['size'])}</p>
                        </div>
                    </div>
                    <a href="{asset['browser_download_url']}" 
                       style="display: block; background: linear-gradient(45deg, #dc2626, #3b82f6); color: white; text-decoration: none; padding: 12px 20px; border-radius: 8px; text-align: center; font-weight: bold;">
                        📥 Download for {platform}
                    </a>
                </div>
            """
        
        page_html += "</div></div>"
    else:
        # Coming soon section
        page_html += """
        <div style="background: #f0f9ff; border: 2px solid #0ea5e9; border-radius: 12px; padding: 30px; margin-bottom: 40px; text-align: center;">
            <h2 style="color: #0c4a6e; margin-bottom: 20px;">🚧 Coming Soon</h2>
            <p style="color: #0c4a6e; margin-bottom: 20px;">Shimmy is in active development. Pre-compiled binaries will be available soon!</p>
            <a href="https://github.com/Michael-A-Kuykendall/shimmy" 
               style="background: #0ea5e9; color: white; padding: 12px 24px; border-radius: 8px; text-decoration: none; font-weight: bold;">
                🔗 Visit GitHub Repository
            </a>
        </div>
        """
    
    page_html += """
        <div style="background: #f9fafb; border-radius: 12px; padding: 30px; margin-bottom: 40px;">
            <h2 style="text-align: center; color: #1f2937; margin-bottom: 30px;">🚀 Quick Start</h2>
            <div style="background: #1f2937; color: white; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
                <h3 style="color: #10b981; margin-bottom: 15px;">📦 Clone & Build</h3>
                <code style="display: block; background: #111827; padding: 15px; border-radius: 6px; font-family: monospace; white-space: pre-wrap;">git clone https://github.com/Michael-A-Kuykendall/shimmy.git
cd shimmy
cargo build --release --features llama</code>
            </div>
            
            <div style="background: #1f2937; color: white; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
                <h3 style="color: #f59e0b; margin-bottom: 15px;">🧪 Test Your Models</h3>
                <code style="display: block; background: #111827; padding: 15px; border-radius: 6px; font-family: monospace; white-space: pre-wrap;"># List pre-configured models
./target/release/shimmy list

# Test PEFT model loading
./target/release/shimmy probe phi3-personal-champion

# Generate text  
./target/release/shimmy generate phi3-personal-champion --prompt "Hello" --max-tokens 50</code>
            </div>
            
            <div style="background: #1f2937; color: white; padding: 20px; border-radius: 8px;">
                <h3 style="color: #3b82f6; margin-bottom: 15px;">🌐 Start Server</h3>
                <code style="display: block; background: #111827; padding: 15px; border-radius: 6px; font-family: monospace; white-space: pre-wrap;"># Start Ollama-compatible server
./target/release/shimmy serve --bind 0.0.0.0:11434

# Use via HTTP API
curl -X POST http://localhost:11434/api/generate \\
  -H "Content-Type: application/json" \\
  -d '{
    "model": "phi3-personal-champion", 
    "prompt": "Explain quantum computing",
    "max_tokens": 150
  }'</code>
            </div>
        </div>
        
        <div style="text-align: center; padding: 20px;">
            <div style="display: flex; justify-content: center; gap: 20px; flex-wrap: wrap;">
                <a href="https://github.com/Michael-A-Kuykendall/shimmy" 
                   style="background: #374151; color: white; padding: 12px 24px; border-radius: 8px; text-decoration: none; display: flex; align-items: center; gap: 8px;">
                    📚 Documentation
                </a>
                <a href="https://github.com/Michael-A-Kuykendall/shimmy/releases" 
                   style="background: #374151; color: white; padding: 12px 24px; border-radius: 8px; text-decoration: none; display: flex; align-items: center; gap: 8px;">
                    🚀 Releases
                </a>
                <a href="https://github.com/Michael-A-Kuykendall/shimmy/issues" 
                   style="background: #dc2626; color: white; padding: 12px 24px; border-radius: 8px; text-decoration: none; display: flex; align-items: center; gap: 8px;">
                    🐛 Report Issues
                </a>
            </div>
        </div>
        
        <div style="text-align: center; margin-top: 40px; padding: 20px; background: linear-gradient(135deg, #fee2e2, #dbeafe); border-radius: 12px; border-left: 4px solid #dc2626;">
            <h3 style="background: linear-gradient(45deg, #dc2626, #3b82f6); -webkit-background-clip: text; -webkit-text-fill-color: transparent; margin-bottom: 15px;">🔄 Why Shimmy?</h3>
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin-top: 20px;">
                <p style="color: #374151; margin: 0;"><strong>No GGUF conversion hell</strong></p>
                <p style="color: #374151; margin: 0;"><strong>Universal compatibility</strong></p>
                <p style="color: #374151; margin: 0;"><strong>Production ready</strong></p>
                <p style="color: #374151; margin: 0;"><strong>Developer friendly</strong></p>
            </div>
            <p style="color: #6b7280; margin-top: 20px; font-style: italic;">
                Your fine-tuned models deserve better than format conversion.
            </p>
        </div>
    </div>
    """
    
    return page_html

# Create Gradio interface
with gr.Blocks(title="Shimmy: Universal LLM Shim", theme=gr.themes.Soft()) as demo:
    gr.HTML(create_shimmy_page)
    
    # Auto-refresh every 10 minutes to check for new releases
    demo.load(fn=lambda: create_shimmy_page(), outputs=gr.HTML(), every=600)

if __name__ == "__main__":
    demo.launch()