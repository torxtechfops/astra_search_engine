async fn do_search(query: String) -> String {
    let mut final_result = String::new();
    println!("Doing search for: {}", query);
    let api_key = "be172e8e22737f962691f21ffa181bc37b1e85950e37938c0fadb76fce755d71".to_string();
    let res = reqwest::get(format!("https://serpapi.com/search.json?engine=duckduckgo_light&q={}&api_key={}", query, api_key))
        .await
        .expect("Request failed");
    final_result = res.text().await.expect("Failed to read response");

    return final_result;
}