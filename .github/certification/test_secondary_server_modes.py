#!/usr/bin/env python3
import os,json,urllib.request
from pathlib import Path
base=os.environ["SHIMMY_CERT_BASE"]
def call(path,body):
 r=urllib.request.Request(base+path,json.dumps(body).encode(),{"Content-Type":"application/json"})
 with urllib.request.urlopen(r,timeout=180) as response:return response.status,response.headers.get("Content-Type",""),response.read().decode()
models=json.load(urllib.request.urlopen(base+"/v1/models"))["data"];model=next(x["id"] for x in models if x["id"].lower().startswith("qwen3"))
messages=[{"role":"user","content":"My name is Ada.\n/nothink"},{"role":"assistant","content":"Hello Ada!"},{"role":"user","content":"What is my name?\n/nothink"}]
system="You are a helpful assistant.\n/nothink"
results=[]
for stream in (False,True):
 body={"model":model,"system":system,"messages":messages,"temperature":0,"max_tokens":8,"stream":stream}
 status,ct,raw=call("/api/generate",body)
 if stream:
  assert "text/event-stream" in ct
  pieces=[l[6:] for l in raw.splitlines() if l.startswith("data: ")];assert pieces[-1]=="[DONE]";text="".join(pieces[:-1]);usage=None
 else:
  x=json.loads(raw);text=x["response"];usage=x["usage"];assert x["finish_reason"]=="stop";assert usage=={"prompt_tokens":52,"completion_tokens":6,"total_tokens":58},usage
 assert text=="My name is Ada.",text;results.append({"mode":"native-sse" if stream else "native-json","status":status,"text":text,"usage":usage})
body={"model":model,"system":system,"messages":messages,"temperature":0,"max_tokens":8,"stream":False}
status,ct,raw=call("/v1/messages",body);x=json.loads(raw);assert x["content"][0]["text"]=="My name is Ada.",x;assert x["stop_reason"]=="end_turn";assert x["usage"]=={"input_tokens":52,"output_tokens":6},x
results.append({"mode":"anthropic-json","status":status,"text":x["content"][0]["text"],"usage":x["usage"]})
status,ct,raw=call("/v1/completions",{"model":model,"prompt":"Reply exactly with the number 4 and no other text. /nothink","temperature":0,"max_tokens":8})
x=json.loads(raw);assert x["choices"][0]["text"].strip()=="4",x;assert x["usage"]["prompt_tokens"]>0;assert x["choices"][0]["finish_reason"]=="stop",x
results.append({"mode":"completion-json","status":status,"text":x["choices"][0]["text"],"usage":x["usage"]})
(Path(os.environ["SHIMMY_CERT_OUTPUT"])/"secondary-http.json").write_text(json.dumps({"results":results},indent=2)+"\n")
print(json.dumps(results,indent=2))
