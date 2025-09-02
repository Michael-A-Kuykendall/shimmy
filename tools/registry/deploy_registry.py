#!/usr/bin/env python3
"""
Micha's Tool Registry Deployment Script
Sets up the personal tool registry system and generates initial context files.
"""

import os
import sys
import json
import shutil
import subprocess
from pathlib import Path

def deploy_registry():
    """Deploy the tool registry system to user profile."""
    
    # Create user tools directory
    user_tools_dir = Path.home() / ".micha-tools"
    user_tools_dir.mkdir(exist_ok=True)
    print(f"Created directory: {user_tools_dir}")
    
    # Copy registry files
    current_dir = Path(__file__).parent
    registry_file = current_dir / "micha-tools-registry.json"
    scanner_file = current_dir / "scan_tools.py"
    generator_file = current_dir / "generate_context.py"
    
    # Copy to user directory
    shutil.copy2(registry_file, user_tools_dir / "registry.json")
    shutil.copy2(scanner_file, user_tools_dir / "scan_tools.py")
    shutil.copy2(generator_file, user_tools_dir / "generate_context.py")
    
    print(f"Deployed files to: {user_tools_dir}")
    
    # Run initial scan
    print("\nRunning initial tool scan...")
    os.chdir(user_tools_dir)
    subprocess.run([sys.executable, "scan_tools.py", "registry.json"])
    
    # Generate context files
    print("\nGenerating context files...")
    subprocess.run([sys.executable, "generate_context.py", "registry.json"])
    
    # Create maintenance script
    maintenance_script = f'''@echo off
REM Micha's Tool Registry Maintenance
cd /d "{user_tools_dir}"
python scan_tools.py registry.json
python generate_context.py registry.json
echo Tool registry updated!
pause
'''
    
    batch_file = user_tools_dir / "update_registry.bat"
    with open(batch_file, 'w') as f:
        f.write(maintenance_script)
    
    print(f"\nDeployment complete!")
    print(f"Registry location: {user_tools_dir / 'registry.json'}")
    print(f"Context files generated in: {user_tools_dir}")
    print(f"Maintenance script: {batch_file}")
    
    print("\nNext steps:")
    print("1. Review generated context files in your user directory")
    print("2. Copy claude-context.md to your Claude conversation files")
    print("3. Copy copilot-instructions.md to project .github/ directories")
    print("4. Run update_registry.bat weekly to keep registry current")
    print("5. Set MICHA_TOOLS_REGISTRY environment variable if desired")
    
    # Show discovered tools
    with open(user_tools_dir / "registry.json", 'r') as f:
        registry = json.load(f)
    
    print(f"\nDiscovered tools ({len(registry['tools'])}):")
    for name, tool in registry['tools'].items():
        print(f"  - {tool['name']} ({tool['type']}) at {tool['path']}")

if __name__ == "__main__":
    try:
        deploy_registry()
    except Exception as e:
        print(f"Deployment failed: {e}")
        sys.exit(1)
