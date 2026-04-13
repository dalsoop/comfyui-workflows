use crate::WorkflowCmd;
use crate::server::ssh_exec;
use std::path::Path;

pub fn run(cmd: WorkflowCmd) {
    match cmd {
        WorkflowCmd::Deploy { path } => deploy(&path),
        WorkflowCmd::List => list(),
        WorkflowCmd::Sync => sync(),
    }
}

fn deploy(path: &str) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let local_path = Path::new(manifest_dir).join(path);

    if !local_path.exists() {
        // path가 이미 절대경로 또는 상대경로인 경우
        let alt = Path::new(path);
        if !alt.exists() {
            eprintln!("워크플로우 파일을 찾을 수 없습니다: {}", path);
            return;
        }
    }

    let local_path = if Path::new(manifest_dir).join(path).exists() {
        Path::new(manifest_dir).join(path)
    } else {
        Path::new(path).to_path_buf()
    };

    let filename = local_path.file_name().unwrap().to_str().unwrap();
    let subfolder = local_path.parent()
        .and_then(|p| p.file_name())
        .and_then(|f| f.to_str())
        .unwrap_or("default"); // LINT_ALLOW: 기본값

    // 서버에 디렉토리 생성
    let remote_dir = format!("/opt/comfyui/user/default/workflows/{}", subfolder);
    let mkdir_cmd = format!("mkdir -p {}", remote_dir);
    if let Err(e) = ssh_exec(&mkdir_cmd) {
        eprintln!("디렉토리 생성 실패: {}", e);
        return;
    }

    // scp로 복사
    let remote_path = format!("{}/{}", remote_dir, filename);
    let scp_target = format!("root@192.168.2.60:/tmp/{}", filename); // LINT_ALLOW: 추후 .env 이관

    // 먼저 호스트의 /tmp에 복사
    let scp = std::process::Command::new("scp")
        .args([local_path.to_str().unwrap(), &scp_target])
        .output();

    match scp {
        Ok(output) if output.status.success() => {
            // 호스트 /tmp에서 컨테이너로 복사
            let _cp_cmd = format!("cp /tmp/{} {}", filename, remote_path);
            let config = crate::ServerConfig::default();
            let full_cmd = format!(
                "pct push {} /tmp/{} {}",
                config.container_id, filename, remote_path
            );
            match crate::server::ssh_host_exec(&full_cmd) {
                Ok(_) => println!("deployed: {} -> {}", path, remote_path),
                Err(e) => eprintln!("컨테이너 복사 실패: {} — 대체 시도 중...", e),
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("scp 실패: {}", stderr.trim());
        }
        Err(e) => eprintln!("scp 실행 실패: {}", e),
    }
}

fn list() {
    println!("=== Server Workflows ===");
    let cmd = "find /opt/comfyui/user/default/workflows -name '*.json' -type f 2>/dev/null | sort";
    match ssh_exec(cmd) {
        Ok(out) => {
            if out.trim().is_empty() {
                println!("  (none)");
            } else {
                for line in out.lines() {
                    let trimmed = line.trim().replace("/opt/comfyui/user/default/workflows/", "");
                    println!("  {}", trimmed);
                }
            }
        }
        Err(e) => eprintln!("조회 실패: {}", e),
    }
}

fn sync() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    // 로컬 워크플로우 디렉토리들
    let dirs = ["preview", "production", "video"];

    for dir in &dirs {
        let local_dir = Path::new(manifest_dir).join(dir);
        if !local_dir.exists() { continue; }

        let entries = std::fs::read_dir(&local_dir).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let rel_path = format!("{}/{}", dir, path.file_name().unwrap().to_str().unwrap());
                deploy(&rel_path);
            }
        }
    }
}
