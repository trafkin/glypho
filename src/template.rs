pub static TEMPLATE: &'static str = r#"<!DOCTYPE html>
<html>
    <head>
        <title>Warp Handlebars template example</title>

        <script type="text/javascript">
            console.log("init");
            var eventSource = new EventSource('sse');

            eventSource.onmessage = function (event) {
                if (event.data !== "false") {
                    document.body.innerHTML = event.data;
                }
            }
        </script>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <style>
            /*! tailwindcss v4.1.11 | MIT License | https://tailwindcss.com */
            @layer properties;
            @layer theme, base, components, utilities;
            @layer theme {
            :root, :host {
                --font-sans: ui-sans-serif, system-ui, sans-serif, 'Apple Color Emoji', 'Segoe UI Emoji', 'Segoe UI Symbol',
                'Noto Color Emoji';
                --font-mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New',
                monospace;
                --color-gray-900: oklch(21% 0.034 264.665);
                --color-black: #000;
                --color-white: #fff;
                --spacing: 0.25rem;
                --container-3xl: 48rem;
                --default-transition-duration: 150ms;
                --default-transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
                --default-font-family: var(--font-sans);
                --default-mono-font-family: var(--font-mono);
            }
            }
            @layer base {
            *, ::after, ::before, ::backdrop, ::file-selector-button {
                box-sizing: border-box;
                margin: 0;
                padding: 0;
                border: 0 solid;
            }
            html, :host {
                line-height: 1.5;
                -webkit-text-size-adjust: 100%;
                tab-size: 4;
                font-family: var(--default-font-family, ui-sans-serif, system-ui, sans-serif, 'Apple Color Emoji', 'Segoe UI Emoji', 'Segoe UI Symbol', 'Noto Color Emoji');
                font-feature-settings: var(--default-font-feature-settings, normal);
                font-variation-settings: var(--default-font-variation-settings, normal);
                -webkit-tap-highlight-color: transparent;
            }
            hr {
                height: 0;
                color: inherit;
                border-top-width: 1px;
            }
            abbr:where([title]) {
                -webkit-text-decoration: underline dotted;
                text-decoration: underline dotted;
            }
            h1, h2, h3, h4, h5, h6 {
                font-size: inherit;
                font-weight: inherit;
            }
            a {
                color: inherit;
                -webkit-text-decoration: inherit;
                text-decoration: inherit;
            }
            b, strong {
                font-weight: bolder;
            }
            code, kbd, samp, pre {
                font-family: var(--default-mono-font-family, ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace);
                font-feature-settings: var(--default-mono-font-feature-settings, normal);
                font-variation-settings: var(--default-mono-font-variation-settings, normal);
                font-size: 1em;
            }
            small {
                font-size: 80%;
            }
            sub, sup {
                font-size: 75%;
                line-height: 0;
                position: relative;
                vertical-align: baseline;
            }
            sub {
                bottom: -0.25em;
            }
            sup {
                top: -0.5em;
            }
            table {
                text-indent: 0;
                border-color: inherit;
                border-collapse: collapse;
            }
            :-moz-focusring {
                outline: auto;
            }
            progress {
                vertical-align: baseline;
            }
            summary {
                display: list-item;
            }
            ol, ul, menu {
                list-style: none;
            }
            img, svg, video, canvas, audio, iframe, embed, object {
                display: block;
                vertical-align: middle;
            }
            img, video {
                max-width: 100%;
                height: auto;
            }
            button, input, select, optgroup, textarea, ::file-selector-button {
                font: inherit;
                font-feature-settings: inherit;
                font-variation-settings: inherit;
                letter-spacing: inherit;
                color: inherit;
                border-radius: 0;
                background-color: transparent;
                opacity: 1;
            }
            :where(select:is([multiple], [size])) optgroup {
                font-weight: bolder;
            }
            :where(select:is([multiple], [size])) optgroup option {
                padding-inline-start: 20px;
            }
            ::file-selector-button {
                margin-inline-end: 4px;
            }
            ::placeholder {
                opacity: 1;
            }
            @supports (not (-webkit-appearance: -apple-pay-button))  or (contain-intrinsic-size: 1px) {
                ::placeholder {
                color: currentcolor;
                @supports (color: color-mix(in lab, red, red)) {
                    color: color-mix(in oklab, currentcolor 50%, transparent);
                }
                }
            }
            textarea {
                resize: vertical;
            }
            ::-webkit-search-decoration {
                -webkit-appearance: none;
            }
            ::-webkit-date-and-time-value {
                min-height: 1lh;
                text-align: inherit;
            }
            ::-webkit-datetime-edit {
                display: inline-flex;
            }
            ::-webkit-datetime-edit-fields-wrapper {
                padding: 0;
            }
            ::-webkit-datetime-edit, ::-webkit-datetime-edit-year-field, ::-webkit-datetime-edit-month-field, ::-webkit-datetime-edit-day-field, ::-webkit-datetime-edit-hour-field, ::-webkit-datetime-edit-minute-field, ::-webkit-datetime-edit-second-field, ::-webkit-datetime-edit-millisecond-field, ::-webkit-datetime-edit-meridiem-field {
                padding-block: 0;
            }
            :-moz-ui-invalid {
                box-shadow: none;
            }
            button, input:where([type='button'], [type='reset'], [type='submit']), ::file-selector-button {
                appearance: button;
            }
            ::-webkit-inner-spin-button, ::-webkit-outer-spin-button {
                height: auto;
            }
            [hidden]:where(:not([hidden='until-found'])) {
                display: none !important;
            }
            }
            @layer utilities {
            .static {
                position: static;
            }
            .container {
                width: 100%;
                @media (width >= 40rem) {
                max-width: 40rem;
                }
                @media (width >= 48rem) {
                max-width: 48rem;
                }
                @media (width >= 64rem) {
                max-width: 64rem;
                }
                @media (width >= 80rem) {
                max-width: 80rem;
                }
                @media (width >= 96rem) {
                max-width: 96rem;
                }
            }
            .mx-auto {
                margin-inline: auto;
            }
            .prose {
                color: var(--tw-prose-body);
                max-width: 65ch;
                :where(p):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 1.25em;
                margin-bottom: 1.25em;
                }
                :where([class~="lead"]):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: var(--tw-prose-lead);
                font-size: 1.25em;
                line-height: 1.6;
                margin-top: 1.2em;
                margin-bottom: 1.2em;
                }
                :where(a):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: var(--tw-prose-links);
                text-decoration: underline;
                font-weight: 500;
                }
                :where(strong):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: var(--tw-prose-bold);
                font-weight: 600;
                }
                :where(a strong):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: inherit;
                }
                :where(blockquote strong):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: inherit;
                }
                :where(thead th strong):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: inherit;
                }
                :where(ol):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                list-style-type: decimal;
                margin-top: 1.25em;
                margin-bottom: 1.25em;
                padding-inline-start: 1.625em;
                }
                :where(ol[type="A"]):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                list-style-type: upper-alpha;
                }
                :where(ol[type="a"]):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                list-style-type: lower-alpha;
                }
                :where(ol[type="A" s]):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                list-style-type: upper-alpha;
                }
                :where(ol[type="a" s]):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                list-style-type: lower-alpha;
                }
                :where(ol[type="I"]):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                list-style-type: upper-roman;
                }
                :where(ol[type="i"]):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                list-style-type: lower-roman;
                }
                :where(ol[type="I" s]):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                list-style-type: upper-roman;
                }
                :where(ol[type="i" s]):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                list-style-type: lower-roman;
                }
                :where(ol[type="1"]):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                list-style-type: decimal;
                }
                :where(ul):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                list-style-type: disc;
                margin-top: 1.25em;
                margin-bottom: 1.25em;
                padding-inline-start: 1.625em;
                }
                :where(ol > li):not(:where([class~="not-prose"],[class~="not-prose"] *))::marker {
                font-weight: 400;
                color: var(--tw-prose-counters);
                }
                :where(ul > li):not(:where([class~="not-prose"],[class~="not-prose"] *))::marker {
                color: var(--tw-prose-bullets);
                }
                :where(dt):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: var(--tw-prose-headings);
                font-weight: 600;
                margin-top: 1.25em;
                }
                :where(hr):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                border-color: var(--tw-prose-hr);
                border-top-width: 1;
                margin-top: 3em;
                margin-bottom: 3em;
                }
                :where(blockquote):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                font-weight: 500;
                font-style: italic;
                color: var(--tw-prose-quotes);
                border-inline-start-width: 0.25rem;
                border-inline-start-color: var(--tw-prose-quote-borders);
                quotes: "\201C""\201D""\2018""\2019";
                margin-top: 1.6em;
                margin-bottom: 1.6em;
                padding-inline-start: 1em;
                }
                :where(blockquote p:first-of-type):not(:where([class~="not-prose"],[class~="not-prose"] *))::before {
                content: open-quote;
                }
                :where(blockquote p:last-of-type):not(:where([class~="not-prose"],[class~="not-prose"] *))::after {
                cowith custom build scripts. Placing a file named build.rs in the root of a package will cause Cargo to compile that script and execute it just before building the package.

ntent: close-quote;
                }
                :where(h1):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: var(--tw-prose-headings);
                font-weight: 800;
                font-size: 2.25em;
                margin-top: 0;
                margin-bottom: 0.8888889em;
                line-height: 1.1111111;
                }
                :where(h1 strong):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                font-weight: 900;
                color: inherit;
                }
                :where(h2):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: var(--tw-prose-headings);
                font-weight: 700;
                font-size: 1.5em;
                margin-top: 2em;
                margin-bottom: 1em;
                line-height: 1.3333333;
                }
                :where(h2 strong):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                font-weight: 800;
                color: inherit;
                }
                :where(h3):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: var(--tw-prose-headings);
                font-weight: 600;
                font-size: 1.25em;
                margin-top: 1.6em;
                margin-bottom: 0.6em;
                line-height: 1.6;
                }
                :where(h3 strong):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                font-weight: 700;
                color: inherit;
                }
                :where(h4):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: var(--tw-prose-headings);
                font-weight: 600;
                margin-top: 1.5em;
                margin-bottom: 0.5em;
                line-height: 1.5;
                }
                :where(h4 strong):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                font-weight: 700;
                color: inherit;
                }
                :where(img):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 2em;
                margin-bottom: 2em;
                }
                :where(picture):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                display: block;
                margin-top: 2em;
                margin-bottom: 2em;
                }
                :where(video):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 2em;
                margin-bottom: 2em;
                }
                :where(kbd):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                font-weight: 500;
                font-family: inherit;
                color: var(--tw-prose-kbd);
                box-shadow: 0 0 0 1px rgb(var(--tw-prose-kbd-shadows) / 10%), 0 3px 0 rgb(var(--tw-prose-kbd-shadows) / 10%);
                font-size: 0.875em;
                border-radius: 0.3125rem;
                padding-top: 0.1875em;
                padding-inline-end: 0.375em;
                padding-bottom: 0.1875em;
                padding-inline-start: 0.375em;
                }
                :where(code):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: var(--tw-prose-code);
                font-weight: 600;
                font-size: 0.875em;
                }
                :where(code):not(:where([class~="not-prose"],[class~="not-prose"] *))::before {
                content: "`";
                }
                :where(code):not(:where([class~="not-prose"],[class~="not-prose"] *))::after {
                content: "`";
                }
                :where(a code):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: inherit;
                }
                :where(h1 code):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: inherit;
                }
                :where(h2 code):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: inherit;
                font-size: 0.875em;
                }
                :where(h3 code):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: inherit;
                font-size: 0.9em;
                }
                :where(h4 code):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: inherit;
                }
                :where(blockquote code):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: inherit;
                }
                :where(thead th code):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: inherit;
                }
                :where(pre):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: var(--tw-prose-pre-code);
                background-color: var(--tw-prose-pre-bg);
                overflow-x: auto;
                font-weight: 400;
                font-size: 0.875em;
                line-height: 1.7142857;
                margin-top: 1.7142857em;
                margin-bottom: 1.7142857em;
                border-radius: 0.375rem;
                padding-top: 0.8571429em;
                padding-inline-end: 1.1428571em;
                padding-bottom: 0.8571429em;
                padding-inline-start: 1.1428571em;
                }
                :where(pre code):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                background-color: transparent;
                border-width: 0;
                border-radius: 0;
                padding: 0;
                font-weight: inherit;
                color: inherit;
                font-size: inherit;
                font-family: inherit;
                line-height: inherit;
                }
                :where(pre code):not(:where([class~="not-prose"],[class~="not-prose"] *))::before {
                content: none;
                }
                :where(pre code):not(:where([class~="not-prose"],[class~="not-prose"] *))::after {
                content: none;
                }
                :where(table):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                width: 100%;
                table-layout: auto;
                margin-top: 2em;
                margin-bottom: 2em;
                font-size: 0.875em;
                line-height: 1.7142857;
                }
                :where(thead):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                border-bottom-width: 1px;
                border-bottom-color: var(--tw-prose-th-borders);
                }
                :where(thead th):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: var(--tw-prose-headings);
                font-weight: 600;
                vertical-align: bottom;
                padding-inline-end: 0.5714286em;
                padding-bottom: 0.5714286em;
                padding-inline-start: 0.5714286em;
                }
                :where(tbody tr):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                border-bottom-width: 1px;
                border-bottom-color: var(--tw-prose-td-borders);
                }
                :where(tbody tr:last-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                border-bottom-width: 0;
                }
                :where(tbody td):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                vertical-align: baseline;
                }
                :where(tfoot):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                border-top-width: 1px;
                border-top-color: var(--tw-prose-th-borders);
                }
                :where(tfoot td):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                vertical-align: top;
                }
                :where(th, td):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                text-align: start;
                }
                :where(figure > *):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 0;
                margin-bottom: 0;
                }
                :where(figcaption):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                color: var(--tw-prose-captions);
                font-size: 0.875em;
                line-height: 1.4285714;
                margin-top: 0.8571429em;
                }
                --tw-prose-body: oklch(37.3% 0.034 259.733);
                --tw-prose-headings: oklch(21% 0.034 264.665);
                --tw-prose-lead: oklch(44.6% 0.03 256.802);
                --tw-prose-links: oklch(21% 0.034 264.665);
                --tw-prose-bold: oklch(21% 0.034 264.665);
                --tw-prose-counters: oklch(55.1% 0.027 264.364);
                --tw-prose-bullets: oklch(87.2% 0.01 258.338);
                --tw-prose-hr: oklch(92.8% 0.006 264.531);
                --tw-prose-quotes: oklch(21% 0.034 264.665);
                --tw-prose-quote-borders: oklch(92.8% 0.006 264.531);
                --tw-prose-captions: oklch(55.1% 0.027 264.364);
                --tw-prose-kbd: oklch(21% 0.034 264.665);
                --tw-prose-kbd-shadows: NaN NaN NaN;
                --tw-prose-code: oklch(21% 0.034 264.665);
                --tw-prose-pre-code: oklch(92.8% 0.006 264.531);
                --tw-prose-pre-bg: oklch(27.8% 0.033 256.848);
                --tw-prose-th-borders: oklch(87.2% 0.01 258.338);
                --tw-prose-td-borders: oklch(92.8% 0.006 264.531);
                --tw-prose-invert-body: oklch(87.2% 0.01 258.338);
                --tw-prose-invert-headings: #fff;
                --tw-prose-invert-lead: oklch(70.7% 0.022 261.325);
                --tw-prose-invert-links: #fff;
                --tw-prose-invert-bold: #fff;
                --tw-prose-invert-counters: oklch(70.7% 0.022 261.325);
                --tw-prose-invert-bullets: oklch(44.6% 0.03 256.802);
                --tw-prose-invert-hr: oklch(37.3% 0.034 259.733);
                --tw-prose-invert-quotes: oklch(96.7% 0.003 264.542);
                --tw-prose-invert-quote-borders: oklch(37.3% 0.034 259.733);
                --tw-prose-invert-captions: oklch(70.7% 0.022 261.325);
                --tw-prose-invert-kbd: #fff;
                --tw-prose-invert-kbd-shadows: 255 255 255;
                --tw-prose-invert-code: #fff;
                --tw-prose-invert-pre-code: oklch(87.2% 0.01 258.338);
                --tw-prose-invert-pre-bg: rgb(0 0 0 / 50%);
                --tw-prose-invert-th-borders: oklch(44.6% 0.03 256.802);
                --tw-prose-invert-td-borders: oklch(37.3% 0.034 259.733);
                font-size: 1rem;
                line-height: 1.75;
                :where(picture > img):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 0;
                margin-bottom: 0;
                }
                :where(li):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 0.5em;
                margin-bottom: 0.5em;
                }
                :where(ol > li):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                padding-inline-start: 0.375em;
                }
                :where(ul > li):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                padding-inline-start: 0.375em;
                }
                :where(.prose > ul > li p):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 0.75em;
                margin-bottom: 0.75em;
                }
                :where(.prose > ul > li > p:first-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 1.25em;
                }
                :where(.prose > ul > li > p:last-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-bottom: 1.25em;
                }
                :where(.prose > ol > li > p:first-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 1.25em;
                }
                :where(.prose > ol > li > p:last-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-bottom: 1.25em;
                }
                :where(ul ul, ul ol, ol ul, ol ol):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 0.75em;
                margin-bottom: 0.75em;
                }
                :where(dl):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 1.25em;
                margin-bottom: 1.25em;
                }
                :where(dd):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 0.5em;
                padding-inline-start: 1.625em;
                }
                :where(hr + *):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 0;
                }
                :where(h2 + *):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 0;
                }
                :where(h3 + *):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 0;
                }
                :where(h4 + *):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 0;
                }
                :where(thead th:first-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                padding-inline-start: 0;
                }
                :where(thead th:last-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                padding-inline-end: 0;
                }
                :where(tbody td, tfoot td):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                padding-top: 0.5714286em;
                padding-inline-end: 0.5714286em;
                padding-bottom: 0.5714286em;
                padding-inline-start: 0.5714286em;
                }
                :where(tbody td:first-child, tfoot td:first-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                padding-inline-start: 0;
                }
                :where(tbody td:last-child, tfoot td:last-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                padding-inline-end: 0;
                }
                :where(figure):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 2em;
                margin-bottom: 2em;
                }
                :where(.prose > :first-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-top: 0;
                }
                :where(.prose > :last-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                margin-bottom: 0;
                }
            }
            .contents {
                display: contents;
            }
            .table {
                display: table;
            }
            .max-w-3xl {
                max-width: var(--container-3xl);
            }
            .border-collapse {
                border-collapse: collapse;
            }
            .resize {
                resize: both;
            }
            .border {
                border-style: var(--tw-border-style);
                border-width: 1px;
            }
            .bg-white {
                background-color: var(--color-white);
            }
            .p-6 {
                padding: calc(var(--spacing) * 6);
            }
            .text-black {
                color: var(--color-black);
            }
            .underline {
                text-decoration-line: underline;
            }
            .outline {
                outline-style: var(--tw-outline-style);
                outline-width: 1px;
            }
            .backdrop-filter {
                -webkit-backdrop-filter: var(--tw-backdrop-blur,) var(--tw-backdrop-brightness,) var(--tw-backdrop-contrast,) var(--tw-backdrop-grayscale,) var(--tw-backdrop-hue-rotate,) var(--tw-backdrop-invert,) var(--tw-backdrop-opacity,) var(--tw-backdrop-saturate,) var(--tw-backdrop-sepia,);
                backdrop-filter: var(--tw-backdrop-blur,) var(--tw-backdrop-brightness,) var(--tw-backdrop-contrast,) var(--tw-backdrop-grayscale,) var(--tw-backdrop-hue-rotate,) var(--tw-backdrop-invert,) var(--tw-backdrop-opacity,) var(--tw-backdrop-saturate,) var(--tw-backdrop-sepia,);
            }
            .transition {
                transition-property: color, background-color, border-color, outline-color, text-decoration-color, fill, stroke, --tw-gradient-from, --tw-gradient-via, --tw-gradient-to, opacity, box-shadow, transform, translate, scale, rotate, filter, -webkit-backdrop-filter, backdrop-filter, display, visibility, content-visibility, overlay, pointer-events;
                transition-timing-function: var(--tw-ease, var(--default-transition-timing-function));
                transition-duration: var(--tw-duration, var(--default-transition-duration));
            }
            .lg\:prose-lg {
                @media (width >= 64rem) {
                font-size: 1.125rem;
                line-height: 1.7777778;
                :where(p):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 1.3333333em;
                    margin-bottom: 1.3333333em;
                }
                :where([class~="lead"]):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    font-size: 1.2222222em;
                    line-height: 1.4545455;
                    margin-top: 1.0909091em;
                    margin-bottom: 1.0909091em;
                }
                :where(blockquote):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 1.6666667em;
                    margin-bottom: 1.6666667em;
                    padding-inline-start: 1em;
                }
                :where(h1):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    font-size: 2.6666667em;
                    margin-top: 0;
                    margin-bottom: 0.8333333em;
                    line-height: 1;
                }
                :where(h2):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    font-size: 1.6666667em;
                    margin-top: 1.8666667em;
                    margin-bottom: 1.0666667em;
                    line-height: 1.3333333;
                }
                :where(h3):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    font-size: 1.3333333em;
                    margin-top: 1.6666667em;
                    margin-bottom: 0.6666667em;
                    line-height: 1.5;
                }
                :where(h4):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 1.7777778em;
                    margin-bottom: 0.4444444em;
                    line-height: 1.5555556;
                }
                :where(img):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 1.7777778em;
                    margin-bottom: 1.7777778em;
                }
                :where(picture):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 1.7777778em;
                    margin-bottom: 1.7777778em;
                }
                :where(picture > img):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 0;
                    margin-bottom: 0;
                }
                :where(video):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 1.7777778em;
                    margin-bottom: 1.7777778em;
                }
                :where(kbd):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    font-size: 0.8888889em;
                    border-radius: 0.3125rem;
                    padding-top: 0.2222222em;
                    padding-inline-end: 0.4444444em;
                    padding-bottom: 0.2222222em;
                    padding-inline-start: 0.4444444em;
                }
                :where(code):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    font-size: 0.8888889em;
                }
                :where(h2 code):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    font-size: 0.8666667em;
                }
                :where(h3 code):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    font-size: 0.875em;
                }
                :where(pre):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    font-size: 0.8888889em;
                    line-height: 1.75;
                    margin-top: 2em;
                    margin-bottom: 2em;
                    border-radius: 0.375rem;
                    padding-top: 1em;
                    padding-inline-end: 1.5em;
                    padding-bottom: 1em;
                    padding-inline-start: 1.5em;
                }
                :where(ol):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 1.3333333em;
                    margin-bottom: 1.3333333em;
                    padding-inline-start: 1.5555556em;
                }
                :where(ul):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 1.3333333em;
                    margin-bottom: 1.3333333em;
                    padding-inline-start: 1.5555556em;
                }
                :where(li):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 0.6666667em;
                    margin-bottom: 0.6666667em;
                }
                :where(ol > li):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    padding-inline-start: 0.4444444em;
                }
                :where(ul > li):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    padding-inline-start: 0.4444444em;
                }
                :where(.lg\:prose-lg > ul > li p):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 0.8888889em;
                    margin-bottom: 0.8888889em;
                }
                :where(.lg\:prose-lg > ul > li > p:first-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 1.3333333em;
                }
                :where(.lg\:prose-lg > ul > li > p:last-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-bottom: 1.3333333em;
                }
                :where(.lg\:prose-lg > ol > li > p:first-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 1.3333333em;
                }
                :where(.lg\:prose-lg > ol > li > p:last-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-bottom: 1.3333333em;
                }
                :where(ul ul, ul ol, ol ul, ol ol):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 0.8888889em;
                    margin-bottom: 0.8888889em;
                }
                :where(dl):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 1.3333333em;
                    margin-bottom: 1.3333333em;
                }
                :where(dt):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 1.3333333em;
                }
                :where(dd):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 0.6666667em;
                    padding-inline-start: 1.5555556em;
                }
                :where(hr):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 3.1111111em;
                    margin-bottom: 3.1111111em;
                }
                :where(hr + *):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 0;
                }
                :where(h2 + *):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 0;
                }
                :where(h3 + *):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 0;
                }
                :where(h4 + *):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 0;
                }
                :where(table):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    font-size: 0.8888889em;
                    line-height: 1.5;
                }
                :where(thead th):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    padding-inline-end: 0.75em;
                    padding-bottom: 0.75em;
                    padding-inline-start: 0.75em;
                }
                :where(thead th:first-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    padding-inline-start: 0;
                }
                :where(thead th:last-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    padding-inline-end: 0;
                }
                :where(tbody td, tfoot td):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    padding-top: 0.75em;
                    padding-inline-end: 0.75em;
                    padding-bottom: 0.75em;
                    padding-inline-start: 0.75em;
                }
                :where(tbody td:first-child, tfoot td:first-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    padding-inline-start: 0;
                }
                :where(tbody td:last-child, tfoot td:last-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    padding-inline-end: 0;
                }
                :where(figure):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 1.7777778em;
                    margin-bottom: 1.7777778em;
                }
                :where(figure > *):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 0;
                    margin-bottom: 0;
                }
                :where(figcaption):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    font-size: 0.8888889em;
                    line-height: 1.5;
                    margin-top: 1em;
                }
                :where(.lg\:prose-lg > :first-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-top: 0;
                }
                :where(.lg\:prose-lg > :last-child):not(:where([class~="not-prose"],[class~="not-prose"] *)) {
                    margin-bottom: 0;
                }
                }
            }
            .dark\:bg-gray-900 {
                @media (prefers-color-scheme: dark) {
                background-color: var(--color-gray-900);
                }
            }
            .dark\:text-white {
                @media (prefers-color-scheme: dark) {
                color: var(--color-white);
                }
            }
            .dark\:prose-invert {
                @media (prefers-color-scheme: dark) {
                --tw-prose-body: var(--tw-prose-invert-body);
                --tw-prose-headings: var(--tw-prose-invert-headings);
                --tw-prose-lead: var(--tw-prose-invert-lead);
                --tw-prose-links: var(--tw-prose-invert-links);
                --tw-prose-bold: var(--tw-prose-invert-bold);
                --tw-prose-counters: var(--tw-prose-invert-counters);
                --tw-prose-bullets: var(--tw-prose-invert-bullets);
                --tw-prose-hr: var(--tw-prose-invert-hr);
                --tw-prose-quotes: var(--tw-prose-invert-quotes);
                --tw-prose-quote-borders: var(--tw-prose-invert-quote-borders);
                --tw-prose-captions: var(--tw-prose-invert-captions);
                --tw-prose-kbd: var(--tw-prose-invert-kbd);
                --tw-prose-kbd-shadows: var(--tw-prose-invert-kbd-shadows);
                --tw-prose-code: var(--tw-prose-invert-code);
                --tw-prose-pre-code: var(--tw-prose-invert-pre-code);
                --tw-prose-pre-bg: var(--tw-prose-invert-pre-bg);
                --tw-prose-th-borders: var(--tw-prose-invert-th-borders);
                --tw-prose-td-borders: var(--tw-prose-invert-td-borders);
                }
            }
            }
            @property --tw-border-style {
            syntax: "*";
            inherits: false;
            initial-value: solid;
            }
            @property --tw-outline-style {
            syntax: "*";
            inherits: false;
            initial-value: solid;
            }
            @property --tw-backdrop-blur {
            syntax: "*";
            inherits: false;
            }
            @property --tw-backdrop-brightness {
            syntax: "*";
            inherits: false;
            }
            @property --tw-backdrop-contrast {
            syntax: "*";
            inherits: false;
            }
            @property --tw-backdrop-grayscale {
            syntax: "*";
            inherits: false;
            }
            @property --tw-backdrop-hue-rotate {
            syntax: "*";
            inherits: false;
            }
            @property --tw-backdrop-invert {
            syntax: "*";
            inherits: false;
            }
            @property --tw-backdrop-opacity {
            syntax: "*";
            inherits: false;
            }
            @property --tw-backdrop-saturate {
            syntax: "*";
            inherits: false;
            }
            @property --tw-backdrop-sepia {
            syntax: "*";
            inherits: false;
            }
            @layer properties {
            @supports ((-webkit-hyphens: none) and (not (margin-trim: inline))) or ((-moz-orient: inline) and (not (color:rgb(from red r g b)))) {
                *, ::before, ::after, ::backdrop {
                --tw-border-style: solid;
                --tw-outline-style: solid;
                --tw-backdrop-blur: initial;
                --tw-backdrop-brightness: initial;
                --tw-backdrop-contrast: initial;
                --tw-backdrop-grayscale: initial;
                --tw-backdrop-hue-rotate: initial;
                --tw-backdrop-invert: initial;
                --tw-backdrop-opacity: initial;
                --tw-backdrop-saturate: initial;
                --tw-backdrop-sepia: initial;
                }
            }
            }
        </style>
    </head>

    <body class="bg-white text-black dark:bg-gray-900 dark:text-white">
        <main class="max-w-3xl mx-auto p-6">
            <article class="prose lg:prose-lg dark:prose-invert">
                {{{ body }}}
            </article>
        </main>
    </body>
</html>"#;
