use crate::ServerConfig;
use std::process::Command;

/// 로컬(호스트)에서 실행 중인지 판별
fn is_local() -> bool {
    // hostname이 ranode-3960x 이거나 192.168.2.60 에서 실행 중이면 로컬
    Command::new("hostname")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().contains("ranode-3960x"))
        .unwrap_or(false)
}

/// 컨테이너 안에서 명령 실행
pub fn ssh_exec(cmd: &str) -> Result<String, String> {
    let config = ServerConfig::default();

    if is_local() {
        // 로컬: pct exec 직접
        let full_cmd = format!("pct exec {} -- bash -c '{}'", config.container_id, cmd);
        let output = Command::new("bash")
            .args(["-c", &full_cmd])
            .output()
            .map_err(|e| format!("실행 실패: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("명령 실패: {}", stderr.trim()))
        }
    } else {
        // 원격: SSH 경유
        let full_cmd = format!("pct exec {} -- bash -c '{}'", config.container_id, cmd);
        let output = Command::new("ssh")
            .args(["-o", "ConnectTimeout=5", &config.ssh_host, &full_cmd])
            .output()
            .map_err(|e| format!("SSH 실행 실패: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("명령 실패: {}", stderr.trim()))
        }
    }
}

/// 호스트에서 직접 명령 실행
pub fn ssh_host_exec(cmd: &str) -> Result<String, String> {
    if is_local() {
        // 로컬: 그냥 bash 실행
        let output = Command::new("bash")
            .args(["-c", cmd])
            .output()
            .map_err(|e| format!("실행 실패: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("명령 실패: {}", stderr.trim()))
        }
    } else {
        // 원격: SSH
        let config = ServerConfig::default();
        let output = Command::new("ssh")
            .args(["-o", "ConnectTimeout=5", &config.ssh_host, cmd])
            .output()
            .map_err(|e| format!("SSH 실행 실패: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("명령 실패: {}", stderr.trim()))
        }
    }
}

/// 서버 전체 상태
pub fn status() {
    let config = ServerConfig::default();
    let local = is_local();

    println!("=== ComfyUI Server Status ===");
    println!("  Mode: {}", if local { "local" } else { "remote" });
    println!("  URL: {}", config.comfyui_url);
    println!("  LXC: {}", config.container_id);
    println!();

    // ComfyUI 프로세스 확인
    match ssh_exec("ps aux | grep comfyui | grep main.py | grep -v grep | head -1") {
        Ok(out) if !out.trim().is_empty() => println!("  ComfyUI: running"),
        _ => println!("  ComfyUI: NOT running"),
    }

    // GPU 상태
    match ssh_host_exec("nvidia-smi --query-gpu=index,name,memory.used,memory.free --format=csv,noheader") {
        Ok(out) => {
            println!("  GPU:");
            for line in out.lines() {
                println!("    {}", line.trim());
            }
        }
        Err(_) => println!("  GPU: unavailable"),
    }

    // 노드 수
    match ssh_exec("ls /opt/comfyui/custom_nodes/ | wc -l") {
        Ok(out) => println!("  Nodes: {}", out.trim()),
        Err(_) => println!("  Nodes: unknown"),
    }

    // 모델 수
    match ssh_exec("ls /opt/comfyui/models/checkpoints/ 2>/dev/null | wc -l") {
        Ok(out) => println!("  Checkpoints: {}", out.trim()),
        Err(_) => println!("  Checkpoints: unknown"),
    }

    // Ollama 상태
    match ssh_host_exec("curl -s --connect-timeout 3 http://localhost:11434/api/tags") {
        Ok(out) if out.contains("models") => println!("  Ollama: running"),
        _ => println!("  Ollama: NOT running"),
    }
}
