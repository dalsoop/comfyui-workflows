use crate::ServerConfig;
use std::process::Command;

/// SSH 명령 실행 (pct exec 경유)
pub fn ssh_exec(cmd: &str) -> Result<String, String> {
    let config = ServerConfig::default();
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

/// SSH 명령 실행 (호스트에서 직접)
pub fn ssh_host_exec(cmd: &str) -> Result<String, String> {
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

/// 서버 전체 상태
pub fn status() {
    let config = ServerConfig::default();

    println!("=== ComfyUI Server Status ===");
    println!("  URL: {}", config.comfyui_url);
    println!("  SSH: {}", config.ssh_host);
    println!("  LXC: {}", config.container_id);
    println!();

    // ComfyUI 프로세스 확인
    match ssh_exec("ps aux | grep 'comfyui.*main.py' | grep -v grep | head -1") {
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
    match ssh_exec("ls /opt/comfyui/models/checkpoints/ 2>/dev/null | grep -v put_ | wc -l") {
        Ok(out) => println!("  Checkpoints: {}", out.trim()),
        Err(_) => println!("  Checkpoints: unknown"),
    }

    // Ollama 상태
    match ssh_host_exec(&format!("curl -s --connect-timeout 3 {}/api/tags | head -1", config.ollama_url)) {
        Ok(out) if out.contains("models") => println!("  Ollama: running"),
        _ => println!("  Ollama: NOT running"),
    }
}
