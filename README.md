# comfyui-workflows

ComfyUI 워크플로우·노드·모델 관리 CLI + 워크플로우 컬렉션.

Rust CLI (`comfyui-workflows`)로 `192.168.2.60` ai-img 서버의 ComfyUI를 관리한다.

## CLI

```bash
cargo run -- <command>
```

### 서버 상태

```bash
comfyui-workflows status          # GPU, 프로세스, 노드 수, 모델, Ollama 상태
```

### 노드 관리

```bash
comfyui-workflows node status     # nodes.yaml vs 서버 비교 (OK/DRIFT/MISSING)
comfyui-workflows node install    # 누락 노드 설치
comfyui-workflows node update     # 전체 노드 업데이트
comfyui-workflows node pin        # 서버 현재 커밋을 nodes.yaml에 기록
comfyui-workflows node add <url>  # 새 노드 추가 (설치 + yaml 등록)
comfyui-workflows node remove <n> # 노드 제거
```

### 워크플로우 관리

```bash
comfyui-workflows workflow list                                 # 서버 워크플로우 목록
comfyui-workflows workflow deploy preview/sdxl_turbo_korean_preview.json  # 서버에 배포
comfyui-workflows workflow sync                                 # 로컬 전체를 서버에 동기화
```

### 모델 관리

```bash
comfyui-workflows model list                    # 체크포인트/UNet/LoRA/Ollama 목록
comfyui-workflows model pull <url> --to <path>  # 모델 다운로드
comfyui-workflows model remove <path>           # 모델 삭제
```

## 구조

```
├── Cargo.toml
├── src/                  # Rust CLI
│   ├── main.rs
│   ├── node.rs           # 노드 관리 (nodes.yaml 기반)
│   ├── workflow.rs       # 워크플로우 배포/동기화
│   ├── model.rs          # 모델 관리
│   └── server.rs         # SSH/서버 접근
├── nodes.yaml            # 커스텀 노드 인벤토리 (28개)
├── preview/              # 프리뷰 워크플로우 (Turbo/Lightning)
├── production/           # 프로덕션 워크플로우
└── video/                # 비디오 생성 워크플로우
```

## 워크플로우

### preview/

| 파일 | 설명 | 모델 |
|------|------|------|
| `sdxl_turbo_realtime_preview.json` | 실시간 프리뷰 (1 step, 512x512) | SDXL Turbo fp16 |
| `sdxl_turbo_korean_preview.json` | 한국어 → 번역 → 프리뷰 | SDXL Turbo + TranslateGemma 12B |

## 빌드

```bash
cargo build
```
