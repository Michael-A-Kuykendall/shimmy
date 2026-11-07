#!/usr/bin/env python3
import os,json,urllib.request
from pathlib import Path
url=os.environ["SHIMMY_CERT_BASE"]
model=next(x["id"] for x in json.load(urllib.request.urlopen(url+"/v1/models"))["data"] if x["id"].lower().startswith("qwen3"))
messages=[{"role":"system","content":"You are a helpful assistant.\n/nothink"},{"role":"user","content":"My name is Ada.\n/nothink"},{"role":"assistant","content":"Hello Ada!"},{"role":"user","content":"What is my name?\n/nothink"}]
rows=[]
for kind,maximum,expected,count,finish in [("multi-token-stop",8,"My",3,"stop"),("unmatched-prefix-at-limit",2,"My name",2,"length")]:
 for stream in (False,True):
  body={"model":model,"messages":messages,"temperature":0,"max_tokens":maximum,"stream":stream,"stop":[" name is"]}
  if stream:body["stream_options"]={"include_usage":True}
  req=urllib.request.Request(url+"/v1/chat/completions",json.dumps(body).encode(),{"Content-Type":"application/json"})
  with urllib.request.urlopen(req,timeout=180) as response:
   assert response.status==200;raw=response.read().decode()
  if stream:
   pieces=[x[6:] for x in raw.splitlines() if x.startswith("data: ")];assert pieces[-1]=="[DONE]";chunks=[json.loads(x) for x in pieces[:-1]]
   text="".join(c["delta"].get("content","") or "" for x in chunks for c in x.get("choices",[]));reason=next(c["finish_reason"] for x in chunks for c in x.get("choices",[]) if c["finish_reason"]);usage=next(x["usage"] for x in chunks if "usage" in x)
  else:
   x=json.loads(raw);text=x["choices"][0]["message"]["content"];reason=x["choices"][0]["finish_reason"];usage=x["usage"]
  assert text==expected and reason==finish,(kind,stream,text,reason)
  assert usage=={"prompt_tokens":52,"completion_tokens":count,"total_tokens":52+count},usage
  rows.append({"mode":kind+("-sse" if stream else "-json"),"text":text,"finish_reason":reason,"usage":usage})
(Path(os.environ["SHIMMY_CERT_OUTPUT"])/"stop-boundaries.json").write_text(json.dumps(rows,indent=2)+"\n");print(json.dumps(rows,indent=2))
