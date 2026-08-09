/*
 * Copyright 2026 Jhe-An Lee
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());

    let webui_dir = manifest_dir.join("../webui");

    let webui_src = webui_dir.join("src");
    let webui_dist = webui_dir.join("dist");

    if env::var_os("DOCKER_BUILD").is_none() {
        println!("cargo:rerun-if-changed={}", webui_src.display());
    }

    if !webui_dist.exists() {
        println!("cargo:warning=Web UI build output missing. Run `cd webui && npm run build`");
        return;
    }

    if env::var_os("DOCKER_BUILD").is_none() {
        let latest_src_modification = latest_dir_modification(&webui_src);
        let latest_dist_modification = latest_dir_modification(&webui_dist);

        if latest_src_modification > latest_dist_modification {
            println!(
                "cargo:warning=Web UI source files are newer than the build output. Run `cd webui && npm run build`"
            );
        }
    }
}

fn latest_dir_modification(dir: &Path) -> Option<std::time::SystemTime> {
    fs::read_dir(dir)
        .inspect_err(|e| {
            println!("cargo:warning=Failed to check Web UI source files: {e}");
        })
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            if entry.path().is_dir() {
                latest_dir_modification(&entry.path())
            } else {
                let metadata = fs::metadata(entry.path()).ok();
                if let Some(metadata) = metadata {
                    metadata.modified().ok()
                } else {
                    None
                }
            }
        })
        .max()
}
