
// #[tokio::main]
async fn old_main() -> Result<(), reqwest::Error> {
    // todo run gpt on csharp golang php scala kotlin swift ruby dart elixir racket erlang
    // try doing it async to
    let langs = vec![
        "golang",
        "kotlin",
        "ruby",
        "scala",
        "csharp",
        "php",
        "swift",
        "typescript",
        "dart",
        "elixir",
        "racket",
        "erlang",
    ];
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


// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let langs = vec!["golang", "kotlin", "ruby", "scala", "csharp", "php", "swift", "typescript", "dart", "elixir", "racket", "erlang"];
//     let model = OPENAI_GPT_MODEL;
//     let start_index = 0;

//     let client = reqwest::Client::new();
//     let base = "https://leetcode.com/problems/";

//     let mut headers = HeaderMap::new();
//     headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
//     headers.insert(
//         AUTHORIZATION,
//         HeaderValue::from_str(&format!("Bearer {}", OPENAI_API_KEY))?,
//     );

//     let v: Value =
//         serde_json::from_str(&std::fs::read_to_string("problemset.json").unwrap()).unwrap();

//     let mut tasks = vec![];
//     for lang in langs {
//         let questions = v["data"]["problemsetQuestionList"]["questions"]
//             .as_array()
//             .unwrap()
//             .clone(); // You may need to clone the questions if you can't share ownership between tasks

//         let task = tokio::spawn(async move {
//             let free_questions: Vec<_> = questions
//                 .iter()
//                 .filter(|q| match q["paidOnly"].as_bool() {
//                     Some(paid_only) => {
//                         !paid_only
//                             && has_lang(
//                                 get_code_snippets(&get_code(&get_title_slug(q)).unwrap()).unwrap(),
//                                 lang,
//                             )
//                     }
//                     None => false,
//                 })
//                 .collect();

//             let mut times = vec![];
//             let mut i = 0;
//             for q in free_questions.iter().skip(start_index).progress() {
//                 let title_slug = q["titleSlug"].as_str().unwrap();
//                 let (content, _code) = build_prompt(&title_slug, &lang).unwrap();
//                 let full_prompt = format!(
//                     "{}\n\n{}\n\nWrite out full solution in a markdown codeblock:",
//                     content, _code
//                 );
//                 let v = match fetch_openai_completion(&full_prompt, model).await {
//                     Ok(val) => val,
//                     Err(e) => {
//                         eprintln!("Failed to fetch OpenAI completion: {} {}", e, title_slug);
//                         continue;
//                     }
//                 };
//                 let local = Local::now();
//                 times.push(local);
//                 println!("{}", local.format("%Y-%m-%d %H:%M:%S").to_string());
//                 save_solution(&title_slug, &lang, &v).await.unwrap();
//                 i += 1;
//             }
//         });
//         tasks.push(task);
//     }
//     println!("num tasks: {}", tasks.len());
//     // for task in tasks {
//     //     task.await?;
//     // }

//     Ok(())
// }