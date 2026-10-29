let url = window.location.href;
let query = url.split("?q=")[1];

query = query.replaceAll("+", " ");
// This does break addition, but can be reproduced on DuckDuckGo

search_text.value = query;

document.title = query + " | Astra";