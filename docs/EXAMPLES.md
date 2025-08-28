# Examples

This document provides practical examples of using Shimmy in various scenarios.

## Basic Usage Examples

### Simple Text Generation

```bash
# Start Shimmy server
export SHIMMY_BASE_GGUF=/path/to/model.gguf
shimmy serve --bind 127.0.0.1:11435

# Generate text via CLI
shimmy generate --prompt "Write a haiku about programming" --max-tokens 50

# Generate text via API
curl -X POST http://localhost:11435/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "default",
    "prompt": "Write a haiku about programming",
    "max_tokens": 50,
    "temperature": 0.7
  }'
```

### Streaming Response

```bash
# Stream tokens as they're generated
curl -N -X POST http://localhost:11435/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "default",
    "prompt": "Count from 1 to 10:",
    "max_tokens": 100,
    "stream": true
  }'
```

## Code Generation Examples

### Function Documentation

```bash
# Generate documentation for a function
curl -X POST http://localhost:11435/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "default",
    "prompt": "Document this Rust function:\n\nfn calculate_fibonacci(n: u32) -> u64 {\n    match n {\n        0 => 0,\n        1 => 1,\n        _ => calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2)\n    }\n}",
    "max_tokens": 200
  }'
```

### Code Review

```bash
# Generate code review comments
curl -X POST http://localhost:11435/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "default",
    "prompt": "Review this code for potential issues:\n\n```rust\nfn unsafe_divide(a: i32, b: i32) -> i32 {\n    a / b\n}\n```",
    "max_tokens": 150
  }'
```

## Automation Examples

### Git Commit Message Generation

```bash
#!/bin/bash
# commit_helper.sh

# Get staged changes
diff=$(git diff --cached)

if [ -z "$diff" ]; then
    echo "No staged changes found"
    exit 1
fi

# Generate commit message
commit_msg=$(curl -s -X POST http://localhost:11435/api/generate \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"default\",
    \"prompt\": \"Generate a concise git commit message for these changes:\\n\\n$diff\",
    \"max_tokens\": 50,
    \"temperature\": 0.3
  }" | jq -r '.choices[0].text' | head -1)

echo "Suggested commit message:"
echo "$commit_msg"

read -p "Use this message? (y/n): " confirm
if [ "$confirm" = "y" ]; then
    git commit -m "$commit_msg"
fi
```

### Code Documentation Generator

```python
#!/usr/bin/env python3
import os
import requests
import re

def generate_docs(file_path):
    """Generate documentation for a source file using Shimmy."""
    
    with open(file_path, 'r') as f:
        code = f.read()
    
    prompt = f"""Generate comprehensive documentation for this code file:

{code}

Include:
1. Overview of what the file does
2. Main functions and their purposes
3. Key data structures
4. Usage examples

Documentation:"""

    response = requests.post('http://localhost:11435/api/generate', json={
        'model': 'default',
        'prompt': prompt,
        'max_tokens': 500,
        'temperature': 0.2
    })
    
    return response.json()['choices'][0]['text']

# Usage
if __name__ == '__main__':
    import sys
    if len(sys.argv) != 2:
        print("Usage: python3 doc_generator.py <source_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    docs = generate_docs(file_path)
    
    # Save documentation
    doc_path = file_path.replace('.rs', '_docs.md').replace('.py', '_docs.md')
    with open(doc_path, 'w') as f:
        f.write(f"# Documentation for {file_path}\n\n")
        f.write(docs)
    
    print(f"Documentation generated: {doc_path}")
```

## WebSocket Examples

### Interactive Chat Client

```javascript
// chat_client.js
const WebSocket = require('ws');

class ShimmyChat {
    constructor(url = 'ws://localhost:11435/ws/generate') {
        this.ws = new WebSocket(url);
        this.setupEventHandlers();
    }
    
    setupEventHandlers() {
        this.ws.on('open', () => {
            console.log('Connected to Shimmy');
        });
        
        this.ws.on('message', (data) => {
            const message = JSON.parse(data);
            if (message.done) {
                console.log('\n--- Response complete ---');
            } else {
                process.stdout.write(message.token);
            }
        });
        
        this.ws.on('error', (error) => {
            console.error('WebSocket error:', error);
        });
    }
    
    send(prompt, options = {}) {
        const message = {
            model: 'default',
            prompt: prompt,
            max_tokens: options.maxTokens || 200,
            temperature: options.temperature || 0.7
        };
        
        this.ws.send(JSON.stringify(message));
    }
    
    close() {
        this.ws.close();
    }
}

// Usage
const chat = new ShimmyChat();

// Wait for connection
setTimeout(() => {
    chat.send("Hello, how can you help me today?");
}, 1000);
```

### Real-time Code Assistance

```html
<!DOCTYPE html>
<html>
<head>
    <title>Shimmy Code Assistant</title>
    <style>
        .container { max-width: 800px; margin: 0 auto; padding: 20px; }
        .input-area { width: 100%; height: 200px; margin-bottom: 10px; }
        .output-area { width: 100%; height: 300px; border: 1px solid #ccc; padding: 10px; }
        button { padding: 10px 20px; margin: 5px; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Shimmy Code Assistant</h1>
        
        <textarea class="input-area" id="codeInput" 
                  placeholder="Paste your code here..."></textarea>
        
        <button onclick="explainCode()">Explain Code</button>
        <button onclick="findBugs()">Find Bugs</button>
        <button onclick="optimize()">Optimize</button>
        
        <div class="output-area" id="output"></div>
    </div>

    <script>
        let ws = null;
        
        function connectWebSocket() {
            ws = new WebSocket('ws://localhost:11435/ws/generate');
            
            ws.onmessage = function(event) {
                const data = JSON.parse(event.data);
                if (!data.done) {
                    document.getElementById('output').innerHTML += data.token;
                }
            };
        }
        
        function sendPrompt(prompt) {
            if (!ws || ws.readyState !== WebSocket.OPEN) {
                connectWebSocket();
                setTimeout(() => sendPrompt(prompt), 1000);
                return;
            }
            
            document.getElementById('output').innerHTML = '';
            ws.send(JSON.stringify({
                model: 'default',
                prompt: prompt,
                max_tokens: 300,
                temperature: 0.3
            }));
        }
        
        function explainCode() {
            const code = document.getElementById('codeInput').value;
            const prompt = `Explain what this code does:\n\n${code}`;
            sendPrompt(prompt);
        }
        
        function findBugs() {
            const code = document.getElementById('codeInput').value;
            const prompt = `Find potential bugs in this code:\n\n${code}`;
            sendPrompt(prompt);
        }
        
        function optimize() {
            const code = document.getElementById('codeInput').value;
            const prompt = `Suggest optimizations for this code:\n\n${code}`;
            sendPrompt(prompt);
        }
        
        // Connect on page load
        connectWebSocket();
    </script>
</body>
</html>
```

## Integration Examples

### Vim Plugin

```vim
" shimmy.vim - Vim plugin for Shimmy integration

function! ShimmyGenerate(prompt)
    let l:cmd = "curl -s -X POST http://localhost:11435/api/generate " .
              \ "-H 'Content-Type: application/json' " .
              \ "-d '{\"model\":\"default\",\"prompt\":\"" . a:prompt . "\",\"max_tokens\":200}'"
    
    let l:response = system(l:cmd)
    let l:json = json_decode(l:response)
    
    if has_key(l:json, 'choices') && len(l:json.choices) > 0
        return l:json.choices[0].text
    else
        return "Error: Could not generate response"
    endif
endfunction

function! ShimmyExplainSelection()
    let l:selected = getline("'<", "'>")
    let l:code = join(l:selected, "\n")
    let l:prompt = "Explain this code:\n\n" . l:code
    
    let l:explanation = ShimmyGenerate(l:prompt)
    
    " Insert explanation as comments
    call append(line("'>"), "")
    call append(line("'>") + 1, "\" " . l:explanation)
endfunction

" Commands
command! -range ShimmyExplain call ShimmyExplainSelection()
command! -nargs=1 ShimmyAsk echo ShimmyGenerate(<q-args>)

" Mappings
vnoremap <leader>se :ShimmyExplain<CR>
nnoremap <leader>sa :ShimmyAsk<Space>
```

### GitHub Actions Integration

```yaml
# .github/workflows/ai-review.yml
name: AI Code Review

on:
  pull_request:
    types: [opened, synchronize]

jobs:
  ai-review:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
      with:
        fetch-depth: 0
    
    - name: Setup Shimmy
      run: |
        # Download and setup Shimmy
        wget https://github.com/user/shimmy/releases/latest/download/shimmy-linux
        chmod +x shimmy-linux
        export SHIMMY_BASE_GGUF=/models/code-review-model.gguf
        ./shimmy-linux serve --bind 127.0.0.1:11435 &
        sleep 10
    
    - name: Generate Review
      run: |
        # Get diff
        git diff origin/main..HEAD > diff.txt
        
        # Generate review with Shimmy
        curl -X POST http://localhost:11435/api/generate \
          -H "Content-Type: application/json" \
          -d @- << EOF > review.md
        {
          "model": "default",
          "prompt": "Review this code change and provide constructive feedback:\n\n$(cat diff.txt)",
          "max_tokens": 1000,
          "temperature": 0.3
        }
        EOF
    
    - name: Post Review
      uses: actions/github-script@v6
      with:
        script: |
          const fs = require('fs');
          const review = fs.readFileSync('review.md', 'utf8');
          
          github.rest.issues.createComment({
            issue_number: context.issue.number,
            owner: context.repo.owner,
            repo: context.repo.repo,
            body: `## AI Code Review\n\n${review}`
          });
```

## Performance Examples

### Benchmarking Script

```bash
#!/bin/bash
# benchmark_shimmy.sh

SHIMMY_URL="http://localhost:11435/api/generate"
PROMPTS=(
    "Hello world"
    "Write a function to calculate fibonacci numbers"
    "Explain quantum computing in simple terms"
    "Generate a JSON schema for a user profile"
)

echo "Benchmarking Shimmy performance..."
echo "URL: $SHIMMY_URL"
echo "=========================="

total_time=0
total_requests=0

for prompt in "${PROMPTS[@]}"; do
    echo "Testing prompt: $prompt"
    
    start_time=$(date +%s.%N)
    
    response=$(curl -s -X POST "$SHIMMY_URL" \
        -H "Content-Type: application/json" \
        -d "{
            \"model\": \"default\",
            \"prompt\": \"$prompt\",
            \"max_tokens\": 100,
            \"temperature\": 0.7
        }")
    
    end_time=$(date +%s.%N)
    duration=$(echo "$end_time - $start_time" | bc)
    
    tokens=$(echo "$response" | jq -r '.usage.completion_tokens // 0')
    
    echo "  Duration: ${duration}s"
    echo "  Tokens: $tokens"
    
    if [ "$tokens" -gt 0 ]; then
        tokens_per_sec=$(echo "scale=2; $tokens / $duration" | bc)
        echo "  Tokens/sec: $tokens_per_sec"
    fi
    
    echo "---"
    
    total_time=$(echo "$total_time + $duration" | bc)
    total_requests=$((total_requests + 1))
done

avg_time=$(echo "scale=3; $total_time / $total_requests" | bc)
echo "=========================="
echo "Total requests: $total_requests"
echo "Total time: ${total_time}s"
echo "Average time per request: ${avg_time}s"
```
