var submit_funcs = {};

document.addEventListener('DOMContentLoaded', (event) => {

    var doc_changed = false;

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

    document.getElementById('edit-activity').addEventListener('input', (event) => { setDocChanged(); });
    document.getElementById('edit-title').addEventListener('input', (event) => { setDocChanged(); });
    document.getElementById('edit-notes').addEventListener('input', (event) => { setDocChanged(); });
    document.getElementById('combo-rating').addEventListener('input', (event) => { setDocChanged(); });
    document.getElementById('combo-censor').addEventListener('input', (event) => { setDocChanged(); });
    document.getElementById('edit-location').addEventListener('input', (event) => { setDocChanged(); });

    document.getElementById('form').addEventListener('submit', (event) => { clearDocChanged(); });

    window.addEventListener("beforeunload", function( event ) {
        if (doc_changed) {
            event.returnValue = "\o/";
            event.preventDefault();
        }
    });
});