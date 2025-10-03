
const el = document.querySelector("article#markdown");
fetch('init').then(response => {
  response.text().then(data => {
        el.innerHTML = data;
        Prism.highlightAllUnder(el);
        MathJax.typeset();
  });
});

