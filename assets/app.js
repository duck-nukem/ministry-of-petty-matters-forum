htmx.config.includeIndicatorStyles = false;  // disable including styles that would conflict with CSP

document.querySelectorAll('[data-utcdate]').forEach((el) => {
    const utcDate = el.getAttribute('data-utcdate');
    const date = new Date(utcDate);
    el.innerText = date.toLocaleString();
});