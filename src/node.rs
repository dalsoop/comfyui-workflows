use crate::NodeCmd;
use crate::server::ssh_exec;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct NodesFile {
    nodes: Vec<NodeEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NodeEntry {
    pub name: String,
    pub repo: String,
    pub commit: String,
    #[serde(default)]
    pub description: String,
}

fn load_nodes_yaml() -> NodesFile {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(manifest_dir).join("nodes.yaml");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("nodes.yaml not found at {}", path.display()));
    serde_yaml::from_str(&content).expect("nodes.yaml 파싱 실패")
}

fn save_nodes_yaml(nodes_file: &NodesFile) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(manifest_dir).join("nodes.yaml");
    let header = "# ComfyUI Custom Nodes — ai-img (60105)\n# Managed by comfyui-workflows CLI\n\n";
    let yaml = serde_yaml::to_string(nodes_file).expect("yaml 직렬화 실패");
    std::fs::write(&path, format!("{}{}", header, yaml)).expect("nodes.yaml 쓰기 실패");
}

/// 서버에서 노드 현재 커밋 조회
fn get_server_commit(name: &str) -> Option<String> {
    let cmd = format!(
        "cd /opt/comfyui/custom_nodes/{} && git rev-parse --short HEAD 2>/dev/null",
        name
    );
    ssh_exec(&cmd).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// 서버에 노드가 존재하는지
fn node_exists_on_server(name: &str) -> bool {
    let cmd = format!("test -d /opt/comfyui/custom_nodes/{} && echo yes", name);
    ssh_exec(&cmd).map(|s| s.trim() == "yes").unwrap_or(false)
}

pub fn run(cmd: NodeCmd) {
    match cmd {
        NodeCmd::Status => status(),
        NodeCmd::Install { name } => install(name),
        NodeCmd::Update { name } => update(name),
        NodeCmd::Pin { name } => pin(name),
        NodeCmd::Add { repo, desc } => add(&repo, desc),
        NodeCmd::Remove { name } => remove(&name),
    }
}

fn status() {
    let nodes_file = load_nodes_yaml();

    println!("{:<40} {:<10} {:<10} {}", "NAME", "EXPECTED", "SERVER", "STATUS");
    println!("{}", "-".repeat(75));

    for node in &nodes_file.nodes {
        let server_commit = get_server_commit(&node.name);
        let (current, status) = match &server_commit {
            Some(c) if c == &node.commit => (c.clone(), "OK"),
            Some(c) => (c.clone(), "DRIFT"),
            None => ("-".into(), "MISSING"),
        };
        println!("{:<40} {:<10} {:<10} {}", node.name, node.commit, current, status);
    }
}

fn install(name: Option<String>) {
    let nodes_file = load_nodes_yaml();
    let targets: Vec<&NodeEntry> = match &name {
        Some(n) => nodes_file.nodes.iter().filter(|e| e.name == *n).collect(),
        None => nodes_file.nodes.iter().collect(),
    };

    if targets.is_empty() {
        eprintln!("노드를 찾을 수 없습니다: {:?}", name);
        return;
    }

    for node in targets {
        if node_exists_on_server(&node.name) {
            println!("  [skip] {} (already exists)", node.name);
            continue;
        }

        println!("  [install] {} from {}", node.name, node.repo);
        let cmd = format!(
            "cd /opt/comfyui/custom_nodes && git clone {} {}",
            node.repo, node.name
        );
        match ssh_exec(&cmd) {
            Ok(_) => println!("    installed"),
            Err(e) => eprintln!("    FAILED: {}", e),
        }
    }

    println!("\nComfyUI 재시작 필요: comfyui-workflows restart (또는 수동)");
}

fn update(name: Option<String>) {
    let nodes_file = load_nodes_yaml();
    let targets: Vec<&NodeEntry> = match &name {
        Some(n) => nodes_file.nodes.iter().filter(|e| e.name == *n).collect(),
        None => nodes_file.nodes.iter().collect(),
    };

    for node in targets {
        if !node_exists_on_server(&node.name) {
            println!("  [missing] {} — install first", node.name);
            continue;
        }

        println!("  [update] {}", node.name);
        let cmd = format!(
            "cd /opt/comfyui/custom_nodes/{} && git pull --ff-only 2>&1",
            node.name
        );
        match ssh_exec(&cmd) {
            Ok(out) => {
                let trimmed = out.trim();
                if trimmed.contains("Already up to date") {
                    println!("    up to date");
                } else {
                    println!("    updated");
                }
            }
            Err(e) => eprintln!("    FAILED: {}", e),
        }
    }
}

fn pin(name: Option<String>) {
    let mut nodes_file = load_nodes_yaml();
    let mut changed = false;

    for node in &mut nodes_file.nodes {
        if let Some(ref n) = name {
            if &node.name != n { continue; }
        }

        if let Some(commit) = get_server_commit(&node.name) {
            if commit != node.commit {
                println!("  [pin] {} {} -> {}", node.name, node.commit, commit);
                node.commit = commit;
                changed = true;
            } else {
                println!("  [ok]  {} {}", node.name, node.commit);
            }
        } else {
            println!("  [skip] {} (not on server)", node.name);
        }
    }

    if changed {
        save_nodes_yaml(&nodes_file);
        println!("\nnodes.yaml updated");
    }
}

fn add(repo: &str, desc: Option<String>) {
    // repo URL에서 이름 추출
    let name = repo
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .rsplit('/')
        .next()
        .unwrap_or("unknown") // LINT_ALLOW: 기본값
        .to_string();

    let mut nodes_file = load_nodes_yaml();

    if nodes_file.nodes.iter().any(|n| n.name == name) {
        eprintln!("{} 는 이미 nodes.yaml에 있습니다", name);
        return;
    }

    // 서버에 설치
    println!("  [install] {} from {}", name, repo);
    let cmd = format!(
        "cd /opt/comfyui/custom_nodes && git clone {} {}",
        repo, name
    );
    match ssh_exec(&cmd) {
        Ok(_) => println!("    installed"),
        Err(e) => {
            eprintln!("    FAILED: {}", e);
            return;
        }
    }

    // 커밋 해시 가져오기
    let commit = get_server_commit(&name).unwrap_or_else(|| "unknown".into());

    // yaml에 추가
    nodes_file.nodes.push(NodeEntry {
        name: name.clone(),
        repo: repo.to_string(),
        commit,
        description: desc.unwrap_or_default(),
    });

    save_nodes_yaml(&nodes_file);
    println!("\n{} added to nodes.yaml", name);
    println!("ComfyUI 재시작 필요");
}

fn remove(name: &str) {
    let mut nodes_file = load_nodes_yaml();
    let before = nodes_file.nodes.len();
    nodes_file.nodes.retain(|n| n.name != name);

    if nodes_file.nodes.len() == before {
        eprintln!("{} 는 nodes.yaml에 없습니다", name);
        return;
    }

    // 서버에서 삭제
    println!("  [remove] {} from server", name);
    let cmd = format!("rm -rf /opt/comfyui/custom_nodes/{}", name);
    match ssh_exec(&cmd) {
        Ok(_) => println!("    removed"),
        Err(e) => eprintln!("    WARNING: {}", e),
    }

    save_nodes_yaml(&nodes_file);
    println!("\n{} removed from nodes.yaml", name);
    println!("ComfyUI 재시작 필요");
}
