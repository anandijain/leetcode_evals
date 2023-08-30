use polars::prelude::*;

fn build_write_solution_df() -> DataFrame {
    let models = vec![OPENAI_GPT_MODEL];
    let qs = get_qs();
    let myslug_tups: Vec<(String, String, String)> =
        build_all_mytups(qs.clone(), ALL_REAL_LANGS.to_vec().clone(), models.clone());

    let cqs = get_common_questions(ALL_REAL_LANGS.to_vec());
    println!("\nCommon questions: {:#?}", cqs.len());
    let model = models[0];

    let myslug_tups_cqs =
        build_all_mytups(cqs.clone(), ALL_REAL_LANGS.to_vec().clone(), models.clone());
    println!("myslug_tups_cqs: {:#?}", myslug_tups_cqs.len());
    let mut slug_col = Vec::new();
    let mut lang_col = Vec::new();
    let mut model_col = Vec::new();
    let mut completion_tokens_col = Vec::new();
    let mut prompt_tokens_col = Vec::new();
    let mut total_tokens_col = Vec::new();
    let mut num_lang_prefixed_codeblocks_col = Vec::new();
    let mut num_codeblocks_col = Vec::new();

    for (slug, lang, model) in myslug_tups_cqs.iter().progress() {
        let filename = get_solution_fn(&slug, lang, model);
        // println!("{:#?}", filename);
        let soln_json = read_json(filename).unwrap();
        let completion_tokens = soln_json["usage"]["completion_tokens"].as_u64().unwrap();
        let prompt_tokens = soln_json["usage"]["prompt_tokens"].as_u64().unwrap();
        let total_tokens = soln_json["usage"]["total_tokens"].as_u64().unwrap();

        let c = extract_content(&soln_json).unwrap();
        let lp_codeblocks = extract_specific_lang_codeblocks(&c, &lang);
        let codeblocks = extract_codeblocks(&c);
        slug_col.push(slug.to_string());
        lang_col.push(lang.to_string());
        model_col.push(model.to_string());
        completion_tokens_col.push(completion_tokens);
        prompt_tokens_col.push(prompt_tokens);
        total_tokens_col.push(total_tokens);
        num_codeblocks_col.push(codeblocks.len() as u64);
        num_lang_prefixed_codeblocks_col.push(lp_codeblocks.len() as u64);
    }
    let mut df = DataFrame::new(vec![
        Series::new("slug", slug_col),
        Series::new("lang", lang_col),
        Series::new("model", model_col),
        Series::new("completion_tokens", completion_tokens_col),
        Series::new("prompt_tokens", prompt_tokens_col),
        Series::new("total_tokens", total_tokens_col),
        Series::new("num_codeblocks", num_codeblocks_col),
        Series::new(
            "num_lang_prefixed_codeblocks",
            num_lang_prefixed_codeblocks_col,
        ),
    ])
    .unwrap();
    let pfn = "solutions2.csv";
    let mut file = File::create(pfn).expect("could not create file");
    CsvWriter::new(&mut file)
        .has_header(true)
        .with_delimiter(b',')
        .finish(&mut df)
        .unwrap();

    df
}
async fn get_creds_df() -> Result<DataFrame, Box<dyn Error>> {
    let mut schema = Schema::new();
    schema.with_column("username".into(), DataType::Utf8);
    schema.with_column("password".into(), DataType::Utf8);

    let df = CsvReader::from_path("creds.csv")?
        .with_schema(schema.into())
        .has_header(true)
        .finish()?;

    Ok(df)
}

async fn get_creds_from_index(
    df: &DataFrame,
    idx: usize,
) -> Result<(String, String), Box<dyn Error>> {
    let username = df.column("username")?.get(idx).unwrap().to_string();
    let password = df.column("password")?.get(idx).unwrap().to_string();

    Ok((username, password))
}

async fn get_cookie_string_from_index(
    df: &DataFrame,
    idx: usize,
) -> Result<String, Box<dyn Error>> {
    let (username, password) = get_creds_from_index(df, idx).await.unwrap();
    let cookie = get_cookie_string(&username, &password).await?;

    Ok(cookie)
}
