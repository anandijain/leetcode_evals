use indicatif::{self, ProgressBar, ProgressIterator, ProgressStyle};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, REFERER, USER_AGENT};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use thirtyfour::prelude::WebDriverError;

use chrono::{prelude::*, Duration};
use cookie::Cookie;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::fs::create_dir_all;
use std::fs::{write, File};
use std::io::Write;
use std::thread::sleep;
use thirtyfour::prelude::*;
use tokio::task::JoinHandle;
use tokio::try_join;

#[macro_use]
extern crate lazy_static;

const OPENAI_API_KEY: &str = env!("OPENAI_API_KEY");
// const OPENAI_API_KEY: &str = &std::env::var("OPENAI_API_KEY").unwrap();
const OPENAI_GPT_MODEL: &str = "gpt-3.5-turbo";
// const OPENAI_GPT_MODEL: &str = "gpt-4";

const COOKIES: &[&str] = &[
    r#"csrftoken=5180mCETfZYeuAf66WuSs7mmEAG48arQ5D5lZaCzKPjt1nmG5saHrXZmkhGaaLRI; messages="355ba39e0f8febeea9925b7911e2e001fb4be2bf$[[\"__json_message\"\0540\05425\054\"Successfully signed in as anandjain.\"]\054[\"__json_message\"\0540\05425\054\"Successfully signed in as anandjain.\"]\054[\"__json_message\"\0540\05425\054\"You have signed out.\"]\054[\"__json_message\"\0540\05420\054\"Confirmation e-mail sent to thisisagag@gmail.com.\"]\054[\"__json_message\"\0540\05425\054\"Successfully signed in as anandijain2.\"]\054[\"__json_message\"\0540\05425\054\"You have confirmed thisisagag@gmail.com.\"]\054[\"__json_message\"\0540\05425\054\"You have signed out.\"]\054[\"__json_message\"\0540\05420\054\"Confirmation e-mail sent to anandash2@gmail.com.\"]\054[\"__json_message\"\0540\05425\054\"Successfully signed in as anandijain3.\"]\054[\"__json_message\"\0540\05425\054\"You have confirmed anandash2@gmail.com.\"]\054[\"__json_message\"\0540\05425\054\"You have signed out.\"]\054[\"__json_message\"\0540\05420\054\"Confirmation e-mail sent to oneofmanymagnumopai@gmail.com.\"]\054[\"__json_message\"\0540\05425\054\"Successfully signed in as anandijain4.\"]\054[\"__json_message\"\0540\05425\054\"You have signed out.\"]\054[\"__json_message\"\0540\05420\054\"Confirmation e-mail sent to radiator.runoff@gmail.com.\"]\054[\"__json_message\"\0540\05425\054\"Successfully signed in as anandijain5.\"]\054[\"__json_message\"\0540\05425\054\"You have confirmed radiator.runoff@gmail.com.\"]\054[\"__json_message\"\0540\05425\054\"You have signed out.\"]\054[\"__json_message\"\0540\05425\054\"Successfully signed in as anandijain2.\"]\054[\"__json_message\"\0540\05425\054\"You have signed out.\"]\054[\"__json_message\"\0540\05425\054\"Successfully signed in as anandjain.\"]\054[\"__json_message\"\0540\05425\054\"You have signed out.\"]\054[\"__json_message\"\0540\05425\054\"Successfully signed in as anandjain.\"]\054[\"__json_message\"\0540\05425\054\"You have signed out.\"]\054[\"__json_message\"\0540\05425\054\"Successfully signed in as anandjain.\"]]"; LEETCODE_SESSION=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJfYXV0aF91c2VyX2lkIjoiMzYwNTc2OCIsIl9hdXRoX3VzZXJfYmFja2VuZCI6ImRqYW5nby5jb250cmliLmF1dGguYmFja2VuZHMuTW9kZWxCYWNrZW5kIiwiX2F1dGhfdXNlcl9oYXNoIjoiMjEwNWIxYWQ3ZjNhOWNhMmZiMzc1N2FhNzEzZjQ4MmFiNTMxOWVmNyIsImlkIjozNjA1NzY4LCJlbWFpbCI6ImRhaG1laC5hbG1vc0BnbWFpbC5jb20iLCJ1c2VybmFtZSI6ImFuYW5kamFpbiIsInVzZXJfc2x1ZyI6ImFuYW5kamFpbiIsImF2YXRhciI6Imh0dHBzOi8vYXNzZXRzLmxlZXRjb2RlLmNvbS91c2Vycy9hdmF0YXJzL2F2YXRhcl8xNjc4OTA5MTg5LnBuZyIsInJlZnJlc2hlZF9hdCI6MTY5MDIyNjg1NSwiaXAiOiI2Ni4zMC4yMjMuOSIsImlkZW50aXR5IjoiNWYwZmY1ZDg3OTllZDRjMGVkMzU1ZmE0NzRhN2JiYzIiLCJzZXNzaW9uX2lkIjo0MzEyMzM0NywiX3Nlc3Npb25fZXhwaXJ5IjoxMjA5NjAwfQ.4ZsVg7zUn-fR-PqC81AsJDRnjaKo9jElMD8E9nHS4f0; _dd_s=rum=0&expire=1690292592248"#,
    r#"csrftoken=OVyYaFct7KtLGrwJ24B9R6SNxveAdLAW3eIUzvhGe4sYZnYCSEu4zZbXKpxxfuHt; messages="40f1b87013c4898ef73e5b0025fdc1d3bd31d22a$[[\"__json_message\"\0540\05425\054\"Successfully signed in as anandijain2.\"]]"; LEETCODE_SESSION=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJfYXV0aF91c2VyX2lkIjoiMTAxMzgwOTMiLCJfYXV0aF91c2VyX2JhY2tlbmQiOiJkamFuZ28uY29udHJpYi5hdXRoLmJhY2tlbmRzLk1vZGVsQmFja2VuZCIsIl9hdXRoX3VzZXJfaGFzaCI6IjY1OTc3NGI0MjBlNmNhNTRkZjMyMzEzZDhlN2U5NjI1YTAyN2I2OTkiLCJpZCI6MTAxMzgwOTMsImVtYWlsIjoidGhpc2lzYWdhZ0BnbWFpbC5jb20iLCJ1c2VybmFtZSI6ImFuYW5kaWphaW4yIiwidXNlcl9zbHVnIjoiYW5hbmRpamFpbjIiLCJhdmF0YXIiOiJodHRwczovL3MzLXVzLXdlc3QtMS5hbWF6b25hd3MuY29tL3MzLWxjLXVwbG9hZC9hc3NldHMvZGVmYXVsdF9hdmF0YXIuanBnIiwicmVmcmVzaGVkX2F0IjoxNjkwNjYwOTE4LCJpcCI6IjY2LjMwLjIyMy45IiwiaWRlbnRpdHkiOiI1ZjBmZjVkODc5OWVkNGMwZWQzNTVmYTQ3NGE3YmJjMiIsInNlc3Npb25faWQiOjQzNDAxNjUwLCJfc2Vzc2lvbl9leHBpcnkiOjEyMDk2MDB9.Me5eiuNOTMLRcKYFfwZt9Z4-Wv77SS-aAJQnjWQzrjI; NEW_PROBLEMLIST_PAGE=1; _dd_s=rum=0&expire=1690661836912"#,
    r#"csrftoken=ulxpAvT3HarrS2exBseycxTSeojAkDgjeiReDxa4d4dv0igFvuKSp3KBeavVjYae; _ga_CDRWKZTDEX=GS1.1.1690651758.1.1.1690651783.35.0.0; _ga=GA1.1.1428573774.1690651758; _dd_s=rum=0&expire=1690652689360; messages="933207930920dd9ca86f3ebb068c06d4926043f4$[[\"__json_message\"\0540\05425\054\"Successfully signed in as anandijain3.\"]]"; LEETCODE_SESSION=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJfYXV0aF91c2VyX2lkIjoiMTAxMzgyMzQiLCJfYXV0aF91c2VyX2JhY2tlbmQiOiJkamFuZ28uY29udHJpYi5hdXRoLmJhY2tlbmRzLk1vZGVsQmFja2VuZCIsIl9…cy5jb20vczMtbGMtdXBsb2FkL2Fzc2V0cy9kZWZhdWx0X2F2YXRhci5qcGciLCJyZWZyZXNoZWRfYXQiOjE2OTA2NTE3NzIsImlwIjoiNjYuMzAuMjIzLjkiLCJpZGVudGl0eSI6IjM4YTE5ZGZlNWJjNWMxMWU5YmQ5YjkwNTQ0ZWZmOGE5Iiwic2Vzc2lvbl9pZCI6NDMzOTU5MzMsIl9zZXNzaW9uX2V4cGlyeSI6MTIwOTYwMH0.4GsK59MHTFITZAMpE8CJO8ohhPLJA7c4AqGGqnpVKNo; NEW_PROBLEMLIST_PAGE=1; gr_user_id=aec70933-3adb-4fcb-b50d-ed3818a46433; 87b5a3c3f1a55520_gr_session_id=6678de7d-96fb-4225-b1be-9f8a3725a4c7; 87b5a3c3f1a55520_gr_session_id_sent_vst=6678de7d-96fb-4225-b1be-9f8a3725a4c7"#,
    r#"csrftoken=Oqahqx51re9HOvQY7KL5YNxhHFTuWrAFcmNQRuOaYxBpE6WmTecLppVQla4jPmyQ; _ga_CDRWKZTDEX=GS1.1.1690651835.1.1.1690652065.6.0.0; _dd_s=rum=0&expire=1690652965722; 87b5a3c3f1a55520_gr_session_id=c6b4daec-3577-4c9b-b250-d08d1094bf36; 87b5a3c3f1a55520_gr_session_id_sent_vst=c6b4daec-3577-4c9b-b250-d08d1094bf36; _ga=GA1.1.994671939.1690651835; _gid=GA1.2.1983051490.1690651835; gr_user_id=1ac9ed3f-509a-4d3a-9935-3c1d17291d8f; _gat=1; NEW_PROBLEMLIST_PAGE=1; LEETCODE_SESSION=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJfYXV0aF91c2VyX2lkIjoiMTAxMzgyNDUiLCJfYXV0aF91c2VyX2JhY2tlbmQiOiJkamFuZ28uY29udHJpYi5hdXRoLmJhY2tlbmRzLk1vZGVsQmFja2VuZCIsIl9hdXRoX3VzZXJfaGFzaCI6IjQ3NTBlYzQ2ODIwYmY5YzU4NDYzZTFjMmE1MGZjYzA3MTViNGM2ZWIiLCJpZCI6MTAxMzgyNDUsImVtYWlsIjoib25lb2ZtYW55bWFnbnVtb3BhaUBnbWFpbC5jb20iLCJ1c2VybmFtZSI6ImFuYW5kaWphaW40IiwidXNlcl9zbHVnIjoiYW5hbmRpamFpbjQiLCJhdmF0YXIiOiJodHRwczovL3MzLXVzLXdlc3QtMS5hbWF6b25hd3MuY29tL3MzLWxjLXVwbG9hZC9hc3NldHMvZGVmYXVsdF9hdmF0YXIuanBnIiwicmVmcmVzaGVkX2F0IjoxNjkwNjUxOTA0LCJpcCI6IjY2LjMwLjIyMy45IiwiaWRlbnRpdHkiOiJmMjczZGFkMTI4OWI3YmZkMWE5YmU2Mzc2ODEzYjkyMiIsInNlc3Npb25faWQiOjQzMzk2MDM5LCJfc2Vzc2lvbl9leHBpcnkiOjEyMDk2MDB9.dE59RL0s1UYma1BqV273wyDVCt7JnaSaNMqHB375_ck; messages="bb0c0d208dc7c09f59ec1e3384ed885072da2eff$[[\"__json_message\"\0540\05425\054\"Successfully signed in as anandijain4.\"]]""#,
    // r#"csrftoken=iR0qA1m253k02nHhf1sF59kEZYfkLVtFfNMHhJTd3akHwpQnwiYJEixhmNHrbeDe; messages="e3c5c17025394d2347b1038aa9b84747c33faa0b$[[\"__json_message\"\0540\05425\054\"Successfully signed in as anandijain5.\"]]"; LEETCODE_SESSION=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJfYXV0aF91c2VyX2lkIjoiMTAxMzgyNjciLCJfYXV0aF91c2VyX2JhY2tlbmQiOiJkamFuZ28uY29udHJpYi5hdXRoLmJhY2tlbmRzLk1vZGVsQmFja2VuZCIsIl9hdXRoX3VzZXJfaGFzaCI6IjI4YWE2OWNjY2ExMWNjZmRiZDg1Y2QwZTcxNzJmNDJmZWQ3ZDM3N2MiLCJpZCI6MTAxMzgyNjcsImVtYWlsIjoicmFkaWF0b3IucnVub2ZmQGdtYWlsLmNvbSIsInVzZXJuYW1lIjoiYW5hbmRpamFpbjUiLCJ1c2VyX3NsdWciOiJhbmFuZGlqYWluNSIsImF2YXRhciI6Imh0dHBzOi8vczMtdXMtd2VzdC0xLmFtYXpvbmF3cy5jb20vczMtbGMtdXBsb2FkL2Fzc2V0cy9kZWZhdWx0X2F2YXRhci5qcGciLCJyZWZyZXNoZWRfYXQiOjE2ODk3MjIyMjgsImlwIjoiNjYuMzAuMjIzLjkiLCJpZGVudGl0eSI6IjVmMGZmNWQ4Nzk5ZWQ0YzBlZDM1NWZhNDc0YTdiYmMyIiwic2Vzc2lvbl9pZCI6NDI4MDU1ODUsIl9zZXNzaW9uX2V4cGlyeSI6MTIwOTYwMH0.Ee6x4W0ydvTANrVhnZvrjkdqIpKJiq-OBz62-1MFV9Y; NEW_PROBLEMLIST_PAGE=1; _dd_s=rum=0&expire=1689723164062"#,
];

const COOKIE: &str = COOKIES[0];
const USER_AGENT_STR: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36";

// const BASE_PATH: &str = "/Users/anand/.rust/dev/leetcode_evals/data/data/";
const BASE_PATH: &str = "./data/data/";

const RATE_LIMIT_STR: &str = "Error 429 - Rate limit exceeded!";

// ignoring SQL and other bullshit
const ALL_REAL_LANGS: &[&str] = &[
    "javascript",
    "typescript",
    "python",
    "python3",
    "java",
    "cpp",
    "csharp",
    "c",
    "golang",
    "kotlin",
    "scala",
    "ruby",
    "php",
    "swift",
    "rust",
    "dart",
    "elixir",
    "racket",
    "erlang",
];

static SLUG_ID_MAP: Lazy<HashMap<String, String>> = Lazy::new(|| build_slug_id_map());

lazy_static! {
    static ref MARKDOWN_PREFIXES: HashMap<&'static str, &'static str> = {
        vec![
            ("golang", "go"),
            ("python3", "python"),
            ("python", "python"),
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
            ("elixir", "elixir"),
            ("racket", "racket"),
            ("erlang", "erlang"),
        ]
        .into_iter()
        .collect()
    };
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionsRoot {
    pub data: QuestionsData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionsData {
    pub problemset_question_list: ProblemsetQuestionList,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemsetQuestionList {
    pub total: i64,
    pub questions: Vec<Question>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub ac_rate: f64,
    pub difficulty: String,
    pub freq_bar: Value,
    pub frontend_question_id: String,
    pub is_favor: bool,
    pub paid_only: bool,
    pub status: Value,
    pub title: String,
    pub title_slug: String,
    pub topic_tags: Vec<TopicTag>,
    pub has_solution: bool,
    pub has_video_solution: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicTag {
    pub name: String,
    pub id: String,
    pub slug: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptRoot {
    pub data: PromptData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptData {
    pub question: Prompt,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    pub content: String,
    pub mysql_schemas: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeRoot {
    pub data: CodeData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeData {
    pub question: CodeQuestion,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeQuestion {
    pub question_id: String,
    pub question_frontend_id: String,
    pub code_snippets: Vec<CodeSnippet>,
    pub env_info: String,
    pub enable_run_code: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSnippet {
    pub lang: String,
    pub lang_slug: String,
    pub code: String,
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
    headers.insert(
        "x-csrftoken",
        HeaderValue::from_str(&csrftoken_from_cookie_str(COOKIE)).unwrap(),
    );

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

fn build_slug_id_map() -> HashMap<String, String> {
    let qs = get_qs();
    let mut slug_to_id = HashMap::new();

    for q in qs {
        let slug = q.title_slug;
        if let Ok(code) = get_code(&slug) {
            let qid = code.data.question.question_id;
            slug_to_id.insert(slug.clone(), qid.clone());
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
    headers.insert(
        "x-csrftoken",
        HeaderValue::from_str(&csrftoken_from_cookie_str(COOKIE)).unwrap(),
    );

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
        let url = format!("{}{}", base, slug);

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

        let data_path = format!("{}/{}/prompt/", BASE_PATH, slug);

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
    Path::new(BASE_PATH).join(title_slug)
}

// Function to get the directory path
fn get_prompt_dir(title_slug: &str) -> PathBuf {
    Path::new(BASE_PATH).join(title_slug).join("prompt")
}

// Function to get the directory path
fn get_solution_dir(title_slug: &str) -> PathBuf {
    Path::new(BASE_PATH).join(title_slug).join("solutions")
}

// Function to get the directory path
fn get_submission_dir(title_slug: &str) -> PathBuf {
    Path::new(BASE_PATH).join(title_slug).join("submissions")
}

// Function to get the prompt path
fn get_prompt_path(title_slug: &str) -> PathBuf {
    let dir_path = get_prompt_dir(title_slug);
    let f = dir_path.join(format!("{}_prompt.json", title_slug));
    f
}

// fn get_title_slug(question: &Question) -> String {
//     question.title_slug
// }

// Function to get the code path
fn get_code_path(title_slug: &str) -> PathBuf {
    let dir_path = get_prompt_dir(title_slug);
    dir_path.join(format!("{}_code.json", title_slug))
}

fn read_json<T: DeserializeOwned, P: AsRef<Path>>(file_path: P) -> Result<T, serde_json::Error> {
    let file_content = fs::read_to_string(file_path).unwrap();
    serde_json::from_str(&file_content)
}

fn get_prompt(title_slug: &str) -> Result<PromptRoot, serde_json::Error> {
    let prompt_path = get_prompt_path(title_slug);
    read_json(prompt_path)
}

fn get_code(title_slug: &str) -> Result<CodeRoot, serde_json::Error> {
    let code_path = get_code_path(title_slug);
    read_json(code_path)
}

// fn get_content_from_json(prompt_json: &PromptRoot) -> Result<String, Box<dyn std::error::Error>> {
//     Ok(prompt_json.data.question.content)
// }


// fn get_code_snippets(data: &CodeRoot) -> Vec<CodeSnippet> {
//     // Navigate to the 'codeSnippets' field.
//     data.data.question.code_snippets
//     // data.get("data")?.get("question")?.get("codeSnippets")
// }

fn get_code_for_lang(code_snippets: &Vec<CodeSnippet>, lang: &str) -> Result<String, Box<dyn Error>> {
    code_snippets
        .iter()
        .find(|&s| s.lang_slug == lang)
        .map(|s| s.code.clone())
        .ok_or_else(|| "No matching snippet for the specified language".into())
}

fn has_lang(code_snippets: &Vec<CodeSnippet>, lang: &str) -> bool {
    get_code_for_lang(code_snippets, lang).is_ok()
}

fn build_prompt(title_slug: &str, lang: &str) -> Result<(String, String), Box<dyn Error>> {
    // Read the prompt and get the content
    let prompt_json = get_prompt(title_slug)?;
    let content = prompt_json.data.question.content;

    // Read the code JSON and get the code snippet for the specified language
    let code_json = get_code(title_slug)?;
    let code_snippets = code_json.data.question.code_snippets;
    let code = get_code_for_lang(&code_snippets, lang)?;

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

fn get_solution_fn(title_slug: &str, lang: &str, model: &str) -> PathBuf {
    get_solution_dir(title_slug).join(my_slug_json(title_slug, lang, model))
}

fn get_solution_json(title_slug: &str, lang: &str, model: &str) -> Result<Value, Box<dyn Error>> {
    Ok(read_json(get_solution_fn(title_slug, lang, model))?)
}

fn get_submission_fns(slug: &str) -> Result<std::fs::ReadDir, std::io::Error> {
    std::fs::read_dir(get_submission_dir(slug))
}

fn get_submission_fn(title_slug: &str, lang: &str, model: &str) -> PathBuf {
    get_submission_dir(title_slug).join(my_slug_json(title_slug, lang, model))
}

fn get_submission_json(title_slug: &str, lang: &str, model: &str) -> Result<Value, Box<dyn Error>> {
    Ok(read_json(get_submission_fn(title_slug, lang, model))?)
}

async fn save_solution(title_slug: &str, lang: &str, v: &Value) -> std::io::Result<()> {
    let dir_path = get_solution_dir(title_slug);
    tokio::fs::create_dir_all(&dir_path).await?;
    let file_path = dir_path.join(get_solution_fn(title_slug, lang, OPENAI_GPT_MODEL));
    println!("{:?}", file_path);
    tokio::fs::write(file_path, format!("{:#}", v)).await
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
    tokio::fs::write(file_path, format!("{:#}", v)).await
}

fn get_submission_code(
    title_slug: &str,
    lang: &str,
    model: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let soln_path = get_solution_fn(title_slug, lang, model);
    let soln_text = fs::read_to_string(&soln_path)?;
    let code_blocks = extract_specific_lang_codeblocks(&soln_text, lang);

    let typed_code = code_blocks
        .last()
        .ok_or("No code blocks found")?
        .to_string();

    Ok(typed_code)
}

fn build_submission_json(
    title_slug: &str,
    lang: &str,
    model: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let typed_code = get_submission_code(title_slug, lang, model)?;
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
    cookie: &str,
) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();

    let referer_url = format!("https://leetcode.com/problems/{}/", slug);

    // Build the headers
    let mut headers = HeaderMap::new();
    headers.insert("Cookie", HeaderValue::from_str(cookie).unwrap());

    headers.insert(
        "X-CSRFToken",
        HeaderValue::from_str(&csrftoken_from_cookie_str(cookie)).unwrap(),
    );
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    headers.insert(REFERER, HeaderValue::from_str(&referer_url).unwrap());

    // println!("{:#?}", post_body);
    let url = format!("https://leetcode.com/problems/{}/submit/", slug);
    let response = client
        .post(&url)
        .headers(headers)
        .body(post_body.to_string())
        .send()
        .await?;

    response.text().await
}

pub async fn get_submission_check(
    id: &str,
    cookie: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut headers = reqwest::header::HeaderMap::new();

    // Set the headers
    headers.insert("authority", HeaderValue::from_static("leetcode.com"));
    headers.insert("accept", HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9"));
    headers.insert(
        "accept-language",
        HeaderValue::from_static("en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7"),
    );
    headers.insert("cookie", HeaderValue::from_str(cookie).unwrap());
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
    headers.insert("user-agent", HeaderValue::from_static(USER_AGENT_STR));

    let client = Client::new();
    let url = format!("https://leetcode.com/submissions/detail/{}/check/", id);

    let res = client.get(&url).headers(headers).send().await?;
    // println!("{:?}", res);
    let response_text = res.text().await?;
    // println!("{}", response_text);
    let json_response = serde_json::from_str(&response_text)?;

    Ok(json_response)
}

fn format_full_prompt(content: &str, _code: &str) -> String {
    format!(
        "{}\n\n{}\n\nWrite out full solution in a markdown codeblock:",
        content, _code
    )
}

fn build_full_prompt(slug: &str, lang: &str) -> Result<String, Box<dyn std::error::Error>> {
    let (content, _code) = build_prompt(&slug, &lang)?;
    Ok(format_full_prompt(&content, &_code))
}

fn build_oai_post_body(
    slug: &str,
    lang: &str,
    model: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let data = json!({
        "model": model,
        "messages": [
            {"role": "user", "content": build_full_prompt(slug, lang)?}
        ]
    });
    Ok(data)
}

fn build_oai_pair(slug: &str, lang: &str, model: &str) -> (PathBuf, Value) {
    (
        get_solution_fn(slug, lang, model),
        build_oai_post_body(slug, lang, model).unwrap(),
    )
}

fn get_questions_for_lang(qs: &Vec<Question>, lang: &str) -> Vec<Question> {
    let mut lqs = vec![];
    for q in qs {
        let code = get_code(&q.title_slug).unwrap();
        let code_snippets = code.data.question.code_snippets;
        if has_lang(&code_snippets, lang) {
            lqs.push(q.clone());
        }
    }
    lqs
}

fn build_all_mytups(
    qs: Vec<Question>,
    langs: Vec<&str>,
    models: Vec<&str>,
) -> Vec<(String, String, String)> {
    let mut bodies = vec![];
    for m in models.clone() {
        for l in langs.clone() {
            let lqs = get_questions_for_lang(&qs, l);
            for q in lqs.clone() {
                let slug = q.title_slug;
                bodies.push((slug, l.to_string(), m.to_string()));
            }
        }
    }
    bodies
}

// fn build_all_mytups_noclone(
//     qs: Vec<Question>,
//     langs: Vec<&str>,
//     models: Vec<&str>,
// ) -> Vec<(String, String, String)> {
//     let mut bodies = vec![];
//     for m in &models {
//         for l in &langs {
//             let lqs = get_questions_for_lang(&qs, l);
//             for q in &lqs {
//                 let slug = q.title_slug;
//                 bodies.push((slug, l.to_string(), m.to_string()));
//             }
//         }
//     }
//     bodies
// }

/// solve assumes that you've already run `get_problems_and_code`
pub async fn solve(
    slug: &str,
    lang: &str,
    model: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let v = fetch_openai_completion(&build_full_prompt(slug, lang)?, model).await?;

    // save_solution(&slug, &lang, &v).await?;

    Ok(v)
}

async fn submit(
    slug: &str,
    lang: &str,
    model: &str,
    cookie: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let post_body = build_submission_json(slug, lang, model)?;
    println!("{:#?}", post_body);
    let response_text = submit_solution(slug, lang, model, post_body, cookie).await?;
    Ok(response_text)
}

fn get_qs() -> Vec<Question> {
    let v: QuestionsRoot =
        serde_json::from_str(&std::fs::read_to_string("problemset.json").unwrap()).unwrap();
    let questions = v.data.problemset_question_list.questions;
    let qs = questions.into_iter().filter(|q| !q.paid_only).collect();
    qs
}

fn tally_langs(qs: Vec<Question>) -> HashMap<String, usize> {
    let mut langs: HashMap<String, usize> = HashMap::new();
    for q in qs.iter() {
        let code = get_code(&q.title_slug).unwrap();
        let code_snippets = code.data.question.code_snippets;
        for snippet in code_snippets {
            let lang = snippet.lang_slug;
            let count = langs.entry(lang.to_string()).or_insert(0);
            *count += 1;
        }
    }
    langs
}

fn tally_files<F>(qs: Vec<Question>, get_fns: F) -> HashMap<String, usize>
where
    F: Fn(&str) -> Result<std::fs::ReadDir, std::io::Error>,
{
    let mut solutions: HashMap<String, usize> = HashMap::new();
    for q in qs.iter() {
        let slug = &q.title_slug;
        // println!("slug: {}", slug);
        match get_fns(&slug) {
            Ok(files) => {
                for file in files {
                    let f = file.unwrap(); // I kept this unwrap, assuming the files are guaranteed to exist. If not, you may want to add error handling here too.
                    let (_slug, lang, _model) =
                        parse_my_slug_json(&f.file_name().to_str().unwrap());
                    let count = solutions.entry(lang.to_string()).or_insert(0);
                    *count += 1;
                }
            }
            Err(e) => {
                // eprintln!("Error while trying to read directory of '{}': {}", slug, e);
                continue;
            }
        }
    }
    solutions
}

fn tally_solutions(qs: Vec<Question>) -> HashMap<String, usize> {
    tally_files(qs, get_solution_fns)
}

fn tally_submissions(qs: Vec<Question>) -> HashMap<String, usize> {
    tally_files(qs, get_submission_fns)
}

fn get_common_question_slugs(langs: Vec<&str>) -> Vec<String> {
    let all_langs_done: HashSet<String> = langs.iter().map(|&s| s.to_string()).collect();

    let qs = get_qs();

    let mut title_slug_langs_map: HashMap<String, HashSet<String>> = HashMap::new();

    for q in qs.iter() {
        let title_slug = q.title_slug.to_string();
        let code = get_code(&title_slug).unwrap();
        let code_snippets = code.data.question.code_snippets;
        for snippet in code_snippets {
            let lang = snippet.lang_slug;
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

fn get_common_questions(langs: Vec<&str>) -> Vec<Question> {
    let common_slugs = get_common_question_slugs(langs);
    let qs = get_qs();

    qs.into_iter()
        .filter(|q| common_slugs.contains(&q.title_slug))
        .collect()
}

fn display_tally<T>(hm: &HashMap<String, T>)
where
    T: Ord + std::fmt::Display,
{
    let mut v: Vec<_> = hm.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1));
    for (lang, count) in v {
        println!("{}: {}", lang, count);
    }
}

fn build_cookie_map(cookie_str: &str) -> HashMap<String, String> {
    let mut cookie_map: HashMap<String, String> = HashMap::new();
    for cookie in Cookie::split_parse(cookie_str) {
        let cookie = cookie.unwrap();
        cookie_map.insert(cookie.name().to_string(), cookie.value().to_string());
    }
    cookie_map
}

fn csrftoken_from_cookie_str(cookie_str: &str) -> String {
    let cookie_map = build_cookie_map(cookie_str);
    cookie_map.get("csrftoken").unwrap().to_string()
}

fn tally_statuses() -> () {
    let model = OPENAI_GPT_MODEL;
    let models = vec![model];
    let qs = get_qs();

    let myslug_tups: Vec<(String, String, String)> =
        build_all_mytups(qs.clone(), ALL_REAL_LANGS.to_vec().clone(), models.clone());

    let lang_tally = tally_langs(qs.clone());
    let sol_tally = tally_solutions(qs.clone());
    let sub_tally = tally_submissions(qs.clone());

    println!("tally_langs:");
    display_tally(&lang_tally);
    println!("\ntally_solutions:");
    display_tally(&sol_tally);
    println!("\ntally_submissions:");
    display_tally(&sub_tally);
    println!("\nQuestions in testset: {:#?}", qs.len());

    println!("\nDifference between tally_langs and tally_solutions:");
    let mut diff_map: HashMap<String, isize> = HashMap::new();
    for (lang, &count) in lang_tally.iter() {
        let sol_count = sol_tally.get(lang).unwrap_or(&0);
        let diff = count as isize - *sol_count as isize;
        diff_map.insert(lang.clone(), diff);
    }
    display_tally(&diff_map);
    ()
}

//

async fn submit_and_check(
    slug: &str,
    lang: &str,
    model: &str,
    cookie: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    if let Ok(sub) = submit(slug, lang, model, cookie).await {
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
        println!("{:#?}", sub);

        if let Ok(json_response) = serde_json::from_str::<Value>(&sub) {
            if let Some(id) = json_response["submission_id"].as_i64() {
                let check = match get_submission_check(&id.to_string(), cookie).await {
                    Ok(check) => check,
                    Err(e) => {
                        return Err(format!("Error getting submission check id: {}", e).into());
                    }
                };
                println!("{:#?}", check);

                match save_submission(&slug, &lang, model, &check).await {
                    Ok(_) => {
                        let local = Local::now();
                        println!("{}", local.format("%Y-%m-%d %H:%M:%S").to_string());
                        println!("{:#?}", get_submission_fn(slug, lang, model));
                    }
                    Err(e) => {
                        return Err(format!("Error saving submission: {}", e).into());
                    }
                }
                return Ok(check);
            }
        } else {
            if sub.contains(RATE_LIMIT_STR) {
                panic!("Rate limit exceeded");
            }
        }
    }
    Err("No submission was made".into())
}

async fn solve_all() -> () {
    let mut i = 0usize;
    for (slug, lang, model) in
        build_all_mytups(get_qs(), ALL_REAL_LANGS.to_vec(), vec![OPENAI_GPT_MODEL])
            .iter()
            .progress()
    {
        if !get_solution_fn(&slug, &lang, &model).exists() {
            i += 1;
            match solve(&slug, &lang, &model).await {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error occurred in solve: {}", e);
                    continue;
                }
            };
            let local = Local::now();
            println!("{}: {}", i, local.format("%Y-%m-%d %H:%M:%S").to_string());
        }
    }
    ()
}

async fn submit_all_solutions(
    myslug_tups: Vec<(String, String, String)>,
) -> Result<(), reqwest::Error> {
    let mut cookie_idx = 0;
    // let df = get_creds_df().await.unwrap();
    // let mut cookie = get_cookie_string_from_index(&df, cookie_idx).await.unwrap();
    let mut cookie = COOKIES[cookie_idx];
    for (slug, lang, model) in myslug_tups.iter().progress() {
        let sub_fn = get_submission_fn(slug, lang, model);
        if !sub_fn.exists() {
            if let Ok(sub) = submit(&slug, &lang, &model, cookie).await {
                tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
                println!("{:#?}", sub);
                if let Ok(json_response) = serde_json::from_str::<Value>(&sub) {
                    if let Some(id) = json_response["submission_id"].as_i64() {
                        let check = match get_submission_check(&id.to_string(), cookie).await {
                            Ok(check) => check,
                            Err(e) => {
                                panic!("Error getting submission check id: {}", e);
                                // continue;
                            }
                        };
                        println!("{:#?}", check);
                        match save_submission(&slug, &lang, model, &check).await {
                            Ok(_) => {
                                let local = Local::now();
                                println!("{}", local.format("%Y-%m-%d %H:%M:%S").to_string());
                                println!("{:#?}", sub_fn);
                            }
                            Err(e) => {
                                println!("Error saving submission: {}", e);
                                // continue;
                            }
                        }
                    } else {
                        if json_response["error"].as_str() == Some("User is not authenticated") {
                            panic!("YO WTF: User is not authenticated: {json_response}");
                        }
                    }
                } else {
                    if sub.contains(RATE_LIMIT_STR) {
                        cookie_idx += 1;
                        cookie = COOKIES[cookie_idx];
                        //     get_cookie_string_from_index(&df, cookie_idx).await.unwrap();

                        // panic!("Rate limit exceeded. TODO add swap cookie ");
                        println!("Rate limit exceeded. New cookie_idx: {}", cookie_idx);
                    }
                }
            }
        }
    }

    Ok(())
}

async fn get_cookie_string(username: &str, password: &str) -> Result<String, WebDriverError> {
    let driver = WebDriver::new("http://localhost:9515", DesiredCapabilities::chrome()).await?;

    // Navigate to URL.
    driver.goto("https://leetcode.com/accounts/login/").await?;
    sleep(std::time::Duration::from_secs(1));
    // Find element.
    let username_field = driver.find(By::Name("login")).await?;
    let password_field = driver.find(By::Name("password")).await?;

    // Type into the element.
    username_field.send_keys(username).await?;
    password_field.send_keys(password).await?;
    // password_field.click(Key::Enter).await?;
    let signin_btn = driver.find(By::Id("signin_btn")).await?;
    sleep(std::time::Duration::from_secs(1));
    signin_btn.click().await?;
    driver.goto("https://leetcode.com/anandjain/").await?;

    // Get cookies
    let cookies = driver.get_all_cookies().await?;
    let mut cookie_string = String::new();
    for cookie in cookies {
        cookie_string.push_str(&format!("{}={}; ", cookie.name(), cookie.value()));
    }
    println!("COOKIE for {} {}: {}", username, password, cookie_string);

    // Quit driver.
    driver.quit().await?;

    Ok(cookie_string)
}

#[tokio::main]
pub async fn main() -> Result<(), reqwest::Error> {
    // step one is remove all the references to COOKIE and make it an arg (done )
    // 2) is see if i need that "LEETCODE_SESSION" part of the cookie to make submissions
    // 3) if it does work then add the code to swap creds/cookies on rate limit error
    // 4) if it doesn't work then look into setting up the proxy server

    // tally_statuses();
    // let cqs = get_common_questions(ALL_REAL_LANGS.to_vec());

    // let model = OPENAI_GPT_MODEL;
    // let myslug_tups_cqs = build_all_mytups(get_qs(), ALL_REAL_LANGS.to_vec(), vec![model]);
    // println!("{}", myslug_tups_cqs.len());

    // submit_all_solutions(myslug_tups_cqs).await?;

    let qs = get_qs();
    let langs = ALL_REAL_LANGS.to_vec();
    let ms = vec![OPENAI_GPT_MODEL];

    let start = std::time::Instant::now();
    let _result = build_all_mytups(qs, langs, ms);
    let duration = start.elapsed();

    println!("Time elapsed is: {:?}", duration);

    // let qs = get_qs();
    // let langs = ALL_REAL_LANGS.to_vec();
    // let ms = vec![OPENAI_GPT_MODEL];

    // let start = std::time::Instant::now();
    // let _result = build_all_mytups_noclone(qs, langs, ms);
    // let duration = start.elapsed();

    // println!("Time elapsed noclone is: {:?}", duration);

    // let (slug, lang, model) = &myslug_tups_cqs[0];
    // let x = submit_and_check(&slug, &lang, &model, COOKIES[1])
    //     .await;
    // println!("{:?}", x);
    // let df = get_creds_df().await.unwrap();
    // let (username, password) = get_creds_from_index(&df, 0).await.unwrap();
    // let cookie = get_cookie_string(&username, &password).await.unwrap();
    // get_cookie_string()
    Ok(())
    // Ok(())
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

    #[test]
    fn test_solution_count() {
        let qs = get_qs();
        let cqs = get_common_questions(ALL_REAL_LANGS.to_vec());
        let models = vec![OPENAI_GPT_MODEL];

        let myslug_tups_cqs =
            build_all_mytups(cqs.clone(), ALL_REAL_LANGS.to_vec().clone(), models.clone());
        assert_eq!(myslug_tups_cqs.len(), ALL_REAL_LANGS.len() * cqs.len());

        let all_myslugs =
            build_all_mytups(qs.clone(), ALL_REAL_LANGS.to_vec().clone(), models.clone());
        assert_eq!(all_myslugs.len(), 39031);

        for (slug, lang, model) in myslug_tups_cqs {
            let p = get_solution_fn(&slug, &lang, &model);
            assert!(p.exists());
            let j: Value = read_json(p).unwrap();
            assert!(j.get("error").is_none());
        }
    }
}

// #[bench]
// fn benchmark_myslug_tups_cqs(b: &mut Bencher) {
//     let qs = get_qs();
//     let langs = ALL_REAL_LANGS.to_vec();
//     let ms = vec![OPENAI_GPT_MODEL];
//     b.iter(|| {
//         build_all_mytups(qs.clone(), langs.clone(), ms.clone())
//     });
// }
