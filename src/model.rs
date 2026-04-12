use crate::ModelCmd;
use crate::server::{ssh_exec, ssh_host_exec};

pub fn run(cmd: ModelCmd) {
    match cmd {
        ModelCmd::List => list(),
        ModelCmd::Pull { url, to } => pull(&url, &to),
        ModelCmd::Remove { path } => remove(&path),
    }
}

fn list_dir(label: &str, dir: &str) {
    println!("=== {} ===", label);
    match ssh_exec(&format!("ls -1S {} 2>/dev/null", dir)) {
        Ok(out) => {
            for line in out.lines() {
                let name = line.trim();
                if name.is_empty() || name.starts_with("put_") { continue; }
                // 파일 크기도 가져오기
                if let Ok(size) = ssh_exec(&format!("du -h {}/{} 2>/dev/null", dir, name)) {
                    let size = size.trim().split('\t').next().unwrap_or("?");
                    println!("  {} ({})", name, size);
                } else {
                    println!("  {}", name);
                }
            }
        }
        Err(_) => println!("  (empty)"),
    }
}

fn list() {
    list_dir("Checkpoints", "/opt/comfyui/models/checkpoints");
    println!();
    list_dir("UNet", "/opt/comfyui/models/unet");
    println!();
    list_dir("LoRA", "/opt/comfyui/models/loras");

    println!("\n=== Ollama Models ===");
    let config = crate::ServerConfig::default();
    match ssh_host_exec(&format!(
        "curl -s {}/api/tags 2>/dev/null",
        config.ollama_url
    )) {
        Ok(out) => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&out) {
                if let Some(models) = v["models"].as_array() {
                    for m in models {
                        let name = m["name"].as_str().unwrap_or("?");
                        let size = m["size"].as_u64().unwrap_or(0);
                        let size_gb = size as f64 / 1_073_741_824.0;
                        println!("  {} ({:.1}GB)", name, size_gb);
                    }
                }
            }
        }
        Err(_) => println!("  Ollama not available"),
    }
}

fn pull(url: &str, to: &str) {
    println!("downloading {} -> models/{}", url, to);

    let remote_path = format!("/opt/comfyui/models/{}", to);
    // 컨테이너 안에서 curl이 없을 수 있으니 호스트에서 실행
    // PID 기반 접근으로 컨테이너 파일시스템에 직접 쓰기
    let host_cmd = format!(
        "PID=$(pgrep -f 'comfyui.*main.py' | head -1) && curl -L -o /proc/$PID/root{} '{}'",
        remote_path, url
    );

    println!("  downloading...");
    match ssh_host_exec(&host_cmd) {
        Ok(_) => {
            // 크기 확인
            let check = format!(
                "PID=$(pgrep -f 'comfyui.*main.py' | head -1) && ls -lh /proc/$PID/root{}",
                remote_path
            );
            match ssh_host_exec(&check) {
                Ok(out) => println!("  done: {}", out.trim()),
                Err(_) => println!("  done"),
            }
        }
        Err(e) => eprintln!("  FAILED: {}", e),
    }
}

fn remove(path: &str) {
    let remote_path = if path.starts_with("models/") {
        format!("/opt/comfyui/{}", path)
    } else {
        format!("/opt/comfyui/models/{}", path)
    };

    println!("removing {}", remote_path);
    let cmd = format!("rm -f {}", remote_path);
    match ssh_exec(&cmd) {
        Ok(_) => println!("  removed"),
        Err(e) => eprintln!("  FAILED: {}", e),
    }
}
