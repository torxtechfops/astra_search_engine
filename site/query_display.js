let url = window.location.href;
let address = url.split("/")[0];
let query = url.split("?q=")[1];

let warning = document.getElementById("cache_warning");
let cache_info = document.getElementById("cache_info");
let engine_info = document.getElementById("engine_info");
let load_info = document.getElementById("load_info");

query = query.replaceAll("+", " ");
// This does break addition, but can be reproduced on DuckDuckGo

search_text.value = query;

document.title = query + " | Astra";

// TODO Get uncached results from our search engine of choice
fetch(address + "/search_query?" + query).then((response) => {
    switch (response.status) {
        case 200: {
            // Do nothing
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


});

function display_error(status) {
    console.error("Error response from server " + status);

    engine_info.value = "Extra results: ERROR";
    load_info.value = "Extra load time: N/A";

    warning.value = "Failed to load additional search data";
    warning.style.borderColor = "#ac1c1c";
};