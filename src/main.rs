use std::{collections::BTreeMap, path::PathBuf};

use clap::{Parser, Subcommand};
use futures_util::{FutureExt, StreamExt};
use handlebars::Handlebars;
use tokio::fs;
use warp::Filter;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    commands: Cmds,
}

#[derive(Subcommand, Debug)]
enum Cmds {
    StartServer {
        #[arg(short, long)]
        file: PathBuf,
        #[arg(short, long)]
        port: Option<u16>,
    },

    Compile {
        #[arg(short, long)]
        file: PathBuf,
        output_file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let template = r#"<!DOCTYPE html>
                    <html>
                      <head>
                        <title>Warp Handlebars template example</title>

                        <script type="text/javascript">
                            // use vanilla JS because why not
                            window.addEventListener("load", function() {
                                
                                // create websocket instance
                                var mySocket = new WebSocket("ws://localhost:3030/ws");
                                
                                // add event listener reacting when message is received
                                mySocket.onmessage = function (event) {
                                    var output = document.getElementById("output");
                                    // put text into our output div
                                    output.textContent = event.data;
                                };
                                var form = document.getElementsByClassName("foo");
                                var input = document.getElementById("input");
                                form[0].addEventListener("submit", function (e) {
                                    // on forms submission send input to our server
                                    input_text = input.value;
                                    mySocket.send(input_text);
                                    e.preventDefault()
                                })
                            });
                        </script>


                        <style>
                        /* Reset and base styles */
                        * {
                            margin: 0;
                            padding: 0;
                            box-sizing: border-box;
                        }

                        body {
                            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                            line-height: 1.6;
                            color: #333;
                            background-color: #f5f5f5;
                            max-width: 800px;
                            margin: 20px auto;
                            padding: 40px;
                            border: 2px solid #ddd;
                            border-radius: 8px;
                            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
                            background-color: #fff;
                        }

                        /* Typography */
                        h1, h2, h3, h4, h5, h6 {
                            margin-bottom: 16px;
                            font-weight: 600;
                            line-height: 1.25;
                        }

                        h1 { font-size: 2em; color: #2c3e50; }
                        h2 { font-size: 1.5em; color: #34495e; }
                        h3 { font-size: 1.25em; color: #34495e; }

                        p {
                            margin-bottom: 16px;
                        }

                        /* Links */
                        a {
                            color: #3498db;
                            text-decoration: none;
                        }

                        a:hover {
                            text-decoration: underline;
                        }

                        /* Lists */
                        ul, ol {
                            margin-bottom: 16px;
                            padding-left: 24px;
                        }

                        li {
                            margin-bottom: 4px;
                        }

                        /* Code blocks */
                        pre {
                            background: #f8f9fa;
                            border: 1px solid #e9ecef;
                            border-radius: 6px;
                            padding: 16px;
                            margin-bottom: 16px;
                            overflow-x: auto;
                            font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
                            font-size: 14px;
                            line-height: 1.45;
                        }

                        code {
                            background: #f1f3f4;
                            padding: 2px 6px;
                            border-radius: 3px;
                            font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
                            font-size: 85%;
                        }

                        pre code {
                            background: none;
                            padding: 0;
                            border-radius: 0;
                        }

                        /* Blockquotes */
                        blockquote {
                            border-left: 4px solid #ddd;
                            padding-left: 16px;
                            margin: 16px 0;
                            color: #666;
                            font-style: italic;
                        }

                        /* Tables */
                        table {
                            border-collapse: collapse;
                            width: 100%;
                            margin-bottom: 16px;
                        }

                        th, td {
                            border: 1px solid #ddd;
                            padding: 8px 12px;
                            text-align: left;
                        }

                        th {
                            background-color: #f8f9fa;
                            font-weight: 600;
                        }

                        /* Images */
                        img {
                            max-width: 100%;
                            height: auto;
                            border-radius: 4px;
                            margin: 16px 0;
                        }

                        /* Horizontal rule */
                        hr {
                            border: none;
                            border-top: 1px solid #eee;
                            margin: 24px 0;
                        }

                        /* Content container */
                        .content {
                            animation: fadeIn 0.3s ease-in;
                        }

                        @keyframes fadeIn {
                            from { opacity: 0; }
                            to { opacity: 1; }
                        }

                        /* Live reload status indicator */
                        #live-reload-status {
                            position: fixed;
                            top: 16px;
                            right: 16px;
                            padding: 8px 12px;
                            border-radius: 6px;
                            font-size: 12px;
                            font-family: 'SF Mono', Monaco, monospace;
                            font-weight: 500;
                            z-index: 10000;
                            transition: all 0.3s ease;
                            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
                            backdrop-filter: blur(10px);
                        }

                        #live-reload-status.success {
                            background-color: #27ae60;
                            color: white;
                        }

                        #live-reload-status.error {
                            background-color: #e74c3c;
                            color: white;
                        }

                        #live-reload-status.info {
                            background-color: #3498db;
                            color: white;
                        }

                        /* Responsive design */
                        @media (max-width: 768px) {
                            body {
                                margin: 10px;
                                padding: 20px;
                                font-size: 16px;
                            }

                            h1 { font-size: 1.75em; }
                            h2 { font-size: 1.375em; }
                            h3 { font-size: 1.125em; }

                            pre {
                                padding: 12px;
                                font-size: 13px;
                            }

                            #live-reload-status {
                                top: 12px;
                                right: 12px;
                                font-size: 11px;
                                padding: 6px 10px;
                            }
                        }

                        /* Dark mode support */
                        @media (prefers-color-scheme: dark) {
                            body {
                                background-color: #2d2d2d;
                                color: #e0e0e0;
                                border-color: #555;
                                box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
                            }

                            html {
                                background-color: #1a1a1a;
                            }

                            h1 { color: #ffffff; }
                            h2, h3 { color: #f0f0f0; }

                            a {
                                color: #5dade2;
                            }

                            pre {
                                background: #3a3a3a;
                                border-color: #555;
                                color: #e0e0e0;
                            }

                            code {
                                background: #3a3a3a;
                                color: #e0e0e0;
                            }

                            blockquote {
                                border-left-color: #666;
                                color: #aaa;
                            }

                            th, td {
                                border-color: #555;
                            }

                            th {
                                background-color: #3a3a3a;
                            }

                            hr {
                                border-top-color: #555;
                            }
                        }

                        /* Print styles */
                        @media print {
                            #live-reload-status {
                                display: none;
                            }

                            body {
                                max-width: none;
                                margin: 0;
                                padding: 20px;
                                font-size: 12pt;
                                line-height: 1.4;
                                border: none;
                                box-shadow: none;
                                background-color: white;
                            }

                            h1, h2, h3 {
                                page-break-after: avoid;
                            }

                            pre, blockquote {
                                page-break-inside: avoid;
                            }
                        }

                        </style>
                      </head>
                      <body>
                            {{{body}}}
                                <form class="foo">
        <input id="input"></input>
        <input type="submit"></input>
    </form>
    <div id="output"></div>
                      </body>
                    </html>"#;
    let mut hb = Handlebars::new();
    // register the template
    hb.register_template_string("template.html", template)?;

    match args.commands {
        Cmds::StartServer { file, port } => {
            let contents = fs::read_to_string(file).await?;
            let mut data = BTreeMap::new();
            let body = markdown::to_html(&contents.clone());
            data.insert("body".to_string(), body.clone());

            let render = hb
                .render("template.html", &data)
                .unwrap_or_else(|err| err.to_string());

            let route = warp::path::end().map(move || warp::reply::html(render.clone()));

            let ws = warp::path("ws")
                // The `ws()` filter will prepare the Websocket handshake.
                .and(warp::ws())
                .map(|ws: warp::ws::Ws| {
                    // And then our closure will be called when it completes...
                    ws.on_upgrade(|websocket| {
                        // Just echo all messages back...
                        let (tx, rx) = websocket.split();
                        rx.forward(tx).map(|result| {
                            if let Err(e) = result {
                                eprintln!("websocket error: {:?}", e);
                            }
                        })
                    })
                });

            let port = port.unwrap_or(3030);

            warp::serve(route.or(ws)).run(([127, 0, 0, 1], port)).await
        }
        Cmds::Compile { file, output_file } => todo!(),
    }
    Ok(())
}
