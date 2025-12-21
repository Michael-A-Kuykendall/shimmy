# Vision Tool Architecture for Shimmy

**Date**: December 1, 2025  
**Purpose**: Enable AI agents to autonomously capture and analyze screenshots

---

## 1. What actually happens when you paste an image

Conceptually:

1. **You paste an image in the UI.**

2. The client encodes it (usually as PNG/JPEG → base64 or multipart).

3. The request to the model contains a message like:

   ```jsonc
   {
     "role": "user",
     "content": [
       {"type": "text", "text": "What is the error in this screenshot?"},
       {
         "type": "image_url",
         "image_url": {
           "url": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgA..."
         }
       }
     ]
   }
   ```

4. The vision model sees both the text and the image and answers.

Your "why can't we just do that automatically?" answer is:

> You can. You just need a tool that:
>
> * grabs an image (screenshot or file),
> * encodes it like that,
> * and feeds it into your vision model *as if* it was pasted.

Shimmy already owns the orchestration, so you're in a perfect spot to do this.

---

## 2. The missing piece: a `read_image` / `capture_screenshot` tool

For an AI that can call tools, the tool your extension exposes should basically:

* **Input**: file path, or "take a screenshot of X".
* **Output**: *structured* "vision-ready" data, not just OCR text.

Think of the tool returning something like:

```jsonc
{
  "kind": "image_bundle",
  "mime": "image/png",
  "data_base64": "iVBORw0KGgoAAAANSUhEUgA...",
  "origin": {
    "source": "screenshot",
    "windowTitle": "shimmy-console",
    "timestamp": 1733070000
  }
}
```

Your orchestrator then turns that into the model-native shape (OpenAI-style `image_url` or whatever your local vision backend wants), and sends a new "vision" message back into the conversation.

---

## 3. Minimal tool schema (Shimmy-style, model-agnostic)

Define a tool the AI can call:

```jsonc
{
  "name": "read_image",
  "description": "Load an image or capture a screenshot so you can inspect UI, errors, or design.",
  "parameters": {
    "type": "object",
    "properties": {
      "path": {
        "type": "string",
        "description": "Absolute or workspace-relative path to an image file."
      },
      "screenshot": {
        "type": "boolean",
        "description": "If true, capture a screenshot instead of reading from disk."
      },
      "region": {
        "type": "object",
        "description": "Optional capture region for screenshot in screen coordinates.",
        "properties": {
          "x": { "type": "integer" },
          "y": { "type": "integer" },
          "width": { "type": "integer" },
          "height": { "type": "integer" }
        },
        "required": ["x", "y", "width", "height"]
      }
    },
    "oneOf": [
      { "required": ["path"] },
      { "required": ["screenshot"] }
    ]
  }
}
```

And you define what it returns (for your own router) as the `image_bundle` object above.

---

## 4. VS Code-side implementation sketch

In a Shimmy/VS Code extension (TypeScript):

```ts
import * as vscode from 'vscode';
import * as fs from 'fs/promises';

type ReadImageArgs = {
  path?: string;
  screenshot?: boolean;
  region?: { x: number; y: number; width: number; height: number };
};

type ImageBundle = {
  kind: 'image_bundle';
  mime: string;
  data_base64: string;
  origin: {
    source: 'file' | 'screenshot';
    path?: string;
    windowTitle?: string;
    timestamp: number;
  };
};

// You'll wire this into whatever tool-calling bridge Shimmy uses
export async function handleReadImage(args: ReadImageArgs): Promise<ImageBundle> {
  if (args.path) {
    const absolutePath = resolveWorkspacePath(args.path); // your helper
    const buf = await fs.readFile(absolutePath);
    const mime = guessMime(absolutePath); // simple extension-based helper
    return {
      kind: 'image_bundle',
      mime,
      data_base64: buf.toString('base64'),
      origin: {
        source: 'file',
        path: absolutePath,
        timestamp: Date.now()
      }
    };
  }

  if (args.screenshot) {
    // Use a screenshot library (e.g. node-robotjs / desktop-capture / your own)
    const { buffer, mime, windowTitle } = await captureScreenshot(args.region);
    return {
      kind: 'image_bundle',
      mime,
      data_base64: buffer.toString('base64'),
      origin: {
        source: 'screenshot',
        windowTitle,
        timestamp: Date.now()
      }
    };
  }

  throw new Error('Either path or screenshot must be provided.');
}
```

On the Shimmy engine side, when the AI calls `read_image`, you:

1. Execute `handleReadImage`.
2. Take the returned `ImageBundle`.
3. Craft a *new* model request with both:

   * The assistant's follow-up question ("analyze the current screenshot for layout issues"), and
   * The image as `image_url` / raw bytes / whatever your vision backend expects.

---

## 5. Getting *OCR + layout + design*, not just text

Once you have that plumbing, you can layer specialized tools on top that *call* the vision model:

### a) "OCR only" tool

Tool: `extract_text_from_image`

* Input: `{ image: ImageBundle, include_layout: boolean }`
* Implementation: call your vision model with a prompt like:

  > Extract all visible text from this image. Preserve line breaks and group by region where possible.
* Return: structured blocks, e.g.:

  ```jsonc
  {
    "blocks": [
      {
        "role": "code_console",
        "text": "error[E0433]: failed to resolve: use of undeclared crate or module `libshimmy`",
        "bbox": [10, 50, 800, 200]
      },
      {
        "role": "status_bar",
        "text": "branch: 001-core-inference-engine",
        "bbox": [0, 700, 1200, 730]
      }
    ]
  }
  ```

Now your dev agent can **reason about the terminal output** as plain text.

### b) "UI / design analysis" tool

Tool: `analyze_ui_screenshot`

* Input: `{ image: ImageBundle, focus?: "accessibility" | "layout" | "visual_theme" | "error_state" }`
* Implementation: prompt a vision model like:

  > You are a UI/UX and frontend dev assistant. Analyze this screenshot of a dev tool interface…
* Return: a list of findings, each with severity, region, and suggestion.

This is where you get the "look at the design" part you mentioned: spacing, color contrast, hierarchy, etc.

### c) "Compare two screenshots" tool

Tool: `diff_screenshots`

* Input: `{ before: ImageBundle, after: ImageBundle }`
* Implementation: load both images into the vision model and ask:

  > Compare these two screenshots. What changed visually, and where?

Perfect for "theme shakedown" and regression checks.

---

## 6. Hooking this into your local AI loops

A typical autonomous flow for Shimmy might be:

1. AI wants to understand a bug → calls `read_image` with the latest terminal screenshot path.
2. Your tool returns `ImageBundle`.
3. AI then calls `extract_text_from_image` to get structured console text.
4. AI uses that text to:

   * locate the repo/branch (`libshimmy`, `001-core-inference-engine`),
   * open the relevant file,
   * propose fixes.
5. Optionally, AI calls `analyze_ui_screenshot` on a UI shot to suggest design/compliance fixes.

From the AI's point of view, this is **identical** to "user pasted a screenshot and then asked a question" — you're just automating the "paste screenshot" part.

---

## 7. Experiment: Quick Test

To test if this approach works with the current Copilot session, we can try encoding a screenshot as base64 and embedding it in a markdown image tag or returning it from a tool in the expected format.

**Test file**: `theme-tester/screenshots/test1.png`
