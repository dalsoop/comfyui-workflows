mod node;
mod workflow;
mod model;
mod server;

use clap::{Parser, Subcommand};

pub const BIN_NAME: &str = "comfyui-workflows"; // LINT_ALLOW: 시스템 경로/상수

/// ComfyUI 서버 접근 설정
pub struct ServerConfig {
    /// ComfyUI API URL
    pub comfyui_url: String,
    /// SSH 접근 (pct exec)
    pub ssh_host: String,
    /// LXC container ID
    pub container_id: String,
    /// ComfyUI 설치 경로 (컨테이너 내)
    pub comfyui_dir: String,
    /// Ollama URL
    pub ollama_url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            comfyui_url: "http://10.0.60.105:8188".into(), // LINT_ALLOW: 추후 .env 이관
            ssh_host: "root@192.168.2.60".into(), // LINT_ALLOW: 추후 .env 이관
            container_id: "60105".into(),
            comfyui_dir: "/opt/comfyui".into(),
            ollama_url: "http://10.0.60.1:11434".into(), // LINT_ALLOW: 추후 .env 이관
        }
    }
}

#[derive(Parser)]
#[command(name = BIN_NAME)]
#[command(about = "ComfyUI 워크플로우·노드·모델 관리 CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 커스텀 노드 관리
    Node {
        #[command(subcommand)]
        cmd: NodeCmd,
    },
    /// 워크플로우 관리
    Workflow {
        #[command(subcommand)]
        cmd: WorkflowCmd,
    },
    /// 모델 관리
    Model {
        #[command(subcommand)]
        cmd: ModelCmd,
    },
    /// 서버 상태 확인
    Status,
}

// === NODE ===
#[derive(Subcommand)]
pub enum NodeCmd {
    /// nodes.yaml vs 서버 비교
    Status,
    /// 누락된 노드 설치
    Install {
        /// 특정 노드만 설치
        name: Option<String>,
    },
    /// 노드 업데이트
    Update {
        /// 특정 노드만 업데이트 (생략 시 전체)
        name: Option<String>,
    },
    /// 서버의 현재 커밋을 nodes.yaml에 기록
    Pin {
        /// 특정 노드만 (생략 시 전체)
        name: Option<String>,
    },
    /// 새 노드 추가 (설치 + yaml 등록)
    Add {
        /// Git repo URL
        repo: String,
        /// 노드 설명
        #[arg(long)]
        desc: Option<String>,
    },
    /// 노드 제거
    Remove {
        /// 노드 이름
        name: String,
    },
}

// === WORKFLOW ===
#[derive(Subcommand)]
pub enum WorkflowCmd {
    /// 워크플로우를 서버에 배포
    Deploy {
        /// 워크플로우 파일 경로 (예: preview/sdxl_turbo_korean_preview.json)
        path: String,
    },
    /// 서버의 워크플로우 목록
    List,
    /// 로컬 워크플로우와 서버 동기화
    Sync,
}

// === MODEL ===
#[derive(Subcommand)]
pub enum ModelCmd {
    /// 서버 모델 목록
    List,
    /// 모델 다운로드
    Pull {
        /// 다운로드 URL
        url: String,
        /// 저장 경로 (예: checkpoints/model.safetensors)
        #[arg(long)]
        to: String,
    },
    /// 모델 삭제
    Remove {
        /// 모델 경로
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Node { cmd } => node::run(cmd),
        Commands::Workflow { cmd } => workflow::run(cmd),
        Commands::Model { cmd } => model::run(cmd),
        Commands::Status => server::status(),
    }
}
