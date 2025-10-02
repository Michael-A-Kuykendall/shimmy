#!/usr/bin/env python3
"""
FastAPI integration for Shimmy
"""
from fastapi import FastAPI, HTTPException
import httpx
import os

app = FastAPI(title="Shimmy FastAPI Integration", version="1.0.0")

SHIMMY_BASE_URL = os.getenv("SHIMMY_BASE_URL", "http://localhost:11435")

@app.get("/")
async def root():
    return {"message": "Shimmy FastAPI Integration"}

@app.post("/v1/chat/completions")
async def chat_completions(request: dict):
    async with httpx.AsyncClient() as client:
        response = await client.post(
            f"{SHIMMY_BASE_URL}/v1/chat/completions",
            json=request
        )
        return response.json()

@app.get("/v1/models")  
async def list_models():
    async with httpx.AsyncClient() as client:
        response = await client.get(f"{SHIMMY_BASE_URL}/v1/models")
        return response.json()

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)