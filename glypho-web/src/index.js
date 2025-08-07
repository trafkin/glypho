
fetch('init').then(response => {
  response.text().then(data => {
      const el = document.querySelector("article#markdown");
        el.innerHTML = data;
        Prism.highlightAllUnder(el);
        MathJax.typeset();
  });


});

var eventSource = new EventSource('sse');
eventSource.onmessage = function (event) {
    if (event.data !== "false") {
      const el = document.querySelector("article#markdown");
        el.innerHTML = event.data;
        Prism.highlightAllUnder(el);
        MathJax.typeset();
    }
};


