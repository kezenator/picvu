var picvu = {};

document.addEventListener('DOMContentLoaded', (event) => {

    var doc_changed = false;
    var submitting = false;

    function setDocChanged() {
        if (!doc_changed) {
            doc_changed = true;

            if (!submitting) {
                document.getElementById('save').classList.add('not-saved');
                document.title = '*'.concat(document.title);
            }
        }
    }

    function clearDocChanged() {
        doc_changed = false;
    }

    picvu.submit = function() {
        clearDocChanged();
        document.getElementById('form').submit();
    };

    picvu.set = function(id, value) {
        setDocChanged();
        document.getElementById(id).value = value;
    }

    picvu.set_and_submit = function(id, value) {
        submitting = true;
        picvu.set(id, value);
        picvu.submit();
    };

    picvu.set_combo = function(id, value, callback) {
        picvu.set('hidden-'.concat(id), value);

        document.getElementById('combo-'.concat(id)).querySelectorAll('a').forEach(option =>
            {
                let option_value = option.getAttribute('value');

                if (option_value === value) {
                    option.classList.add('combo-selected');
                } else {
                    option.classList.remove('combo-selected');
                }

                if (callback) {
                    callback(option, option_value, value);
                }
            });
    }

    picvu.rating_combo = function(option, option_value, cur_value) {
        if (option_value !== '0') {
            let icon = option.querySelector('i');
            if (option_value <= cur_value) {
                option.classList.add('rating-yellow');
                icon.classList.remove('bi-star');
                icon.classList.add('bi-star-fill');
            } else {
                option.classList.remove('rating-yellow');
                icon.classList.remove('bi-star-fill');
                icon.classList.add('bi-star');
            }
        }
    };
    
    picvu.set_combo_and_submit = function(id, value) {
        submitting = true;
        picvu.set_combo(id, value);
        picvu.submit();
    }

    picvu.register_edit = function(id) {
        document.getElementById(id).addEventListener('input', (event) => { setDocChanged(); });
    };

    document.getElementById('form').addEventListener('submit', (event) => { clearDocChanged(); });

    window.addEventListener("beforeunload", function( event ) {
        if (doc_changed) {
            event.returnValue = "\o/";
            event.preventDefault();
        }
    });
});