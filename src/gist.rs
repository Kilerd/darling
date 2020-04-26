use std::env;
use serde::{Serialize, Deserialize};

mod detail {
    use serde::{Serialize, Deserialize};
    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Root {
        pub data: Data,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Data {
        pub viewer: Viewer,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Viewer {
        pub gist: Gist,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Gist {
        pub files: Vec<File>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct File {
        pub encoded_name: String,
        pub is_image: bool,
        pub name: String,
        pub text: String,
    }
}

mod list {
    use serde::{Serialize, Deserialize};
    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Root {
        pub data: Data,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Data {
        pub viewer: Viewer,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Viewer {
        pub gists: Gists,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Gists {
        pub nodes: Vec<Node>,
        pub page_info: PageInfo,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Node {
        pub is_public: bool,
        pub description: String,
        pub id: String,
        pub name: String,
        pub url: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PageInfo {
        pub has_next_page: bool,
        pub end_cursor: String,
    }

}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub query: Option<String>,
    pub mutation: Option<String>
}


const FIRST_GIST_QUERY : &'static str = r#"query {viewer {gists(first: 100,privacy:SECRET) {nodes {isPublic,description,id,name,url}pageInfo {hasNextPage,endCursor}}}}"#;

pub async fn get_gist_file_content(name: impl AsRef<str>) -> Result<Vec<detail::File>, surf::Exception> {
    let github_token = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set");
    let string = format!("{{viewer{{gist(name: \"{}\") {{files {{encodedName,isImage,name,text}}}}}}}}", name.as_ref());
    let request = Request {
        query: Some(string),
        mutation: None
    };
    let x = surf::post("https://api.github.com/graphql")
        .set_header("Authorization", format!("Bearer {}", github_token))
        .body_json(&request)?
        .await?
        .body_json::<detail::Root>()
        .await?;
    Ok(x.data.viewer.gist.files)
}

pub async fn get_secret_gist_list() -> Result<Vec<list::Node>, surf::Exception> {
    let github_token = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set");

    let request = Request {
        query: Some(FIRST_GIST_QUERY.to_string()),
        mutation: None
    };
    let string = surf::post("https://api.github.com/graphql")
        .set_header("Authorization", format!("Bearer {}", github_token))
        .body_json(&request)?
        // .set_header("Content-Type", "application/json"))
        .await?
        .body_json::<list::Root>()
        .await?;
    Ok(string.data.viewer.gists.nodes)
}