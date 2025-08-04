
import "./tex-mml-chtml.js";

MathJax = {
  tex: {
    inlineMath: [['$', '$'], ['\\(', '\\)']]
  }
};


fetch('init').then(response => {
  response.text().then(data => {
      const el = document.querySelector("article#markdown");
        el.innerHTML = data;
        Prism.highlightAllUnder(el);
        MathJax.typeset();
  });


});

console.log("init");
var eventSource = new EventSource('sse');
eventSource.onmessage = function (event) {
    if (event.data !== "false") {
      const el = document.querySelector("article#markdown");
        el.innerHTML = event.data;
        Prism.highlightAllUnder(el);
        MathJax.typeset();
    }
};


