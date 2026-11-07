#!/usr/bin/env python3
"""Actual-token/protocol regression. Requires the pinned Qwen fixture."""
import argparse,json,urllib.request,urllib.error,socket,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument("--base",default="http://127.0.0.1:18095");p.add_argument("--output",required=True);a=p.parse_args()
def request(path,body=None):
 r=urllib.request.Request(a.base+path,None if body is None else json.dumps(body).encode(),{} if body is None else {"Content-Type":"application/json"})
 try:
  with urllib.request.urlopen(r,timeout=180) as response:return response.status,response.headers.get("Content-Type",""),response.read().decode()
 except urllib.error.HTTPError as e:return e.code,e.headers.get("Content-Type",""),e.read().decode()
status,_,raw=request("/v1/models");models=json.loads(raw)["data"];model=next(x["id"] for x in models if x["id"].lower().startswith("qwen3"))
messages=[{"role":"system","content":"You are a helpful assistant.\n/nothink"},{"role":"user","content":"My name is Ada.\n/nothink"},{"role":"assistant","content":"Hello Ada!"},{"role":"user","content":"What is my name?\n/nothink"}]
base={"model":model,"messages":messages,"temperature":0,"max_tokens":8}
results=[]
for mode in ["json","omitted","sse","stop-json","stop-sse","repeat"]:
 body=dict(base)
 if mode!="omitted":body["stream"]="sse" in mode
 if body.get("stream"):body["stream_options"]={"include_usage":True}
 if mode.startswith("stop"):body["stop"]=[" name"]
 status,ct,raw=request("/v1/chat/completions",body);assert status==200,(mode,status,raw)
 if body.get("stream"):
  assert "text/event-stream" in ct
  chunks=[line[6:] for line in raw.splitlines() if line.startswith("data: ")]
  assert chunks[-1]=="[DONE]";chunks=[json.loads(x) for x in chunks[:-1]]
  assert not any("error" in x for x in chunks),chunks
  text="".join(c["delta"].get("content","") or "" for x in chunks for c in x.get("choices",[]))
  finish=next(c["finish_reason"] for x in reversed(chunks) for c in x.get("choices",[]) if c["finish_reason"])
  usage=next(x["usage"] for x in chunks if "usage" in x)
 else:
  assert "application/json" in ct
  x=json.loads(raw);text=x["choices"][0]["message"]["content"];finish=x["choices"][0]["finish_reason"];usage=x["usage"]
 assert text==("My" if mode.startswith("stop") else "My name is Ada."),(mode,text)
 assert finish=="stop",(mode,finish)
 assert usage["prompt_tokens"]==52,(mode,usage)
 assert usage["completion_tokens"]==(2 if mode.startswith("stop") else 6),(mode,usage)
 assert usage["total_tokens"]==usage["prompt_tokens"]+usage["completion_tokens"]
 results.append({"mode":mode,"status":status,"text":text,"finish_reason":finish,"usage":usage});print(results[-1],flush=True)
for case,changes,expected in [("context",{"max_tokens":512},"context_length_exceeded"),("invalid-temperature",{"temperature":-1},"invalid_sampling_parameters")]:
 body=dict(base);body.update(changes);status,_,raw=request("/v1/chat/completions",body);error=json.loads(raw)["error"]
 assert status==400 and error["code"]==expected,(case,status,error);results.append({"mode":case,"status":status,"code":error["code"]})
# Close an SSE response after its role chunk; the next request must recover.
body=dict(base,stream=True,max_tokens=64);r=urllib.request.Request(a.base+"/v1/chat/completions",json.dumps(body).encode(),{"Content-Type":"application/json"})
with urllib.request.urlopen(r,timeout=180) as response:
 while True:
  line=response.readline().decode()
  assert line,"SSE ended before first generated content"
  if line.startswith("data: ") and line[6:].strip()!="[DONE]":
   chunk=json.loads(line[6:])
   if any(c.get("delta",{}).get("content") for c in chunk.get("choices",[])):break
status,_,raw=request("/v1/chat/completions",base);x=json.loads(raw);assert status==200 and x["choices"][0]["message"]["content"]=="My name is Ada.",(status,x)
results.append({"mode":"disconnect-then-repeat","status":status,"text":x["choices"][0]["message"]["content"]})
status,_,raw=request("/v1/models?capabilities=true");caps=next(x["capabilities"] for x in json.loads(raw)["data"] if x["id"]==model)
assert caps["structural_support"] and caps["numerical_certification"]=="NOT_RUN" and caps["platform_certification"]=="NOT_RUN"
results.append({"mode":"discovery","status":status,"capabilities":{k:v for k,v in caps.items() if k!="chat_template"}})
Path(a.output).write_text(json.dumps({"schema":"shimmy.b5.http.v1","results":results},indent=2)+"\n")
