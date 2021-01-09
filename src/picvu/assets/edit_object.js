document.addEventListener('DOMContentLoaded', (event) => {

    picvu.register_edit('edit-activity');
    picvu.register_edit('edit-title');
    picvu.register_edit('edit-notes');
    picvu.register_edit('edit-location');

    picvu.add_tag = function(name, kind, rating, censor) {
        document.getElementById('hidden-add-tag-name').value = name;
        document.getElementById('hidden-add-tag-kind').value = kind;
        document.getElementById('hidden-add-tag-rating').value = rating;
        document.getElementById('hidden-add-tag-censor').value = censor;
        picvu.submit();
    }

    var add_search_required = false;
    var add_search_in_progress = false;

    function addSearchRequired() {
        add_search_required = true;

        if (!add_search_in_progress)
        {
            add_search_in_progress = true;

            setTimeout(() =>
            {
                add_search_required = false;

                var name = document.getElementById('edit-search-tag-name').value;

                window.fetch('/edit/find_tags?name=' + encodeURIComponent(name))
                    .then(response => response.text())
                    .then(text => addSearchResults(text))
                    .catch((error) => addSearchError());
            },
            300);
        }
    }

    function addSearchResults(data) {
        add_search_in_progress = false;

        document.getElementById('add-tags-search-div').innerHTML = data;

        if (add_search_required) {
            addSearchRequired();
        }
    }

    function addSearchError() {
        add_search_in_progress = false;

        if (add_search_required) {
            addSearchRequired();
        }
    }

    document.getElementById('edit-search-tag-name').addEventListener('input', (event) => { addSearchRequired(); });
    
});