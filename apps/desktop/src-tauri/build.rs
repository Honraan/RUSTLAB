fn find_webview2_in_out(dir: &std::path::Path, arch_dir: &str) -> Option<std::path::PathBuf> {
    let cand = dir.join("out").join(arch_dir).join("WebView2Loader.dll");
    if cand.is_file() {
        return Some(cand);
    }
    None
}

fn stage_webview2_loader() {
    #[cfg(target_os = "windows")]
    {
        let Ok(out_dir) = std::env::var("OUT_DIR") else {
            return;
        };
        let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") else {
            return;
        };
        let arch_dir = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
            Ok("aarch64") => "arm64",
            Ok("x86") => "x86",
            _ => "x64",
        };

        let mut candidates = Vec::new();
        // Sobe até target/<profile>/build (3 ou 4 níveis acima de OUT_DIR dependendo do layout Cargo)
        let mut cursor = std::path::Path::new(&out_dir).to_path_buf();
        for _ in 0..4 {
            if let Some(parent) = cursor.parent() {
                cursor = parent.to_path_buf();
                candidates.push(cursor.clone());
            }
        }

        let mut found = None;
        'search: for root in candidates {
            let Ok(entries) = std::fs::read_dir(&root) else {
                continue;
            };
            for entry in entries.flatten() {
                let name = entry.file_name();
                if !name.to_string_lossy().starts_with("webview2-com-sys") {
                    continue;
                }
                if let Some(p) = find_webview2_in_out(&entry.path(), arch_dir) {
                    found = Some(p);
                    break 'search;
                }
                if let Ok(subentries) = std::fs::read_dir(entry.path()) {
                    for sub in subentries.flatten() {
                        if let Some(p) = find_webview2_in_out(&sub.path(), arch_dir) {
                            found = Some(p);
                            break 'search;
                        }
                    }
                }
            }
        }

        if let Some(dll_src) = found {
            let dest_dir = std::path::Path::new(&manifest_dir).join("resources");
            let _ = std::fs::create_dir_all(&dest_dir);
            let _ = std::fs::copy(&dll_src, dest_dir.join("WebView2Loader.dll"));
        }
    }
}

fn main() {
    stage_webview2_loader();
    tauri_build::build();
    #[cfg(target_os = "windows")]
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        for name in ["libresource.a", "resource.lib"] {
            let cand = std::path::Path::new(&out_dir).join(name);
            if cand.is_file() {
                println!("cargo:rustc-link-arg-tests={}", cand.display());
            }
        }
    }
}
