use crate::Opts;
use anyhow::anyhow;
use base64::{decode, encode};
use chrono::{Datelike, Local};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub async fn upload_to_github(opt: Arc<Opts>, content: String) -> anyhow::Result<()> {
    let (uri, data) = get_github_file_content(&opt.github_token, &opt.repo).await?;

    let time = Local::now();
    let month = format!("# {}-{:02}", time.year(), time.month());
    let day = format!("## {}-{:02}-{:02}", time.year(), time.month(), time.day());

    let new_data = if let Some((sha, mut old_content)) = data {
        old_content = old_content.replace("\n", "");
        let vec = decode(old_content).expect("cannot decode");
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

    update_github_content(&uri, &opt.github_token, new_data.0, new_data.1).await?;
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
) -> anyhow::Result<(String, Option<(String, String)>)> {
    info!("get github file");
    let time = Local::now();
    let path = format!("{}/{:02}.md", time.year(), time.month());
    let uri = format!("https://api.github.com/repos/{}/contents/{}", repo, path);
    let client = reqwest::Client::new();
    let result = client
        .get(&uri)
        .header("Accept", "application/vnd.github.v3+json")
        .header("Authorization", format!("Bearer {}", token))
        .header(
            "User-Agent",
            "darling 0.1.5 (https://github.com/kilerd/darling)",
        )
        .send()
        .await?
        .json::<FileContent>()
        .await?;
    match result {
        FileContent::Ok { sha, content } => Ok((uri, Some((sha, content)))),
        FileContent::NotFound { .. } => Ok((uri, None)),
    }
}

#[derive(Serialize, Debug)]
pub struct UpdateGithubFile {
    message: String,
    content: String,
    sha: Option<String>,
}

pub async fn update_github_content(
    uri: &str,
    token: &str,
    sha: Option<String>,
    content: String,
) -> anyhow::Result<()> {
    info!("update github content");
    let encoded_content = encode(content);
    let req = UpdateGithubFile {
        message: "journal: update by telegram bot".to_string(),
        content: encoded_content,
        sha,
    };
    let client = reqwest::Client::new();
    let res = client
        .put(uri)
        .header("Accept", "application/vnd.github.v3+json")
        .header("Authorization", format!("Bearer {}", token))
        .header(
            "User-Agent",
            "darling 0.1.5 (https://github.com/kilerd/darling)",
        )
        .json(&req)
        .send()
        .await?;
    if res.status().is_success() {
        Ok(())
    } else {
        let res_text = res.text().await?;
        Err(anyhow!("upload to github fail: {}", res_text))
    }
}
