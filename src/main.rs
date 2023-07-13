use indicatif::{self, ProgressIterator};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, REFERER, USER_AGENT};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{prelude::*, Duration};
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs::create_dir_all;
use std::fs::{write, File};
use std::io::Write;
use std::thread::sleep;

#[macro_use]
extern crate lazy_static;

const OPENAI_API_KEY: &str = env!("OPENAI_API_KEY");
const OPENAI_GPT_MODEL: &str = "gpt-3.5-turbo";
// const OPENAI_GPT_MODEL: &str = "gpt-4";
const CSRF_TOKEN: &str = "vqXWMXcAkYJu75Pid4qoJCDpQgceZ0zFqgN2AeaPELpaE89289U7USSjkdYDrXXo";
const COOKIE: &str = "csrftoken=vqXWMXcAkYJu75Pid4qoJCDpQgceZ0zFqgN2AeaPELpaE89289U7USSjkdYDrXXo; messages=\"ce012aae62d93e5358036fbb514a4ba766fad69e$[[\\\"__json_message\\\",0,25,\\\"Successfully signed in as anandjain.\\\"],[\\\"__json_message\\\",0,25,\\\"Successfully signed in as anandjain.\\\"]]; LEETCODE_SESSION=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJfYXV0aF91c2VyX2lkIjoiMzYwNTc2OCIsIl9hdXRoX3VzZXJfYmFja2VuZCI6ImRqYW5nby5jb250cmliLmF1dGguYmFja2VuZHMuTW9kZWxCYWNrZW5kIiwiX2F1dGhfdXNlcl9oYXNoIjoiMjEwNWIxYWQ3ZjNhOWNhMmZiMzc1N2FhNzEzZjQ4MmFiNTMxOWVmNyIsImlkIjozNjA1NzY4LCJlbWFpbCI6ImRhaG1laC5hbG1vc0BnbWFpbC5jb20iLCJ1c2VybmFtZSI6ImFuYW5kamFpbiIsInVzZXJfc2x1ZyI6ImFuYW5kamFpbiIsImF2YXRhciI6Imh0dHBzOi8vYXNzZXRzLmxlZXRjb2RlLmNvbS91c2Vycy9hdmF0YXJzL2F2YXRhcl8xNjc4OTA5MTg5LnBuZyIsInJlZnJlc2hlZF9hdCI6MTY4OTEzODk2MSwiaXAiOiI2Ni4zMC4yMjMuOSIsImlkZW50aXR5IjoiNWYwZmY1ZDg3OTllZDRjMGVkMzU1ZmE0NzRhN2JiYzIiLCJzZXNzaW9uX2lkIjo0MjQzNjg5NCwiX3Nlc3Npb25fZXhwaXJ5IjoxMjA5NjAwfQ.NLrgwyu-mQchlpOr0LzaB_FGUOdguZlWmGNGkYZEDLs; _dd_s=rum=1&id=0117f673-b41c-4903-a298-74dc9c61d369&created=1689138954646&expire=1689139941224";
const TOKEN: &str = "vqXWMXcAkYJu75Pid4qoJCDpQgceZ0zFqgN2AeaPELpaE89289U7USSjkdYDrXXo";

const USER_AGENT_STR: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36";
const GPT_3_SOLVED_LANGS: [&str; 6] = ["c", "cpp", "java", "javascript", "python3", "rust"];

static SLUG_ID_MAP: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let data = fs::read_to_string("problemset.json").expect("Failed to read problemset.json");
    let v: Value = serde_json::from_str(&data).expect("Failed to parse JSON data");
    build_slug_to_id_map(&v)
});

lazy_static! {
    static ref MARKDOWN_PREFIXES: HashMap<&'static str, &'static str> = {
        vec![
            ("golang", "go"),
            ("python3", "python"),
            ("javascript", "javascript"),
            ("typescript", "typescript"),
            ("java", "java"),
            ("c", "c"),
            ("cpp", "cpp"),
            ("csharp", "csharp"),
            ("ruby", "ruby"),
            ("swift", "swift"),
            ("kotlin", "kotlin"),
            ("rust", "rust"),
            ("shell", "shell"),
            ("r", "r"),
            ("scala", "scala"),
            ("php", "php"),
            ("perl", "perl"),
            ("lua", "lua"),
            ("haskell", "haskell"),
            ("groovy", "groovy"),
            ("dart", "dart"),
        ]
        .into_iter()
        .collect()
    };
}

fn get_markdown_prefix(lang_slug: &str) -> Option<&&str> {
    MARKDOWN_PREFIXES.get(lang_slug)
}

async fn get_problemset() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let base = "https://leetcode.com/problems/";

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("cookie", HeaderValue::from_static(COOKIE));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://leetcode.com/problemset/all/"),
    );
    headers.insert("x-csrftoken", HeaderValue::from_static(TOKEN));

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

pub fn build_slug_to_id_map(v: &Value) -> HashMap<String, String> {
    let mut slug_to_id = HashMap::new();

    if let Value::Array(questions) = &v["data"]["problemsetQuestionList"]["questions"] {
        for question in questions {
            if let (Value::String(slug), Value::String(id)) =
                (&question["titleSlug"], &question["frontendQuestionId"])
            {
                slug_to_id.insert(slug.clone(), id.clone());
            }
        }
    }

    slug_to_id
}

async fn get_problems_and_code() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let base = "https://leetcode.com/problems/";

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("cookie", HeaderValue::from_static(COOKIE));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://leetcode.com/problemset/all/"),
    );
    headers.insert("x-csrftoken", HeaderValue::from_static(TOKEN));

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

async fn fetch_openai_completion(message: &str, model: &str) -> Result<Value, Box<dyn Error>> {
    let client = Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", OPENAI_API_KEY))?,
    );

    let data = json!({
        "model": model,
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
    Path::new("/Users/anand/.rust/dev/leetcode_evals/data/data/").join(title_slug)
}

// Function to get the directory path
fn get_prompt_dir(title_slug: &str) -> PathBuf {
    Path::new("/Users/anand/.rust/dev/leetcode_evals/data/data/")
        .join(title_slug)
        .join("prompt")
}

// Function to get the directory path
fn get_solution_dir(title_slug: &str) -> PathBuf {
    Path::new("/Users/anand/.rust/dev/leetcode_evals/data/data/")
        .join(title_slug)
        .join("solutions")
}

// Function to get the directory path
fn get_submission_dir(title_slug: &str) -> PathBuf {
    Path::new("/Users/anand/.rust/dev/leetcode_evals/data/data/")
        .join(title_slug)
        .join("submissions")
}

// Function to get the prompt path
fn get_prompt_path(title_slug: &str) -> PathBuf {
    let dir_path = get_prompt_dir(title_slug);
    let f = dir_path.join(format!("{}_prompt.json", title_slug));
    // println!("{:?}", f);
    f
}

fn get_title_slug(question: &Value) -> String {
    question["titleSlug"].as_str().unwrap().to_string()
}

// Function to get the code path
fn get_code_path(title_slug: &str) -> PathBuf {
    let dir_path = get_prompt_dir(title_slug);
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
    get_code_for_lang(code_snippets, lang).is_ok()
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

fn extract_specific_lang_codeblocks(text: &str, lang: &str) -> Vec<String> {
    let markdown_prefix = get_markdown_prefix(lang).unwrap();
    let codeblock_regex =
        Regex::new(&format!(r"```{}(?:\n)?((?s:.+?))```", markdown_prefix)).unwrap();
    codeblock_regex
        .captures_iter(text)
        .map(|cap| cap[1].to_string())
        .collect()
}

fn my_slug(slug: &str, lang: &str, model: &str) -> String {
    format!("{}_{}_{}", slug, lang, model)
}

fn my_slug_json(slug: &str, lang: &str, model: &str) -> String {
    format!("{}.json", my_slug(slug, lang, model))
}

fn parse_my_slug(my_slug: &str) -> (String, String, String) {
    let parts: Vec<&str> = my_slug.split('_').collect();
    let slug = parts.get(0).unwrap_or(&"").to_string();
    let lang = parts.get(1).unwrap_or(&"").to_string();
    let model = match parts.get(2) {
        Some(part) => part.to_string(),
        None => {
            println!("parts: {:?}", parts);
            "".to_string()
        }
    };

    (slug, lang, model)
}

fn parse_my_slug_json(my_slug_json: &str) -> (String, String, String) {
    let parts: Vec<&str> = my_slug_json.split('.').collect();
    parse_my_slug(parts[0])
}

fn get_solution_fns(slug: &str) -> Result<std::fs::ReadDir, std::io::Error> {
    std::fs::read_dir(get_solution_dir(slug))
}

fn get_submission_fns(slug: &str) -> Result<std::fs::ReadDir, std::io::Error> {
    std::fs::read_dir(get_submission_dir(slug))
}

fn get_soln_fn(title_slug: &str, lang: &str, model: &str) -> PathBuf {
    get_solution_dir(title_slug).join(my_slug_json(title_slug, lang, model))
}

async fn save_solution(title_slug: &str, lang: &str, v: &Value) -> std::io::Result<()> {
    let dir_path = get_solution_dir(title_slug);
    tokio::fs::create_dir_all(&dir_path).await?;
    let file_path = dir_path.join(get_soln_fn(title_slug, lang, OPENAI_GPT_MODEL));
    println!("{:?}", file_path);
    tokio::fs::write(file_path, v.to_string()).await
}

async fn save_submission(
    title_slug: &str,
    lang: &str,
    model: &str,
    v: &Value,
) -> std::io::Result<()> {
    let dir_path = get_submission_dir(title_slug);
    tokio::fs::create_dir_all(&dir_path).await?;
    let file_path = dir_path.join(my_slug_json(title_slug, lang, model));
    println!("{:?}", file_path);
    tokio::fs::write(file_path, v.to_string()).await
}

fn build_submission_json(
    title_slug: &str,
    lang: &str,
    model: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let soln_path = get_soln_fn(title_slug, lang, model);
    let soln_text = fs::read_to_string(&soln_path)?;
    let code_blocks = extract_specific_lang_codeblocks(&soln_text, lang);

    let typed_code = code_blocks.last().ok_or("No code blocks found")?;
    let unescaped_typed_code = typed_code.replace("\\n", "\n");

    let question_id = SLUG_ID_MAP
        .get(title_slug)
        .ok_or("Title slug not found in global map")?;

    let json_value = json!({
        "question_id": question_id,
        "lang": lang,
        "typed_code": unescaped_typed_code
    });

    Ok(json_value)
}

pub async fn submit_solution(
    slug: &str,
    lang: &str,
    model: &str,
    post_body: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let referer_url = format!("https://leetcode.com/problems/{}/", slug);

    // Build the headers
    let mut headers = HeaderMap::new();
    headers.insert("X-CSRFToken", HeaderValue::from_static(CSRF_TOKEN));
    headers.insert("Cookie", HeaderValue::from_static(COOKIE));
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    headers.insert(REFERER, HeaderValue::from_str(&referer_url)?);

    // println!("{:#?}", post_body);
    let url = format!("https://leetcode.com/problems/{}/submit/", slug);
    let response = client
        .post(&url)
        .headers(headers)
        .body(post_body.to_string())
        .send()
        .await?;

    let response_text = response.text().await?;
    println!("{}", response_text);
    let json_response = serde_json::from_str(&response_text)?;

    Ok(json_response)
}

pub async fn get_submission_check(id: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut headers = reqwest::header::HeaderMap::new();

    // Set the headers
    headers.insert("authority", HeaderValue::from_static("leetcode.com"));
    headers.insert("accept", HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9"));
    headers.insert(
        "accept-language",
        HeaderValue::from_static("en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7"),
    );
    headers.insert("cookie", HeaderValue::from_str(COOKIE).unwrap());
    headers.insert("dnt", HeaderValue::from_static("1"));
    headers.insert(
        "sec-ch-ua",
        HeaderValue::from_static(
            "\"Not.A/Brand\";v=\"8\", \"Chromium\";v=\"114\", \"Google Chrome\";v=\"114\"",
        ),
    );
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"macOS\""));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
    headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
    headers.insert("upgrade-insecure-requests", HeaderValue::from_static("1"));
    headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36"));

    let client = Client::new();
    let url = format!("https://leetcode.com/submissions/detail/{}/check/", id);

    let res = client.get(&url).headers(headers).send().await?;
    // println!("{:?}", res);
    let response_text = res.text().await?;
    // println!("{}", response_text);
    let json_response = serde_json::from_str(&response_text)?;

    Ok(json_response)
}

fn build_full_prompt(content: &str, _code: &str) -> String {
    format!(
        "{}\n\n{}\n\nWrite out full solution in a markdown codeblock:",
        content, _code
    )
}

/// solve assumes that you've already run `get_problems_and_code`
pub async fn solve(slug: &str, lang: &str, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (content, _code) = build_prompt(&slug, &lang)?;

    let full_prompt = build_full_prompt(&content, &_code);

    let v = fetch_openai_completion(&full_prompt, model).await?;

    save_solution(&slug, &lang, &v).await?;

    Ok(())
}

// #[tokio::main]
async fn old_main() -> Result<(), reqwest::Error> {
    let langs = vec!["java", "cpp"];
    let model = OPENAI_GPT_MODEL;
    // let start_title_slug = "kth-missing-positive-number";
    let start_index = 0;

    let client = reqwest::Client::new();
    let base = "https://leetcode.com/problems/";

    let mut oai_headers = HeaderMap::new();
    oai_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    oai_headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", OPENAI_API_KEY)).unwrap(),
    );

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("cookie", HeaderValue::from_static(COOKIE));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://leetcode.com/problemset/all/"),
    );
    headers.insert("x-csrftoken", HeaderValue::from_static(TOKEN));

    let v: Value =
        serde_json::from_str(&std::fs::read_to_string("problemset.json").unwrap()).unwrap();

    for lang in langs {
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

        // let start_index = free_questions
        //     .iter()
        //     .position(|q| q["titleSlug"].as_str().unwrap() == start_title_slug)
        //     .unwrap();

        let mut times = vec![];
        let mut i = 0;
        /// todo use solve here
        for q in free_questions.iter().skip(start_index).progress() {
            let title_slug = q["titleSlug"].as_str().unwrap();
            // println!("{}: {}", i, title_slug);
            let (content, _code) = build_prompt(&title_slug, &lang).unwrap();
            let full_prompt = format!(
                "{}\n\n{}\n\nWrite out full solution in a markdown codeblock:",
                content, _code
            );
            let v = match fetch_openai_completion(&full_prompt, model).await {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("Failed to fetch OpenAI completion: {} {}", e, title_slug);
                    continue; // or however you want to handle the error
                }
            };
            let local = Local::now();
            times.push(local);
            println!("{}", local.format("%Y-%m-%d %H:%M:%S").to_string());
            save_solution(&title_slug, &lang, &v).await.unwrap();
            i += 1;
        }
    }
    Ok(())
}

fn get_qs() -> Vec<Value> {
    let v: Value =
        serde_json::from_str(&std::fs::read_to_string("problemset.json").unwrap()).unwrap();
    let questions = v["data"]["problemsetQuestionList"]["questions"]
        .as_array()
        .unwrap();
    let qs: Vec<_> = questions
        .iter()
        .filter(|q| match q["paidOnly"].as_bool() {
            Some(paid_only) => !paid_only,
            None => false,
        })
        .cloned() // Cloning the elements here
        .collect();
    qs
}

fn tally_langs(qs: Vec<Value>) -> HashMap<String, usize> {
    let mut langs: HashMap<String, usize> = HashMap::new();
    for q in qs.iter() {
        let code = get_code(&get_title_slug(q)).unwrap();
        let code_snippets = get_code_snippets(&code).unwrap();
        for snippet in code_snippets.as_array().unwrap() {
            let lang = snippet["langSlug"].as_str().unwrap();
            let count = langs.entry(lang.to_string()).or_insert(0);
            *count += 1;
        }
    }
    langs
}

fn tally_files<F>(qs: Vec<Value>, get_fns: F) -> HashMap<String, usize>
where
    F: Fn(&str) -> Result<std::fs::ReadDir, std::io::Error>,
{
    let mut solutions: HashMap<String, usize> = HashMap::new();
    for q in qs.iter() {
        let slug = get_title_slug(q);
        for file in get_fns(&slug).unwrap() {
            let f = file.unwrap();
            let (_slug, lang, _model) = parse_my_slug_json(&f.file_name().to_str().unwrap());
            let count = solutions.entry(lang.to_string()).or_insert(0);
            *count += 1;
        }
    }
    solutions
}

fn tally_solutions(qs: Vec<Value>) -> HashMap<String, usize> {
    tally_files(qs, get_solution_fns)
}

fn tally_submissions(qs: Vec<Value>) -> HashMap<String, usize> {
    tally_files(qs, get_submission_fns)
}

fn get_common_question_slugs(langs: Vec<&str>) -> Vec<String> {
    let all_langs_done: HashSet<String> = langs.iter().map(|&s| s.to_string()).collect();

    let qs = get_qs();

    let mut title_slug_langs_map: HashMap<String, HashSet<String>> = HashMap::new();

    for q in qs.iter() {
        let title_slug = get_title_slug(q).to_string();
        let code = get_code(&title_slug).unwrap();
        let code_snippets = get_code_snippets(&code).unwrap();
        for snippet in code_snippets.as_array().unwrap() {
            let lang = snippet["langSlug"].as_str().unwrap().to_string();
            title_slug_langs_map
                .entry(title_slug.clone())
                .or_insert(HashSet::new())
                .insert(lang);
        }
    }

    title_slug_langs_map
        .into_iter()
        .filter(|(_k, v)| all_langs_done.is_subset(v))
        .map(|(k, _v)| k)
        .collect()
}

fn get_common_questions(langs: Vec<&str>) -> Vec<Value> {
    let common_slugs = get_common_question_slugs(langs);
    let qs = get_qs();

    qs.into_iter()
        .filter(|q| common_slugs.contains(&get_title_slug(q)))
        .collect()
}

fn display_tally(hm: HashMap<String, usize>) {
    let mut v: Vec<_> = hm.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    for (lang, count) in v {
        println!("{}: {}", lang, count);
    }
}

#[tokio::main]
pub async fn main() -> Result<(), reqwest::Error> {
    let langs: Vec<&str> = vec!["c", "cpp", "java", "javascript", "python3", "rust"];
    // todo run gpt on csharp golang php scala kotlin swift ruby dart elixir racket erlang
    // try doing it async to
    let qs = get_common_questions(langs.clone());

    let start_index = qs
        .iter()
        .position(|q| {
            q["titleSlug"].as_str().unwrap() == "random-point-in-non-overlapping-rectangles"
        })
        .unwrap_or(0);
    println!("Start index: {:#?}", start_index);

    println!("Start question: {:#?}", qs[start_index]);
    println!("tally_langs:");
    display_tally(tally_langs(qs.clone()));
    println!("\ntally_solutions:");
    display_tally(tally_solutions(qs.clone()));
    println!("\ntally_submissions:");
    display_tally(tally_submissions(qs.clone()));
    println!("\nQuestions in testset: {:#?}", qs.len());

    let model = OPENAI_GPT_MODEL;

    for q in qs.iter().skip(start_index).progress() {
        for lang in langs.clone() {
            let title_slug = q["titleSlug"].as_str().unwrap();
            let soln_fn = get_soln_fn(title_slug, lang, model);
            println!("{:?}", soln_fn);

            let sub_path = get_submission_dir(title_slug);

            match build_submission_json(title_slug, lang, model) {
                Ok(post_body) => match submit_solution(title_slug, lang, model, post_body).await {
                    Ok(v) => {
                        println!("{:?}", v);
                        let id = v["submission_id"].as_i64().unwrap();
                        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
                        let check = match get_submission_check(&id.to_string()).await {
                            Ok(check) => check,
                            Err(e) => {
                                println!("Error getting submission check: {}", e);
                                continue;
                            }
                        };
                        println!("{:#?}", check);
                        match save_submission(title_slug, lang, model, &check).await {
                            Ok(_) => (),
                            Err(e) => {
                                println!("Error saving submission: {}", e);
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error submitting solution: {}", e);
                        continue;
                    }
                },
                Err(e) => {
                    println!("Error building submission JSON: {}", e);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_specific_lang_codeblocks() {
        let text = "
```python
print('Hello, World!')
```
";
        let lang = "python3";
        let code_blocks = extract_specific_lang_codeblocks(text, lang);
        assert_eq!(code_blocks[0], "print('Hello, World!')\n");
    }
}
