#!/usr/bin/env python3
"""
Simple DCO (Developer Certificate of Origin) sign-off tool
Adds "Signed-off-by" lines to git commits retroactively
"""

import subprocess
import sys
import re
from datetime import datetime

def get_git_config(key):
    """Get git config value"""
    try:
        result = subprocess.run(['git', 'config', key], 
                              capture_output=True, text=True, check=True, encoding='utf-8')
        return result.stdout.strip()
    except subprocess.CalledProcessError:
        return None

def get_author_info():
    """Get author name and email from git config"""
    name = get_git_config('user.name')
    email = get_git_config('user.email')
    
    if not name or not email:
        print("Error: Git user.name and user.email must be configured")
        print("Run: git config user.name 'Your Name'")
        print("Run: git config user.email 'your.email@example.com'")
        sys.exit(1)
    
    return name, email

def has_signoff(commit_message, author_email):
    """Check if commit message already has sign-off"""
    signoff_pattern = rf'Signed-off-by:.*{re.escape(author_email)}'
    return bool(re.search(signoff_pattern, commit_message, re.IGNORECASE))

def add_signoff_to_commits(commit_range="HEAD"):
    """Add DCO sign-off to commits in the specified range"""
    name, email = get_author_info()
    signoff_line = f"Signed-off-by: {name} <{email}>"
    
    print(f"Adding DCO sign-off: {signoff_line}")
    
    # Get list of commits to process
    try:
        result = subprocess.run(['git', 'rev-list', '--reverse', commit_range], 
                              capture_output=True, text=True, check=True, encoding='utf-8')
        commits = result.stdout.strip().split('\n')
        
        if not commits or commits == ['']:
            print("No commits found to process")
            return
            
    except subprocess.CalledProcessError as e:
        print(f"Error getting commit list: {e}")
        return

    print(f"Processing {len(commits)} commits...")
    
    # Process each commit
    modified_count = 0
    for commit in commits:
        try:
            # Get current commit message
            result = subprocess.run(['git', 'show', '--format=%B', '--no-patch', commit], 
                                  capture_output=True, text=True, check=True, encoding='utf-8')
            original_message = result.stdout.strip()
            
            # Check if already signed off
            if has_signoff(original_message, email):
                continue
                
            # Add sign-off line
            new_message = original_message + f"\n\n{signoff_line}"
            
            # Use git filter-branch to rewrite the commit message
            env = {'FILTER_BRANCH_SQUELCH_WARNING': '1'}
            subprocess.run([
                'git', 'filter-branch', '-f', '--msg-filter', 
                f'if [ "$GIT_COMMIT" = "{commit}" ]; then '
                f'echo \'{new_message}\'; '
                f'else cat; fi'
            ], env={**subprocess.os.environ, **env}, check=True, 
               capture_output=True)
            
            modified_count += 1
            print(f"[OK] Added sign-off to commit {commit[:8]}")
            
        except subprocess.CalledProcessError as e:
            print(f"[ERROR] Failed to process commit {commit[:8]}: {e}")
            continue
    
    print(f"\nSUCCESS: Successfully added DCO sign-off to {modified_count} commits")
    
    if modified_count > 0:
        print("\nIMPORTANT: Commit hashes have changed due to rewriting history")
        print("   If you've already pushed to a remote, you'll need to force push:")
        print("   git push --force-with-lease")

def setup_git_hook():
    """Setup git commit-msg hook to automatically add DCO sign-off"""
    name, email = get_author_info()
    signoff_line = f"Signed-off-by: {name} <{email}>"
    
    hook_path = ".git/hooks/commit-msg"
    hook_content = f'''#!/bin/sh
# Automatically add DCO sign-off to commit messages

# Check if message already has sign-off
if ! grep -q "Signed-off-by: .*{email}" "$1"; then
    echo "" >> "$1"
    echo "{signoff_line}" >> "$1"
fi
'''
    
    try:
        with open(hook_path, 'w') as f:
            f.write(hook_content)
        
        # Make hook executable (Unix-style permissions)
        import os
        import stat
        st = os.stat(hook_path)
        os.chmod(hook_path, st.st_mode | stat.S_IEXEC)
        
        print(f"[OK] Git commit-msg hook installed")
        print(f"   Future commits will automatically include: {signoff_line}")
        
    except Exception as e:
        print(f"[ERROR] Failed to setup git hook: {e}")

def main():
    """Main function"""
    import argparse
    
    parser = argparse.ArgumentParser(description='DCO sign-off tool for git commits')
    parser.add_argument('--sign', action='store_true', 
                       help='Add DCO sign-off to existing commits')
    parser.add_argument('--range', default='HEAD', 
                       help='Commit range to process (default: HEAD)')
    parser.add_argument('--setup-hook', action='store_true',
                       help='Setup git commit-msg hook for automatic DCO sign-off')
    parser.add_argument('--check', action='store_true',
                       help='Check DCO compliance of commits')
    
    args = parser.parse_args()
    
    if not any([args.sign, args.setup_hook, args.check]):
        parser.print_help()
        return
    
    # Verify we're in a git repository
    try:
        subprocess.run(['git', 'rev-parse', '--git-dir'], 
                      capture_output=True, check=True, encoding='utf-8')
    except subprocess.CalledProcessError:
        print("Error: Not in a git repository")
        sys.exit(1)
    
    if args.setup_hook:
        setup_git_hook()
    
    if args.sign:
        add_signoff_to_commits(args.range)
    
    if args.check:
        name, email = get_author_info()
        try:
            result = subprocess.run(['git', 'log', '--format=%H %s'], 
                                  capture_output=True, text=True, check=True, encoding='utf-8')
            commits = result.stdout.strip().split('\n')
            
            missing_signoff = 0
            for line in commits:
                if not line:
                    continue
                commit_hash = line.split()[0]
                
                # Get full commit message
                result = subprocess.run(['git', 'show', '--format=%B', '--no-patch', commit_hash], 
                                      capture_output=True, text=True, check=True, encoding='utf-8')
                message = result.stdout.strip()
                
                if not has_signoff(message, email):
                    print(f"[MISSING] DCO: {commit_hash[:8]} - {line[9:]}")
                    missing_signoff += 1
                else:
                    print(f"[OK] DCO: {commit_hash[:8]} - {line[9:]}")
            
            if missing_signoff == 0:
                print(f"\nSUCCESS: All commits have proper DCO sign-off!")
            else:
                print(f"\nWARNING: {missing_signoff} commits missing DCO sign-off")
                print("Run with --sign to fix them")
                
        except subprocess.CalledProcessError as e:
            print(f"Error checking commits: {e}")

if __name__ == '__main__':
    main()