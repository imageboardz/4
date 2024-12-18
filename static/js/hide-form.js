$(document).ready(function() {
    // Ensure active_page is defined and correct
    if (typeof active_page === 'undefined' || (active_page !== 'index' && active_page !== 'thread')) {
        return;
    }

    let form_el = $('form[name="post"]');
    if (form_el.length === 0) {
        console.warn("No form with name='post' found.");
        return;
    }

    // Determine the message
    let form_msg = (active_page === 'index') ? 'Start a New Thread' : 'Post a Reply';

    // Hide the form initially
    form_el.hide();

    // Add toggle link after the form
    form_el.after(
        '<div id="show-post-form" style="font-size:175%;text-align:center;font-weight:bold; margin-bottom:20px;">' +
        '[<a href="#" style="text-decoration:none">' + form_msg + '</a>]' +
        '</div>'
    );

    // Add some top padding to the form to provide space for the close button
    form_el.css({
        'position': 'relative',
        'padding-top': '50px' // creates space above the top Name field
    });

    // Add a close link inside the form (hidden by default, placed above the top name field)
    form_el.prepend(
        '<div id="close-post-form" style="position:absolute; top:0; left:50%; transform:translateX(-50%); font-size:150%; display:none; background:rgba(255,255,255,0.8); padding:5px 10px; border-radius:5px; box-shadow:0 0 5px rgba(0,0,0,0.3);">' +
        '<a href="#" style="text-decoration:none; font-weight:bold; color:#000;">[x]</a>' +
        '</div>'
    );

    // Show the form and close button, hide the link when clicked
    $('#show-post-form').on('click', function(e) {
        e.preventDefault();
        $(this).hide();
        form_el.show();
        $('#close-post-form').show();
    });

    // Hide the form and show the link when close button is clicked
    $('#close-post-form').on('click', function(e) {
        e.preventDefault();
        $('#close-post-form').hide();
        form_el.hide();
        $('#show-post-form').show();
    });
});
