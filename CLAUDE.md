# ComfyUI Workflows

## 이 레포는

Rust CLI + 워크플로우 JSON + 노드 인벤토리를 하나의 레포로 관리한다.
`aip`/`avp` 패턴 (clap + serde + git2)을 따른다.

## 서버 환경

- **Proxmox 호스트**: `ssh root@192.168.2.60`
- **LXC 60105** (ai-img): ComfyUI 실행 컨테이너
- **ComfyUI**: `http://10.0.60.105:8188` (0.18.1)
- **Ollama**: `http://10.0.60.1:11434` (호스트에서 실행, 컨테이너에서 `10.0.60.1`로 접근)
- **GPU**: RTX 3090 x4 (GPU 0,1 서비스 사용 중, GPU 2,3 여유)

## 컨테이너 접근 방법

```bash
# pct exec (권장)
pct exec 60105 -- <command>

# ComfyUI 재시작
pct exec 60105 -- systemctl restart comfyui

# 호스트에서 컨테이너 파일시스템 직접 접근 (PID 기반)
PID=$(pgrep -f 'comfyui.*main.py' | head -1)
nsenter --target $PID --mount --pid -- <command>
# 또는 /proc/$PID/root/opt/comfyui/ 경로로 직접 읽기/쓰기
```

## 모델

### 체크포인트
- `sd_xl_turbo_1.0_fp16.safetensors` (6.5G) — 1-step 실시간 프리뷰
- `Juggernaut-XL_v9_RunDiffusionPhoto_v2.safetensors` (6.7G) — 프로덕션
- `ltx-video-2b-v0.9.5.safetensors` (6.0G) — 비디오 생성
- `ltx-2.3-22b-dev-fp8.safetensors` (28G) — LTX Video 최신

### UNet
- `sdxl_lightning_4step_unet.safetensors` (4.8G) — 4-step 프리뷰

### Ollama
- `translategemma:12b` (7.6G) — 한국어→영어 번역 전용 (Google)

## 노드 관리

`nodes.yaml`에 28개 커스텀 노드가 등록되어 있다.
CLI로 관리:

```bash
comfyui-workflows node status   # 서버와 yaml 비교
comfyui-workflows node pin      # 서버 커밋을 yaml에 기록
```

노드 추가 시 `comfyui-workflows node add <repo>` 사용.
수동으로 nodes.yaml 편집하지 않는다.

## ComfyUI API

```bash
# 프롬프트 큐
curl -X POST http://10.0.60.105:8188/prompt \
  -H "Content-Type: application/json" \
  -d '{"prompt": {...}}'

# 결과 확인
curl http://10.0.60.105:8188/history/<prompt_id>

# 이미지 가져오기
curl http://10.0.60.105:8188/view?filename=<name>&type=output
```

## 워크플로우 배포

```bash
comfyui-workflows workflow deploy preview/sdxl_turbo_korean_preview.json
```

서버 경로: `/opt/comfyui/user/default/workflows/<subfolder>/`
