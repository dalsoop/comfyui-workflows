# ComfyUI Workflows

## Server Environment

- **ComfyUI**: LXC 60105 (ai-img) on Proxmox 192.168.2.60
- **ComfyUI URL**: `http://10.0.60.105:8188`
- **Ollama**: host 192.168.2.60, accessible from container at `http://10.0.60.1:11434`
- **GPU**: RTX 3090 x4 (GPU 0,1 used by services, GPU 2,3 available)

## Models

### Checkpoints
- `sd_xl_turbo_1.0_fp16.safetensors` — 1-step real-time preview (512x512)
- `Juggernaut-XL_v9_RunDiffusionPhoto_v2.safetensors` — production quality
- `ltx-video-2b-v0.9.5.safetensors` — video generation

### UNet
- `sdxl_lightning_4step_unet.safetensors` — 4-step preview (1024x1024)

### LLM (Ollama)
- `translategemma:12b` — Korean→English translation, Google specialized model

## Custom Nodes
- `comfyui-ollama` — Ollama LLM integration for prompt translation/enhancement

## Workflow Conventions

- **preview/**: fast iteration workflows (Turbo/Lightning, low step count)
- **production/**: full quality workflows (high steps, upscale, controlnet)
- **video/**: video generation workflows

## SSH Access

```bash
# Host
ssh root@192.168.2.60

# Container commands
pct exec 60105 -- <command>

# ComfyUI restart
pct exec 60105 -- systemctl restart comfyui

# Container process access from host
PID=$(pgrep -f 'comfyui.*main.py' | head -1)
nsenter --target $PID --mount --pid -- <command>
```

## ComfyUI API

```bash
# Queue a prompt
curl -X POST http://10.0.60.105:8188/prompt -H "Content-Type: application/json" -d '{"prompt": {...}}'

# Check history
curl http://10.0.60.105:8188/history/<prompt_id>

# View generated image
curl http://10.0.60.105:8188/view?filename=<name>&type=output
```
