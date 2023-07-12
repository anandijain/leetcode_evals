const COOKIE: &str = "csrftoken=U38VtRxr5YfvLADqqgCQvJkSHYwu3X6laeFJqGf464BYKRZ0IGF23hu9DUdQKBJJ; messages=\"12877f56d355501b9812ecfb2d621ba942950006$[[\\\"__json_message\\\",0,25,\\\"Successfully signed in as anandjain.\\\"]]\"; LEETCODE_SESSION=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJfYXV0aF91c2VyX2lkIjoiMzYwNTc2OCIsIl9hdXRoX3VzZXJfYmFja2VuZCI6ImRqYW5nby5jb250cmliLmF1dGguYmFja2VuZHMuTW9kZWxCYWNrZW5kIiwiX2F1dGhfdXNlcl9oYXNoIjoiMjEwNWIxYWQ3ZjNhOWNhMmZiMzc1N2FhNzEzZjQ4MmFiNTMxOWVmNyIsImlkIjozNjA1NzY4LCJlbWFpbCI6ImRhaG1laC5hbG1vc0BnbWFpbC5jb20iLCJ1c2VybmFtZSI6ImFuYW5kamFpbiIsInVzZXJfc2x1ZyI6ImFuYW5kamFpbiIsImF2YXRhciI6Imh0dHBzOi8vYXNzZXRzLmxlZXRjb2RlLmNvbS91c2Vycy9hdmF0YXJzL2F2YXRhcl8xNjc4OTA5MTg5LnBuZyIsInJlZnJlc2hlZF9hdCI6MTY4MDk3NTA3MiwiaXAiOiI1MC4xNzAuNDQuMjEwIiwiaWRlbnRpdHkiOiI3MjNjNTEyNjNjODBmNmJlNzlmYTIxMTkxZWUwYjM4NyIsInNlc3Npb25faWQiOjM3Nzk3NzYzLCJfc2Vzc2lvbl9leHBpcnkiOjEyMDk2MDB9.9HpZ8N5J9Lzfz7LHcNryx9sjIDtshhXLfQ3MSmBMFkE; NEW_PROBLEMLIST_PAGE=1; _dd_s=rum=0&expire=1681147877868";
const TOKEN: &str = "U38VtRxr5YfvLADqqgCQvJkSHYwu3X6laeFJqGf464BYKRZ0IGF23hu9DUdQKBJJ";

async fn submit(slug: &str, lang: &str, soln: &str) -> Result<(), reqwest::Error> {
    // let problem_url = "PROBLEM_URL/submit/";
    // let submission_body_fn = fs::read_to_string("SUBMISSION_BODY_FN").expect("Unable to read file");
    let client = Client::new();

    let mut headers = HeaderMap::new();
    headers.insert("authority", HeaderValue::from_static("leetcode.com"));
    headers.insert("accept", HeaderValue::from_static("*/*"));
    headers.insert(
        "accept-language",
        HeaderValue::from_static("en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7"),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    // Add other headers as needed here...
    headers.insert(USER_AGENT, HeaderValue::from_static());

    let res = client
        .post(problem_url)
        .headers(headers)
        .body(submission_body_fn)
        .send()
        .await?;

    let response_text = res.text().await?;
    fs::write("SUBMISSION_RESPONSE_FN", response_text).expect("Unable to write file");

    println!("Response: {}", response_text);

    Ok(())
}
