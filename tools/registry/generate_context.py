#!/usr/bin/env python3
"""
Context Generator for AI Assistants
Generates context files from tool registry for Claude, Copilot, and other AI assistants.
"""

import json
import sys
from pathlib import Path
from datetime import datetime
from typing import Dict, List

class ContextGenerator:
    def __init__(self, registry_path: str):
        self.registry_path = Path(registry_path)
        
    def load_registry(self) -> Dict:
        """Load tool registry."""
        with open(self.registry_path, 'r', encoding='utf-8') as f:
            return json.load(f)
    
    def generate_claude_context(self, registry: Dict) -> str:
        """Generate context for Claude conversations."""
        context = f"""# Micha's Personal Tool Context
*Generated from tool registry at {datetime.now().strftime('%Y-%m-%d %H:%M')}*

## Available Tools & Capabilities

You have access to Micha's personal development tools. Here's what's available:

"""
        
        for tool_name, tool in registry["tools"].items():
            context += f"### {tool['name']} ({tool['type']})\n"
            context += f"**Location**: `{tool['path']}`\n"
            context += f"**Description**: {tool['description']}\n"
            
            if tool['capabilities']:
                context += f"**Capabilities**: {', '.join(tool['capabilities'])}\n"
            
            if tool['cli_commands']:
                context += "\n**CLI Commands**:\n"
                for cmd in tool['cli_commands']:
                    context += f"- `{cmd}`\n"
            
            if tool['api_endpoints']:
                context += "\n**API Endpoints**:\n"
                for endpoint in tool['api_endpoints']:
                    context += f"- `{endpoint}`\n"
            
            if tool.get('github'):
                context += f"**Repository**: {tool['github']}\n"
            
            context += f"**Status**: {tool['status']} (verified {tool['last_verified'][:10]})\n\n"
        
        context += """## Usage Guidelines

When working with these tools:
1. **Always use absolute paths** when referencing tool locations
2. **Check tool status** before suggesting usage
3. **Prefer API endpoints** over CLI when available for automation
4. **Reference documentation** paths for detailed usage information

## Integration Notes

- **Shimmy** can serve as local LLM backend for other tools
- **Punch** provides codebase analysis that can inform development decisions  
- **RustChain** handles mission-driven development workflows
- Tools can be combined for powerful development workflows

This context is automatically updated when the tool registry changes.
"""
        
        return context
    
    def generate_copilot_instructions(self, registry: Dict) -> str:
        """Generate instructions for GitHub Copilot."""
        instructions = f"""# GitHub Copilot Instructions for Micha's Development Environment
*Auto-generated from tool registry on {datetime.now().strftime('%Y-%m-%d')}*

## Available Development Tools

You are working in Micha's development environment with access to these tools:

"""
        
        for tool_name, tool in registry["tools"].items():
            instructions += f"### {tool['name']}\n"
            instructions += f"- **Type**: {tool['type']}\n"
            instructions += f"- **Path**: `{tool['path']}`\n"
            instructions += f"- **Purpose**: {tool['description']}\n"
            
            if tool['capabilities']:
                instructions += f"- **Capabilities**: {', '.join(tool['capabilities'])}\n"
            
            instructions += "\n"
        
        instructions += """## Code Generation Guidelines

When generating code or suggesting solutions:

1. **Tool Integration**: Consider how generated code can work with available tools
2. **Path References**: Use absolute paths when referencing tool locations
3. **API Usage**: Prefer tool APIs over manual implementations when available
4. **Architecture Alignment**: Align suggestions with existing tool patterns

## Common Workflows

- **Local AI**: Use Shimmy for local LLM inference in development
- **Code Analysis**: Use Punch for repository analysis and metrics
- **Mission Execution**: Use RustChain for systematic development tasks
- **API Integration**: Leverage tool APIs for automation and scripting

## Best Practices

- Check tool availability before suggesting usage
- Provide complete working examples that integrate tools
- Consider tool combinations for comprehensive solutions
- Maintain consistency with existing tool architectures

Keep replies focused and actionable, leveraging the available tool ecosystem.
"""
        
        return instructions
    
    def generate_vscode_context(self, registry: Dict) -> str:
        """Generate context for VS Code extension."""
        context = {
            "version": "1.0",
            "generated": datetime.now().isoformat(),
            "tools": {}
        }
        
        for tool_name, tool in registry["tools"].items():
            context["tools"][tool_name] = {
                "name": tool["name"],
                "type": tool["type"],
                "description": tool["description"],
                "capabilities": tool["capabilities"],
                "commands": tool["cli_commands"],
                "endpoints": tool["api_endpoints"],
                "documentation": tool.get("documentation", ""),
                "status": tool["status"]
            }
        
        return json.dumps(context, indent=2)
    
    def generate_all_contexts(self) -> None:
        """Generate all context files."""
        registry = self.load_registry()
        
        # Generate Claude context
        claude_context = self.generate_claude_context(registry)
        claude_path = self.registry_path.parent / "claude-context.md"
        with open(claude_path, 'w', encoding='utf-8') as f:
            f.write(claude_context)
        print(f"Generated Claude context: {claude_path}")
        
        # Generate Copilot instructions
        copilot_instructions = self.generate_copilot_instructions(registry)
        copilot_path = self.registry_path.parent / "copilot-instructions.md"
        with open(copilot_path, 'w', encoding='utf-8') as f:
            f.write(copilot_instructions)
        print(f"Generated Copilot instructions: {copilot_path}")
        
        # Generate VS Code context
        vscode_context = self.generate_vscode_context(registry)
        vscode_path = self.registry_path.parent / "vscode-context.json"
        with open(vscode_path, 'w', encoding='utf-8') as f:
            f.write(vscode_context)
        print(f"Generated VS Code context: {vscode_path}")

def main():
    if len(sys.argv) > 1:
        registry_path = sys.argv[1]
    else:
        registry_path = "micha-tools-registry.json"
    
    generator = ContextGenerator(registry_path)
    generator.generate_all_contexts()
    
    print("\nContext files generated successfully!")
    print("You can now copy these to your global context locations:")
    print("- Copy claude-context.md to your Claude conversation files")
    print("- Copy copilot-instructions.md to .github/copilot-instructions.md")
    print("- Use vscode-context.json for VS Code extension integration")

if __name__ == "__main__":
    main()
