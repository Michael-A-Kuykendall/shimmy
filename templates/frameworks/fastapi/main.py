"""
FastAPI integration example for Shimmy AI inference
"""
import asyncio
import httpx
import os
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List, Optional

app = FastAPI(
    title="Shimmy AI API",
    description="FastAPI wrapper for Shimmy AI inference engine",
    version="1.0.0"
)

# Configuration
SHIMMY_BASE_URL = os.getenv("SHIMMY_URL", "http://localhost:11435")
SHIMMY_API_KEY = os.getenv("SHIMMY_API_KEY", "sk-local")

class ChatMessage(BaseModel):
    role: str
    content: str

class ChatRequest(BaseModel):
    model: str
    messages: List[ChatMessage]
    max_tokens: Optional[int] = 100
    temperature: Optional[float] = 0.7
    stream: Optional[bool] = False

class ChatResponse(BaseModel):
    id: str
    object: str
    model: str
    choices: List[dict]
    usage: dict

@app.get("/")
async def root():
    """Health check endpoint"""
    return {"status": "healthy", "service": "shimmy-fastapi"}

@app.get("/models")
async def list_models():
    """List available models from Shimmy"""
    async with httpx.AsyncClient() as client:
        try:
            response = await client.get(
                f"{SHIMMY_BASE_URL}/v1/models",
                headers={"Authorization": f"Bearer {SHIMMY_API_KEY}"}
            )
            response.raise_for_status()
            return response.json()
        except httpx.RequestError as e:
            raise HTTPException(status_code=503, detail=f"Shimmy service unavailable: {e}")
        except httpx.HTTPStatusError as e:
            raise HTTPException(status_code=e.response.status_code, detail=str(e))

@app.post("/chat/completions", response_model=ChatResponse)
async def chat_completions(request: ChatRequest):
    """Chat completions endpoint - OpenAI compatible"""
    async with httpx.AsyncClient() as client:
        try:
            shimmy_request = {
                "model": request.model,
                "messages": [msg.dict() for msg in request.messages],
                "max_tokens": request.max_tokens,
                "temperature": request.temperature,
                "stream": request.stream
            }

            response = await client.post(
                f"{SHIMMY_BASE_URL}/v1/chat/completions",
                json=shimmy_request,
                headers={
                    "Authorization": f"Bearer {SHIMMY_API_KEY}",
                    "Content-Type": "application/json"
                },
                timeout=300.0  # 5 minutes for long responses
            )
            response.raise_for_status()
            return response.json()

        except httpx.RequestError as e:
            raise HTTPException(status_code=503, detail=f"Shimmy service unavailable: {e}")
        except httpx.HTTPStatusError as e:
            raise HTTPException(status_code=e.response.status_code, detail=str(e))

@app.post("/simple-chat")
async def simple_chat(prompt: str, model: str = "phi3-mini"):
    """Simplified chat endpoint for quick testing"""
    request = ChatRequest(
        model=model,
        messages=[ChatMessage(role="user", content=prompt)],
        max_tokens=150
    )
    return await chat_completions(request)

# Startup event
@app.on_event("startup")
async def startup_event():
    """Verify Shimmy connection on startup"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{SHIMMY_BASE_URL}/v1/models",
                headers={"Authorization": f"Bearer {SHIMMY_API_KEY}"},
                timeout=10.0
            )
            response.raise_for_status()
            print(f"✅ Connected to Shimmy at {SHIMMY_BASE_URL}")
    except Exception as e:
        print(f"⚠️  Could not connect to Shimmy: {e}")

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
