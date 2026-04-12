# comfyui-workflows

ComfyUI workflow collection for `192.168.2.60` (ai-img server).

## Structure

```
preview/      — fast preview workflows (SDXL Turbo, 1 step, 512x512)
production/   — full quality generation workflows
video/        — video generation workflows (LTX, etc.)
```

## Workflows

### preview/

| Workflow | Description | Model |
|----------|-------------|-------|
| `sdxl_turbo_realtime_preview.json` | Basic real-time preview | SDXL Turbo fp16 |
| `sdxl_turbo_korean_preview.json` | Korean prompt -> translate -> preview | SDXL Turbo + TranslateGemma 12B (Ollama) |

## Requirements

- ComfyUI 0.18+
- SDXL Turbo fp16 checkpoint
- [comfyui-ollama](https://github.com/stavsap/comfyui-ollama) (for Korean workflows)
- Ollama with `translategemma:12b` model

## Server Info

- ComfyUI: `http://10.0.60.105:8188`
- Ollama: `http://10.0.60.1:11434`
