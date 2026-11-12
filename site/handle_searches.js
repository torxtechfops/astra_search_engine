let search_button = document.getElementById("search_button");
let search_text = document.getElementById("search_text");

search_button.addEventListener("click", function () {
    console.log(search_text);

    let query = search_text.value;
    query = query.replaceAll(" ", "+"); // Also done server side

    let redirect = "search?q=" + query;

    console.log(redirect);

    window.location.href=redirect;
});
