use crate::Opts;
use base64::{decode, encode};
use chrono::{Datelike, Local};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub async fn upload_to_github(
    opt: Arc<Opts>,
    content: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let (uri, data) = get_github_file_content(&opt.github_token, &opt.repo).await;

    let time = Local::now();
    let month = format!("# {}-{:02}", time.year(), time.month());
    let day = format!("## {}-{:02}-{:02}", time.year(), time.month(), time.day());

    let new_data = if let Some((sha, mut old_content)) = data {
        old_content = old_content.replace("\n", "");
        let vec = decode(dbg!(old_content)).expect("cannot decode");
        let decoded_content = String::from_utf8_lossy(&vec[..]);

        if decoded_content.contains(&format!("\n{}\n", day)) {
            (
                Some(sha),
                format!("{}\n\n---\n{}", decoded_content, content),
            )
        } else {
            (
                Some(sha),
                format!("{}\n{}\n{}", decoded_content, day, content),
            )
        }
    } else {
        let content_with_header = format!("{}\n{}\n{}", month, day, content);
        (None, content_with_header)
    };

    update_github_content(&uri, &opt.github_token, new_data.0, new_data.1).await;
    Ok(())
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum FileContent {
    Ok {
        sha: String,
        content: String,
    },
    NotFound {
        message: String,
        documentation_url: String,
    },
}

pub async fn get_github_file_content(
    token: &str,
    repo: &str,
) -> (String, Option<(String, String)>) {
    loop {
        let time = Local::now();
        let path = format!("{}/{:02}.md", time.year(), time.month());
        let uri = format!("https://api.github.com/repos/{}/contents/{}", repo, path);
        let x = surf::get(dbg!(&uri))
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", format!("Bearer {}", token))
            .recv_json::<FileContent>()
            .await;
        if let Ok(ret) = x {
            return match ret {
                FileContent::Ok { sha, content } => (uri, Some((sha, content))),
                FileContent::NotFound { .. } => (uri, None),
            };
        }
    }
}

#[derive(Serialize, Debug)]
pub struct UpdateGithubFile {
    message: String,
    content: String,
    sha: Option<String>,
}

pub async fn update_github_content(uri: &str, token: &str, sha: Option<String>, content: String) {
    let encoded_content = encode(content);
    let req = UpdateGithubFile {
        message: "journal: update by telegram bot".to_string(),
        content: encoded_content,
        sha,
    };
    loop {
        let x = surf::put(uri)
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", format!("Bearer {}", token))
            .body(surf::Body::from_json(&req).expect("cannot to body bytes"))
            .recv_string()
            .await;
        if let Ok(_) = x {
            break;
        }
    }
}
