#!/usr/bin/env python3
"""
Micha's Tool Registry Scanner
Auto-discovers and updates tool registry with capabilities and metadata.
"""

import json
import os
import sys
import glob
import subprocess
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional

class ToolScanner:
    def __init__(self, registry_path: str):
        self.registry_path = Path(registry_path)
        self.scan_dirs = ["C:\\Users\\micha\\repos"]
        self.exclude_dirs = {"node_modules", "target", ".git", "dist", "build", "__pycache__"}
        
    def load_registry(self) -> Dict:
        """Load existing registry or create new one."""
        if self.registry_path.exists():
            with open(self.registry_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        return self._create_empty_registry()
    
    def _create_empty_registry(self) -> Dict:
        """Create empty registry structure."""
        return {
            "version": "1.0",
            "updated": datetime.now().isoformat(),
            "tools": {},
            "integrations": {
                "context_light": {"enabled": False},
                "claude_copilot": {"enabled": True},
                "vscode_extension": {"enabled": False}
            },
            "discovery": {
                "scan_directories": self.scan_dirs,
                "file_patterns": ["Cargo.toml", "package.json", "requirements.txt", "go.mod", "README.md"],
                "exclude_directories": list(self.exclude_dirs)
            },
            "metadata": {
                "created": datetime.now().isoformat(),
                "last_scan": datetime.now().isoformat(),
                "tool_count": 0,
                "auto_discovery": True
            }
        }
    
    def scan_for_tools(self) -> Dict:
        """Scan directories for tools and return discovered metadata."""
        discovered = {}
        
        for scan_dir in self.scan_dirs:
            if not os.path.exists(scan_dir):
                continue
                
            for root, dirs, files in os.walk(scan_dir):
                # Skip excluded directories
                dirs[:] = [d for d in dirs if d not in self.exclude_dirs]
                
                tool_info = self._analyze_directory(root, files)
                if tool_info:
                    tool_name = os.path.basename(root)
                    discovered[tool_name] = tool_info
        
        return discovered
    
    def _analyze_directory(self, path: str, files: List[str]) -> Optional[Dict]:
        """Analyze directory to determine if it's a tool and extract metadata."""
        path_obj = Path(path)
        
        # Check for key project files
        has_cargo = "Cargo.toml" in files
        has_package = "package.json" in files
        has_requirements = "requirements.txt" in files
        has_go_mod = "go.mod" in files
        has_readme = any(f.lower().startswith('readme') for f in files)
        
        if not (has_cargo or has_package or has_requirements or has_go_mod):
            return None
            
        # Extract basic info
        tool_info = {
            "name": path_obj.name.replace('-', ' ').title(),
            "path": str(path_obj),
            "type": self._infer_tool_type(path, files),
            "description": self._extract_description(path, files),
            "capabilities": self._extract_capabilities(path, files),
            "api_endpoints": self._extract_api_endpoints(path, files),
            "cli_commands": self._extract_cli_commands(path, files),
            "documentation": self._find_documentation(path, files),
            "status": "active",
            "last_verified": datetime.now().isoformat()
        }
        
        # Add language-specific metadata
        if has_cargo:
            tool_info.update(self._analyze_rust_project(path))
        elif has_package:
            tool_info.update(self._analyze_node_project(path))
        elif has_requirements:
            tool_info.update(self._analyze_python_project(path))
        elif has_go_mod:
            tool_info.update(self._analyze_go_project(path))
            
        return tool_info
    
    def _infer_tool_type(self, path: str, files: List[str]) -> str:
        """Infer tool type from project structure."""
        if "server" in path.lower() or "api" in path.lower():
            return "server"
        elif "cli" in path.lower() or "cmd" in path.lower():
            return "cli"
        elif "framework" in path.lower() or "lib" in path.lower():
            return "framework"
        elif "Cargo.toml" in files:
            # Check Cargo.toml for binary vs library
            cargo_path = Path(path) / "Cargo.toml"
            if cargo_path.exists():
                with open(cargo_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                    if "[[bin]]" in content or 'name = "' in content:
                        return "cli"
            return "library"
        elif "package.json" in files:
            return "cli"
        else:
            return "library"
    
    def _extract_description(self, path: str, files: List[str]) -> str:
        """Extract description from README or project files."""
        # Try README first
        for file in files:
            if file.lower().startswith('readme'):
                readme_path = Path(path) / file
                try:
                    with open(readme_path, 'r', encoding='utf-8') as f:
                        lines = f.readlines()
                        for line in lines[1:10]:  # Skip title, check next 9 lines
                            line = line.strip()
                            if line and not line.startswith('#') and len(line) > 20:
                                return line
                except:
                    pass
        
        # Try Cargo.toml description
        cargo_path = Path(path) / "Cargo.toml"
        if cargo_path.exists():
            try:
                with open(cargo_path, 'r', encoding='utf-8') as f:
                    for line in f:
                        if line.startswith('description ='):
                            return line.split('=', 1)[1].strip().strip('"\'')
            except:
                pass
                
        return f"Tool located at {path}"
    
    def _extract_capabilities(self, path: str, files: List[str]) -> List[str]:
        """Extract capabilities from source code and documentation."""
        capabilities = []
        
        # Analyze source files for common patterns
        src_patterns = {
            "api": ["server", "http", "rest", "endpoint"],
            "cli": ["command", "arg", "subcommand"],
            "analysis": ["analyze", "scan", "profile", "metrics"],
            "inference": ["model", "generate", "predict", "llm"],
            "discovery": ["discover", "find", "search", "detect"],
            "registry": ["register", "store", "cache", "database"]
        }
        
        # Scan source files for patterns
        for root, dirs, source_files in os.walk(path):
            dirs[:] = [d for d in dirs if d not in self.exclude_dirs]
            for file in source_files:
                if file.endswith(('.rs', '.py', '.js', '.ts', '.go')):
                    file_path = Path(root) / file
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            content = f.read().lower()
                            for capability, keywords in src_patterns.items():
                                if any(keyword in content for keyword in keywords):
                                    if capability not in capabilities:
                                        capabilities.append(capability)
                    except:
                        continue
        
        return capabilities if capabilities else ["general purpose tool"]
    
    def _extract_api_endpoints(self, path: str, files: List[str]) -> List[str]:
        """Extract API endpoints from source code."""
        endpoints = []
        
        # Look for route definitions in common frameworks
        route_patterns = [
            r'@app\.route\(["\']([^"\']+)["\']',  # Flask
            r'app\.(get|post|put|delete)\(["\']([^"\']+)["\']',  # Express
            r'\.route\(["\']([^"\']+)["\']',  # Axum/Warp
            r'#\[get\(["\']([^"\']+)["\']',  # Actix-web
        ]
        
        for root, dirs, source_files in os.walk(path):
            dirs[:] = [d for d in dirs if d not in self.exclude_dirs]
            for file in source_files:
                if file.endswith(('.rs', '.py', '.js', '.ts')):
                    file_path = Path(root) / file
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            content = f.read()
                            # Simple pattern matching for common route definitions
                            if '/api/' in content or '/v1/' in content:
                                lines = content.split('\n')
                                for line in lines:
                                    if any(pattern in line.lower() for pattern in ['get ', 'post ', 'put ', 'delete ']):
                                        if '/api/' in line or '/v1/' in line:
                                            # Extract endpoint from line
                                            if '"' in line:
                                                parts = line.split('"')
                                                for part in parts:
                                                    if part.startswith('/'):
                                                        if part not in endpoints:
                                                            endpoints.append(part)
                    except:
                        continue
        
        return endpoints
    
    def _extract_cli_commands(self, path: str, files: List[str]) -> List[str]:
        """Extract CLI commands from source code and documentation."""
        commands = []
        
        # Look for clap/argparse definitions
        for root, dirs, source_files in os.walk(path):
            dirs[:] = [d for d in dirs if d not in self.exclude_dirs]
            for file in source_files:
                if file.endswith(('.rs', '.py')):
                    file_path = Path(root) / file
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            content = f.read()
                            # Look for subcommand definitions
                            lines = content.split('\n')
                            for line in lines:
                                if 'subcommand' in line.lower() or 'command' in line.lower():
                                    if '"' in line:
                                        parts = line.split('"')
                                        for i, part in enumerate(parts):
                                            if i % 2 == 1:  # Inside quotes
                                                if part.isalpha() and len(part) > 2:
                                                    cmd = f"{Path(path).name} {part}"
                                                    if cmd not in commands:
                                                        commands.append(cmd)
                    except:
                        continue
        
        return commands
    
    def _find_documentation(self, path: str, files: List[str]) -> str:
        """Find main documentation file."""
        for file in files:
            if file.lower().startswith('readme'):
                return str(Path(path) / file)
        return ""
    
    def _analyze_rust_project(self, path: str) -> Dict:
        """Extract Rust-specific metadata."""
        cargo_path = Path(path) / "Cargo.toml"
        metadata = {}
        
        if cargo_path.exists():
            try:
                with open(cargo_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                    if 'repository =' in content:
                        for line in content.split('\n'):
                            if line.strip().startswith('repository ='):
                                repo = line.split('=', 1)[1].strip().strip('"\'')
                                if 'github.com' in repo:
                                    metadata['github'] = repo
            except:
                pass
                
        return metadata
    
    def _analyze_node_project(self, path: str) -> Dict:
        """Extract Node.js-specific metadata."""
        return {}
    
    def _analyze_python_project(self, path: str) -> Dict:
        """Extract Python-specific metadata."""
        return {}
    
    def _analyze_go_project(self, path: str) -> Dict:
        """Extract Go-specific metadata."""
        return {}
    
    def update_registry(self, discovered_tools: Dict) -> None:
        """Update registry with discovered tools."""
        registry = self.load_registry()
        
        # Update existing tools and add new ones
        for tool_name, tool_info in discovered_tools.items():
            if tool_name in registry["tools"]:
                # Update existing tool, preserve manual edits
                existing = registry["tools"][tool_name]
                existing["last_verified"] = tool_info["last_verified"]
                # Update auto-discoverable fields
                existing["capabilities"] = tool_info["capabilities"]
                existing["api_endpoints"] = tool_info["api_endpoints"]
                existing["cli_commands"] = tool_info["cli_commands"]
            else:
                # Add new tool
                registry["tools"][tool_name] = tool_info
        
        # Update metadata
        registry["updated"] = datetime.now().isoformat()
        registry["metadata"]["last_scan"] = datetime.now().isoformat()
        registry["metadata"]["tool_count"] = len(registry["tools"])
        
        # Save updated registry
        with open(self.registry_path, 'w', encoding='utf-8') as f:
            json.dump(registry, f, indent=2, ensure_ascii=False)
        
        print(f"Registry updated with {len(discovered_tools)} tools")
        print(f"Total tools in registry: {len(registry['tools'])}")

def main():
    if len(sys.argv) > 1:
        registry_path = sys.argv[1]
    else:
        registry_path = "micha-tools-registry.json"
    
    scanner = ToolScanner(registry_path)
    
    print("Scanning for tools...")
    discovered = scanner.scan_for_tools()
    
    print(f"Discovered {len(discovered)} tools:")
    for name, info in discovered.items():
        print(f"  - {name}: {info['type']} at {info['path']}")
    
    print("\nUpdating registry...")
    scanner.update_registry(discovered)
    
    print(f"Registry saved to: {scanner.registry_path}")

if __name__ == "__main__":
    main()
