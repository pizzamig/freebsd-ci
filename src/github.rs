use crate::Project;
use failure::Error;
use log::debug;
use log::trace;
use reqwest::header;
use serde_derive::Deserialize;
use std::fmt::Display;

#[derive(Debug, Deserialize)]
struct ReplyData {
    data: Repository,
}

#[derive(Debug, Deserialize)]
struct Repository {
    repository: RepoStatusJson,
    user: UserEmail,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct RepoStatusJson {
    isPrivate: bool,
    isArchived: bool,
    isLocked: bool,
    url: String,
    updatedAt: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct UserEmail {
    email: String,
}

#[derive(Debug)]
pub(crate) struct RepoStatus {
    pub is_private: bool,
    pub is_archived: bool,
    pub is_locked: bool,
    pub url: url::Url,
    pub update_at: chrono::DateTime<chrono::offset::FixedOffset>,
    pub email: String,
}

impl Display for RepoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(
            f,
            "  url: {}\n  private: {}\n  archived: {}\n  locked: {}\n updated at: {}",
            self.url, self.is_private, self.is_archived, self.is_locked, self.update_at
        )
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct AssetJson {
    pub(crate) id: u64,
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
struct ReleaseJson {
    id: u64,
    tag_name: String,
    assets: Vec<AssetJson>,
}

fn get_req_remaining(h: &header::HeaderMap) -> u32 {
    let rate_header = header::HeaderName::from_static("x-ratelimit-remaining");
    h.get(rate_header)
        .and_then(|x| x.to_str().ok())
        .and_then(|x| x.parse().ok())
        .unwrap_or(0)
}

pub(crate) fn get_status(prj: &Project, token: &str) -> Result<(RepoStatus, u32), Error> {
    let mut token_str = "token ".to_string();
    token_str.push_str(token);
    let mut h = header::HeaderMap::new();
    h.insert(reqwest::header::AUTHORIZATION, token_str.parse().unwrap());
    let q = format!(r#"{{ "query" : "query {{ repository(owner: \"{}\", name: \"{}\") {{ isPrivate isArchived isLocked updatedAt url }} user(login: \"{}\") {{ email }} }}" }}"#, prj.owner, prj.project, prj.owner);
    let client = reqwest::Client::builder().default_headers(h).build()?;
    let mut reply = client
        .post("https://api.github.com/graphql")
        .body(q)
        .send()?;
    let req_left = get_req_remaining(&reply.headers());
    let json = reply.text()?;
    trace!("output is {:?}", json);

    let rs: ReplyData = serde_json::from_str(&json)?;
    trace!("headers: {:?}", reply.headers());
    debug!("requests left: {:?}", req_left);
    let repos = rs.data.repository;

    let last_commit = chrono::DateTime::parse_from_rfc3339(&repos.updatedAt)?;
    debug!("Last commit in the repo: {:?}", last_commit);

    let the_url = url::Url::parse(&repos.url)?;
    debug!("The repo url : {:?}", the_url);
    let rs = RepoStatus {
        is_private: repos.isPrivate,
        is_archived: repos.isArchived,
        is_locked: repos.isLocked,
        url: the_url,
        update_at: last_commit,
        email: rs.data.user.email,
    };
    Ok((rs, req_left))
}

fn _get_release_id_from_json(_tag: &str, json: &str) -> Result<(u64, Vec<AssetJson>), Error> {
    let rss: ReleaseJson = serde_json::from_str(json)?;
    Ok((rss.id, rss.assets))
}

pub(crate) fn get_release_id(
    prj: &Project,
    tag: &str,
    token: &str,
) -> Result<(u64, Vec<AssetJson>, u32), Error> {
    let mut token_str = "token ".to_string();
    token_str.push_str(token);
    let mut h = header::HeaderMap::new();
    h.insert(reqwest::header::AUTHORIZATION, token_str.parse().unwrap());
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/tags/{}",
        prj.owner, prj.project, tag
    );
    let client = reqwest::Client::builder().default_headers(h).build()?;
    let mut reply = client.get(&url).send()?;
    let req_left = get_req_remaining(&reply.headers());
    let json = reply.text()?;
    trace!("output is {:?}", json);
    let (release_id, assets) = _get_release_id_from_json(tag, &json)?;
    Ok((release_id, assets, req_left))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_release_id_from_json() {
        let json = r#"
        {
            "url": "https://api.github.com/repos/pizzamig/ci-test/releases/15076202",
            "id": 15076202,
            "tag_name": "0.1.1",
            "assets": [],
            "author": {
              "login": "pizzamig"
            }
        }
        "#;
        let (id, assets) = _get_release_id_from_json("0.1.1", json).unwrap();
        assert!(id == 15_076_202);
        assert!(assets.is_empty());
    }

    #[test]
    fn test_get_release_id_from_json_assets() {
        let json = r#"
        {
            "url": "https://api.github.com/repos/pizzamig/ci-test/releases/15076202",
            "id": 15076202,
            "tag_name": "0.1.1",
            "author": {
              "login": "pizzamig"
            },
            "assets": [
            {
                "id": 10739504,
                "name": "FreeBSD-11.2-ci-test.tar.gz",
                "download_count": 0
            },
            {
                "id": 10739535,
                "name": "FreeBSD-12.0-ci-test.tar.gz",
                "download_count": 1
            }
            ]
        }
        "#;
        let (id, assets) = _get_release_id_from_json("0.1.1", json).unwrap();
        assert!(id == 15_076_202);
        assert_eq!(assets.len(), 2);
    }
}
