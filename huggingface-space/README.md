---
title: Shimmy - Universal LLM Shim
emoji: 🔄
colorFrom: red
colorTo: blue
sdk: gradio
sdk_version: 4.44.0
app_file: app.py
pinned: false
license: apache-2.0
short_description: Stop converting your models. Start using them. Universal LLM serving for PEFT, GGUF, and beyond.
---

# Shimmy: Universal LLM Shim

**Run ANY model format through Ollama-compatible API**

Stop converting your models. Start using them.

Shimmy provides a single Ollama-compatible API for any model format - PEFT/LoRA, GGUF, and future formats.

## 🎯 The Problem

- ❌ Ollama forces lossy GGUF conversion  
- ❌ No universal model serving solution exists
- ❌ Each format needs different tooling
- ❌ Converting models loses precision

## ✨ The Solution

Universal LLM shim with Ollama-compatible API for any model format.

## Features

- ✅ **PEFT/LoRA Support**: Use your fine-tuned models directly
- ✅ **GGUF Support**: Keep your quantized models for performance  
- ✅ **Ollama API**: Drop-in replacement for existing workflows
- ✅ **Zero Dependencies**: Single Rust binary, no Python needed
- ✅ **Multi-Backend**: Automatic backend selection for optimal performance

Visit our GitHub repository for downloads and documentation.