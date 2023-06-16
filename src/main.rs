use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, REFERER};
use reqwest::Client;
use serde_json::{json, Value};
use std::fs::create_dir_all;
use std::fs::{write, File};
use std::io::Write;

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

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
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

    // let free_questions: Vec<_> = questions
    //     .iter()
    //     .filter(|q| match q["paidOnly"].as_bool() {
    //         Some(paid_only) => !paid_only,
    //         None => false,
    //     })
    //     .collect();

    // println!("Number of free questions: {}", free_questions.len());
    // assert_eq!( free_questions.len(), 2207);
    Ok(())
}
