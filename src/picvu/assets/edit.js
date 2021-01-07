var submit_funcs = {};

document.addEventListener('DOMContentLoaded', (event) => {

    var doc_changed = false;
    var add_search_required = false;
    var add_search_in_progress = false;

    function setDocChanged() {
        doc_changed = true;
    }

    function clearDocChanged() {
        doc_changed = false;
    }

    submit_funcs.submit = function() {
        clearDocChanged();
        document.getElementById('form').submit();
    }

    submit_funcs.delete_tag = function(tag_id) {
        document.getElementById('hidden-remove-tag-id').value = tag_id;
        clearDocChanged();
        document.getElementById('form').submit();
    }

    submit_funcs.add_tag = function(tag_name) {
        document.getElementById('edit-add-tag-name').value = tag_name;
        clearDocChanged();
        document.getElementById('form').submit();
    }

    submit_funcs.rating = function(rating) {
        document.getElementById('hidden-rating').value = rating;
        clearDocChanged();
        document.getElementById('form').submit();
    }

    submit_funcs.censor = function(censor) {
        document.getElementById('hidden-censor').value = censor;
        clearDocChanged();
        document.getElementById('form').submit();
    }

    submit_funcs.move_to = function(object_id) {
        document.getElementById('hidden-next-object-id').value = object_id;
        clearDocChanged();
        document.getElementById('form').submit();
    }

    function addSearchRequired() {
        add_search_required = true;

        if (!add_search_in_progress)
        {
            add_search_in_progress = true;

            setTimeout(() =>
            {
                add_search_required = false;

                var name = document.getElementById('edit-add-tag-name').value;

                window.fetch('/edit/find_tags?name=' + encodeURIComponent(name))
                    .then(response => response.text())
                    .then(text => addSearchResults(text))
                    .catch((error) => addSearchError());
            },
            100);
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

    document.getElementById('edit-activity').addEventListener('input', (event) => { setDocChanged(); });
    document.getElementById('edit-title').addEventListener('input', (event) => { setDocChanged(); });
    document.getElementById('edit-notes').addEventListener('input', (event) => { setDocChanged(); });
    document.getElementById('edit-location').addEventListener('input', (event) => { setDocChanged(); });

    document.getElementById('form').addEventListener('submit', (event) => { clearDocChanged(); });

    window.addEventListener("beforeunload", function( event ) {
        if (doc_changed) {
            event.returnValue = "\o/";
            event.preventDefault();
        }
    });

    document.getElementById('edit-add-tag-name').addEventListener('input', (event) => { addSearchRequired(); });
});