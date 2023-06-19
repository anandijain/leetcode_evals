use indicatif::{self, ProgressIterator};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, REFERER};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use std::fs::create_dir_all;
use std::fs::{write, File};
use std::io::Write;
use std::thread::sleep;

const OPENAI_API_KEY: &str = env!("OPENAI_API_KEY");
const OPENAI_GPT_MODEL: &str = "gpt-3.5-turbo";
// const OPENAI_GPT_MODEL: &str = "gpt-4";

async fn get_problemset() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let base = "https://leetcode.com/problems/";

    let cookie = "csrftoken=U38VtRxr5YfvLADqqgCQvJkSHYwu3X6laeFJqGf464BYKRZ0IGF23hu9DUdQKBJJ; messages=\"12877f56d355501b9812ecfb2d621ba942950006$[[\\\"__json_message\\\",0,25,\\\"Successfully signed in as anandjain.\\\"]]\"; LEETCODE_SESSION=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJfYXV0aF91c2VyX2lkIjoiMzYwNTc2OCIsIl9hdXRoX3VzZXJfYmFja2VuZCI6ImRqYW5nby5jb250cmliLmF1dGguYmFja2VuZHMuTW9kZWxCYWNrZW5kIiwiX2F1dGhfdXNlcl9oYXNoIjoiMjEwNWIxYWQ3ZjNhOWNhMmZiMzc1N2FhNzEzZjQ4MmFiNTMxOWVmNyIsImlkIjozNjA1NzY4LCJlbWFpbCI6ImRhaG1laC5hbG1vc0BnbWFpbC5jb20iLCJ1c2VybmFtZSI6ImFuYW5kamFpbiIsInVzZXJfc2x1ZyI6ImFuYW5kamFpbiIsImF2YXRhciI6Imh0dHBzOi8vYXNzZXRzLmxlZXRjb2RlLmNvbS91c2Vycy9hdmF0YXJzL2F2YXRhcl8xNjc4OTA5MTg5LnBuZyIsInJlZnJlc2hlZF9hdCI6MTY4MDk3NTA3MiwiaXAiOiI1MC4xNzAuNDQuMjEwIiwiaWRlbnRpdHkiOiI3MjNjNTEyNjNjODBmNmJlNzlmYTIxMTkxZWUwYjM4NyIsInNlc3Npb25faWQiOjM3Nzk3NzYzLCJfc2Vzc2lvbl9leHBpcnkiOjEyMDk2MDB9.9HpZ8N5J9Lzfz7LHcNryx9sjIDtshhXLfQ3MSmBMFkE; NEW_PROBLEMLIST_PAGE=1; _dd_s=rum=0&expire=1681147877868";
    let token = "U38VtRxr5YfvLADqqgCQvJkSHYwu3X6laeFJqGf464BYKRZ0IGF23hu9DUdQKBJJ";

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("cookie", HeaderValue::from_static(cookie));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://leetcode.com/problemset/all/"),
    );
    headers.insert("x-csrftoken", HeaderValue::from_static(token));

    let data = json!({
        "query": "\n    query problemsetQuestionList($categorySlug: String, $limit: Int, $skip: Int, $filters: QuestionListFilterInput) {\n  problemsetQuestionList: questionList(\n    categorySlug: $categorySlug\n    limit: $limit\n    skip: $skip\n    filters: $filters\n  ) {\n    total: totalNum\n    questions: data {\n      acRate\n      difficulty\n      freqBar\n      frontendQuestionId: questionFrontendId\n      isFavor\n      paidOnly: isPaidOnly\n      status\n      title\n      titleSlug\n      topicTags {\n        name\n        id\n        slug\n      }\n      hasSolution\n      hasVideoSolution\n    }\n  }\n}\n",
        "variables": {
            "categorySlug": "",
            "skip": 0,
            "limit": 3000,
            "filters": {}
        },
        "operationName": "problemsetQuestionList"
    });

    let resp = client
        .post("https://leetcode.com/graphql/")
        .headers(headers)
        .json(&data)
        .send()
        .await?
        .text()
        .await?;

    let mut file = File::create("problemset.json").expect("create failed");
    file.write_all(resp.as_bytes()).expect("write failed");

    Ok(())
}

async fn get_problems_and_code() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let base = "https://leetcode.com/problems/";

    let cookie = "csrftoken=U38VtRxr5YfvLADqqgCQvJkSHYwu3X6laeFJqGf464BYKRZ0IGF23hu9DUdQKBJJ; messages=\"12877f56d355501b9812ecfb2d621ba942950006$[[\\\"__json_message\\\",0,25,\\\"Successfully signed in as anandjain.\\\"]]\"; LEETCODE_SESSION=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJfYXV0aF91c2VyX2lkIjoiMzYwNTc2OCIsIl9hdXRoX3VzZXJfYmFja2VuZCI6ImRqYW5nby5jb250cmliLmF1dGguYmFja2VuZHMuTW9kZWxCYWNrZW5kIiwiX2F1dGhfdXNlcl9oYXNoIjoiMjEwNWIxYWQ3ZjNhOWNhMmZiMzc1N2FhNzEzZjQ4MmFiNTMxOWVmNyIsImlkIjozNjA1NzY4LCJlbWFpbCI6ImRhaG1laC5hbG1vc0BnbWFpbC5jb20iLCJ1c2VybmFtZSI6ImFuYW5kamFpbiIsInVzZXJfc2x1ZyI6ImFuYW5kamFpbiIsImF2YXRhciI6Imh0dHBzOi8vYXNzZXRzLmxlZXRjb2RlLmNvbS91c2Vycy9hdmF0YXJzL2F2YXRhcl8xNjc4OTA5MTg5LnBuZyIsInJlZnJlc2hlZF9hdCI6MTY4MDk3NTA3MiwiaXAiOiI1MC4xNzAuNDQuMjEwIiwiaWRlbnRpdHkiOiI3MjNjNTEyNjNjODBmNmJlNzlmYTIxMTkxZWUwYjM4NyIsInNlc3Npb25faWQiOjM3Nzk3NzYzLCJfc2Vzc2lvbl9leHBpcnkiOjEyMDk2MDB9.9HpZ8N5J9Lzfz7LHcNryx9sjIDtshhXLfQ3MSmBMFkE; NEW_PROBLEMLIST_PAGE=1; _dd_s=rum=0&expire=1681147877868";
    let token = "U38VtRxr5YfvLADqqgCQvJkSHYwu3X6laeFJqGf464BYKRZ0IGF23hu9DUdQKBJJ";

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("cookie", HeaderValue::from_static(cookie));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://leetcode.com/problemset/all/"),
    );
    headers.insert("x-csrftoken", HeaderValue::from_static(token));

    let v: Value =
        serde_json::from_str(&std::fs::read_to_string("problemset.json").unwrap()).unwrap();
    let questions = v["data"]["problemsetQuestionList"]["questions"]
        .as_array()
        .unwrap();

    for question in questions {
        if question["paidOnly"].as_bool().unwrap_or(false) {
            continue;
        }
        let slug = question["titleSlug"].as_str().unwrap();
        let url = format!("{}{}", base, slug.clone());

        headers.insert(REFERER, url.parse().unwrap());

        let json_body1 = serde_json::json!({
            "query": "\n    query questionContent($titleSlug: String!) {\n  question(titleSlug: $titleSlug) {\n    content\n    mysqlSchemas\n  }\n}\n    ",
            "variables": {
                "titleSlug": slug,
            },
            "operationName": "questionContent"
        });

        let res1 = client
            .post("https://leetcode.com/graphql/")
            .headers(headers.clone())
            .json(&json_body1)
            .send()
            .await?;

        let data_path = format!(
            "/Users/anand/.rust/dev/leetcode_evals/data/data/{}/prompt/",
            slug
        );
        create_dir_all(&data_path).unwrap();
        write(
            format!("{}{}_prompt.json", data_path, slug),
            res1.text().await?,
        )
        .unwrap();

        let json_body2 = serde_json::json!({
            "query": "\n    query questionEditorData($titleSlug: String!) {\n  question(titleSlug: $titleSlug) {\n    questionId\n    questionFrontendId\n    codeSnippets {\n      lang\n      langSlug\n      code\n    }\n    envInfo\n    enableRunCode\n  }\n}\n    ",
            "variables": {
                "titleSlug": slug,
            },
            "operationName": "questionEditorData"
        });

        let res2 = client
            .post("https://leetcode.com/graphql/")
            .headers(headers.clone())
            .json(&json_body2)
            .send()
            .await?;

        write(
            format!("{}{}_code.json", data_path, slug),
            res2.text().await?,
        )
        .unwrap();
    }

    Ok(())
}

async fn fetch_openai_completion(message: &str) -> Result<Value, Box<dyn Error>> {
    let client = Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", OPENAI_API_KEY))?,
    );

    let data = json!({
        "model": OPENAI_GPT_MODEL,
        "messages": [
            {"role": "user", "content": message}
        ]
    });

    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .headers(headers)
        .json(&data)
        .send()
        .await?;

    let result: Value = res.json().await?;

    Ok(result)
}

fn extract_content(response: &Value) -> Result<String, Box<dyn Error>> {
    let content = response["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Unable to extract content")?
        .to_string();

    Ok(content)
}

// Function to get the directory path
fn get_directory_path(title_slug: &str) -> PathBuf {
    Path::new("/Users/anand/.rust/dev/leetcode_evals/data/data/")
        .join(title_slug)
        .join("prompt")
}

// Function to get the directory path
fn get_soln_directory_path(title_slug: &str) -> PathBuf {
    Path::new("/Users/anand/.rust/dev/leetcode_evals/data/data/")
        .join(title_slug)
        .join("solutions")
}

// Function to get the prompt path
fn get_prompt_path(title_slug: &str) -> PathBuf {
    let dir_path = get_directory_path(title_slug);
    let f = dir_path.join(format!("{}_prompt.json", title_slug));
    // println!("{:?}", f);
    f
}

fn get_title_slug(question: &Value) -> String {
    question["titleSlug"].as_str().unwrap().to_string()
}

// Function to get the code path
fn get_code_path(title_slug: &str) -> PathBuf {
    let dir_path = get_directory_path(title_slug);
    dir_path.join(format!("{}_code.json", title_slug))
}

// Function to read prompt JSON file
fn get_prompt(title_slug: &str) -> Result<Value, serde_json::Error> {
    let prompt_path = get_prompt_path(title_slug);
    let prompt_str = fs::read_to_string(prompt_path).unwrap();
    serde_json::from_str(&prompt_str)
}

// Function to read code JSON file
fn get_code(title_slug: &str) -> Result<Value, serde_json::Error> {
    let code_path = get_code_path(title_slug);
    let code_str = fs::read_to_string(code_path).unwrap();
    serde_json::from_str(&code_str)
}

fn get_content_from_json(prompt_json: &Value) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(content) = prompt_json["data"]["question"]["content"].as_str() {
        Ok(content.to_string())
    } else {
        Err("Content not found in JSON".into())
    }
}
fn get_content_from_title_slug(title_slug: &str) -> Result<String, Box<dyn std::error::Error>> {
    let prompt_json = get_prompt(title_slug)?;
    get_content_from_json(&prompt_json)
}

fn get_code_snippets(data: &Value) -> Option<&Value> {
    // Navigate to the 'codeSnippets' field.
    data.get("data")?.get("question")?.get("codeSnippets")
}

// fn get_code_snippets_from_question(question: &Value) -> Option<&Value> {
//     get_code_snippets(get_code(&get_title_slug(question)).unwrap())
// }

fn get_code_for_lang(code_snippets: &Value, lang: &str) -> Result<String, Box<dyn Error>> {
    code_snippets
        .as_array()
        .ok_or("Expected array")?
        .iter()
        .filter_map(|s| s.as_object())
        .find(|s| s.get("langSlug").and_then(Value::as_str) == Some(lang))
        .and_then(|s| s.get("code").and_then(Value::as_str))
        .map(str::to_string)
        .ok_or_else(|| "No matching snippet for the specified language".into())
}

fn has_lang(code_snippets: &Value, lang: &str) -> bool {
    match get_code_for_lang(code_snippets, lang) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn build_prompt(title_slug: &str, lang: &str) -> Result<(String, String), Box<dyn Error>> {
    // Read the prompt and get the content
    let prompt_json = get_prompt(title_slug)?;
    let content = get_content_from_json(&prompt_json)?;

    // Read the code JSON and get the code snippet for the specified language
    let code_json = get_code(title_slug)?;
    let code_snippets = get_code_snippets(&code_json).ok_or("Code snippets not found")?;
    let code = get_code_for_lang(code_snippets, lang)?;

    Ok((content, code))
}

fn extract_codeblocks(text: &str) -> Vec<String> {
    let codeblock_regex = Regex::new(r"```(?:\w*\n)?((?s:.+?))```").unwrap();
    codeblock_regex
        .captures_iter(text)
        .map(|cap| cap[1].to_string())
        .collect()
}

fn soln_fn(title_slug: &str, lang: &str, model: &str) -> PathBuf {
    get_soln_directory_path(title_slug).join(format!("{}_{}_{}.json", title_slug, lang, model))
}

async fn save_solution(title_slug: &str, lang: &str, v: &Value) -> std::io::Result<()> {
    let dir_path = get_soln_directory_path(title_slug);
    tokio::fs::create_dir_all(&dir_path).await?;
    let file_path = dir_path.join(soln_fn(title_slug, lang, OPENAI_GPT_MODEL));
    println!("{:?}", file_path);
    tokio::fs::write(file_path, v.to_string()).await
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let lang = "python3";
    let start_title_slug = "dungeon-game";

    let client = reqwest::Client::new();
    let base = "https://leetcode.com/problems/";

    let cookie = "csrftoken=U38VtRxr5YfvLADqqgCQvJkSHYwu3X6laeFJqGf464BYKRZ0IGF23hu9DUdQKBJJ; messages=\"12877f56d355501b9812ecfb2d621ba942950006$[[\\\"__json_message\\\",0,25,\\\"Successfully signed in as anandjain.\\\"]]\"; LEETCODE_SESSION=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJfYXV0aF91c2VyX2lkIjoiMzYwNTc2OCIsIl9hdXRoX3VzZXJfYmFja2VuZCI6ImRqYW5nby5jb250cmliLmF1dGguYmFja2VuZHMuTW9kZWxCYWNrZW5kIiwiX2F1dGhfdXNlcl9oYXNoIjoiMjEwNWIxYWQ3ZjNhOWNhMmZiMzc1N2FhNzEzZjQ4MmFiNTMxOWVmNyIsImlkIjozNjA1NzY4LCJlbWFpbCI6ImRhaG1laC5hbG1vc0BnbWFpbC5jb20iLCJ1c2VybmFtZSI6ImFuYW5kamFpbiIsInVzZXJfc2x1ZyI6ImFuYW5kamFpbiIsImF2YXRhciI6Imh0dHBzOi8vYXNzZXRzLmxlZXRjb2RlLmNvbS91c2Vycy9hdmF0YXJzL2F2YXRhcl8xNjc4OTA5MTg5LnBuZyIsInJlZnJlc2hlZF9hdCI6MTY4MDk3NTA3MiwiaXAiOiI1MC4xNzAuNDQuMjEwIiwiaWRlbnRpdHkiOiI3MjNjNTEyNjNjODBmNmJlNzlmYTIxMTkxZWUwYjM4NyIsInNlc3Npb25faWQiOjM3Nzk3NzYzLCJfc2Vzc2lvbl9leHBpcnkiOjEyMDk2MDB9.9HpZ8N5J9Lzfz7LHcNryx9sjIDtshhXLfQ3MSmBMFkE; NEW_PROBLEMLIST_PAGE=1; _dd_s=rum=0&expire=1681147877868";
    let token = "U38VtRxr5YfvLADqqgCQvJkSHYwu3X6laeFJqGf464BYKRZ0IGF23hu9DUdQKBJJ";

    let mut oai_headers = HeaderMap::new();
    oai_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    oai_headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", OPENAI_API_KEY)).unwrap(),
    );

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("cookie", HeaderValue::from_static(cookie));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://leetcode.com/problemset/all/"),
    );
    headers.insert("x-csrftoken", HeaderValue::from_static(token));

    let v: Value =
        serde_json::from_str(&std::fs::read_to_string("problemset.json").unwrap()).unwrap();

    let questions = v["data"]["problemsetQuestionList"]["questions"]
        .as_array()
        .unwrap();

    let free_questions: Vec<_> = questions
        .iter()
        .filter(|q| match q["paidOnly"].as_bool() {
            Some(paid_only) => {
                !paid_only
                    && has_lang(
                        get_code_snippets(&get_code(&get_title_slug(q)).unwrap()).unwrap(),
                        lang,
                    )
            }
            None => false,
        })
        .collect();

    let start_index = free_questions
        .iter()
        .position(|q| q["titleSlug"].as_str().unwrap() == start_title_slug)
        .unwrap();
    let mut i = 0;
    for q in free_questions.iter().skip(start_index).progress() {
        let title_slug = q["titleSlug"].as_str().unwrap();
        // println!("{}: {}", i, title_slug);
        let (content, _code) = build_prompt(&title_slug, &lang).unwrap();
        let full_prompt = format!(
            "{}\n\n{}\n\nWrite out full solution in a markdown codeblock:",
            content, _code
        );
        let v = fetch_openai_completion(&full_prompt).await.unwrap();
        save_solution(&title_slug, &lang, &v).await.unwrap();
        i += 1;
    }
    Ok(())
}
