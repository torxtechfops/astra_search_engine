let url = window.location.href;
let address = url.split("/")[0];
let query = url.split("?q=")[1];

let warning = document.getElementById("cache_warning");
let cache_info = document.getElementById("cache_info");
let engine_info = document.getElementById("engine_info");
let load_info = document.getElementById("load_info");

let decoder = new TextDecoder('utf-8');

query = query.replaceAll("+", " ");
// This does break addition, but can be reproduced on DuckDuckGo

search_text.value = query;

document.title = query + " | Astra";

// Get uncached results from our search engine of choice
get_search_results();

async function get_search_results() {
    let pre_time = performance.now();
    let response = await fetch(address + "/search_query?" + query);
    let post_time = performance.now();

    load_info.textContent = "Extra load time: " + (post_time - pre_time) + "ms";
    
    ////console.log(response.status);

    switch (response.status) {
        case 200: {
            // Do nothing

            break
        };
        case 500: { // Couldn't parse gateway response
            display_error(500);
            
            break
        };
        case 502: { // Bad gateway
            display_error(502);
            
            break
        };
        case 503: { // Service unavaliable
            display_error(503);
            
            break
        };
        case 0: {
            console.error("Error with fetch request");
            
            display_error("LFE"); // Local fetch error
            
            break
        }
        default: {
            console.error("Unexpected response from server");
            
            display_error("UNKNOWN");
            
            break
        };
    };

    ////console.warn(response.body);

    let reader = response.body.getReader();

    let data_raw = await reader.read();

    let data_string = decoder.decode(data_raw.value);

    ////console.log(data_string);

    let data_json = JSON.parse(data_string)

    engine_info.textContent = "Extra results: " + data_json.length;

    for (let i = 0; i < data_json.length; i++) {
        console.log(data_json[i]);
    };
};

function add_search_result(search_json) {

};

function display_error(status) {
    console.error("Error response from server " + status);

    engine_info.value = "Extra results: ERROR";
    load_info.value = "Extra load time: N/A";

    warning.value = "Failed to load additional search data";
    warning.style.borderColor = "#ac1c1c";
};