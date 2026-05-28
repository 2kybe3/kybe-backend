use axum::{Extension, extract::State, http::header, response::IntoResponse};
use reqwest::StatusCode;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    db::website_traces::{RequestStatus, WebsiteTrace},
    external::lastfm,
    webserver::{
        RequestContext, TERMINAL_PROMPT, WebServerState, common,
        render::{
            Page, Theme,
            builders::{CodeBlockBuilder, TextBlobBuilder},
            object::Objects,
            user_agent_is_cli,
        },
    },
};

pub async fn root(
    State(state): State<WebServerState>,
    Extension(trace): Extension<Arc<Mutex<WebsiteTrace>>>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    let mut trace = trace.lock().await;
    let theme = Theme::default();

    let playing = if let Some(lastfm) = state.lastfm {
        lastfm.get_playing(Some(&mut trace.data)).await.result
    } else {
        None
    };

    let mut page: Vec<Objects> = vec![
        theme.title("Hello Stranger (also on ").into(),
        theme
            .title(theme.link_colored("i2p", "http://kybe.i2p"))
            .into(),
        theme.title(")\n").into(),
        theme.subtitle("kybe - /dev/urandom stuff\n\n").into(),
    ];

    if !user_agent_is_cli(&ctx.user_agent) {
        page.append(&mut vec![
            CodeBlockBuilder::new(vec![
                theme.terminal_prompt(TERMINAL_PROMPT).into(),
                TextBlobBuilder::new("curl https://kybe.xyz").into(),
            ])
            .into(),
            theme.raw("\n").into(),
        ]);
    };

    page.append(&mut vec![
        theme.title("Currently Listening:\n\n").into(),
        theme
            .label(
                "Playing",
                vec![
					theme
						.link_colored(
							if playing.is_some() {
								"True"
							} else {
								"False"
							},
							"https://metrics.kybe.xyz/public-dashboards/f64d242587e14e2689b22e0ff542a1e9",
						)
						.into(),
                    theme.comment(" (click me)\n").into(),
				],
            )
            .into(),
    ]);

    if let Some(playing) = playing {
        page.append(&mut vec![
            theme
                .label(
                    "Song",
                    vec![
                        theme
                            .link_colored(playing.name.as_str(), &playing.url)
                            .into(),
                        TextBlobBuilder::new(" — ").style(theme.link.clone()).into(), // oh no, a em-dash kek (- looks to short lol)
                        theme
                            .link_colored(
                                format!("{}\n", playing.artist.as_str()).as_str(),
                                &lastfm::artist_url(&playing.artist),
                            )
                            .into(),
                    ],
                )
                .into(),
        ])
    };

    page.append(&mut vec![
        theme.title("\nProjects:\n\n").into(),
        theme
            .label(
                "hexix",
                vec![
                    theme
                        .link_colored("My NixOS Config", &ctx.url("/nix"))
                        .into(),
                    theme.text("\n").into(),
                ],
            )
            .into(),
        theme
            .label(
                "cheat-sh",
                vec![
                    theme
                        .link_colored(
                            "A tiny wrapper for cheat.sh",
                            "https://git.kybe.xyz/2kybe3/cheat-sh",
                        )
                        .into(),
                    theme.text("\n").into(),
                ],
            )
            .into(),
        theme
            .label(
                "kybe-backend",
                vec![
                    theme
                        .link_colored(
                            "This sites source code",
                            "https://git.kybe.xyz/2kybe3/kybe-backend",
                        )
                        .into(),
                    theme.text("\n").into(),
                ],
            )
            .into(),
        theme
            .label(
                "gh-notify-daemon",
                vec![
                    theme
                        .link_colored(
                            "A github notification daemon",
                            "https://git.kybe.xyz/2kybe3/gh-notify-daemon",
                        )
                        .into(),
                    theme.text("\n").into(),
                ],
            )
            .into(),
        theme.title("\nContact:\n\n").into(),
        theme
            .label(
                "PGP",
                vec![
                    theme
                        .link_colored(
                            "4B2067C3BD6D410F13E5 36A343CE43938A3C7A8F\n",
                            &ctx.url("/pgp"),
                        )
                        .into(),
                ],
            )
            .into(),
        theme
            .label(
                "Email",
                vec![
                    theme
                        .link_colored("kybe@kybe.xyz\n", "mailto:kybe@kybe.xyz")
                        .into(),
                ],
            )
            .into(),
        theme
            .label(
                "Matrix",
                vec![
                    theme
                        .link_colored("@kybe:kybe.xyz", "https://matrix.to/#/@kybe:kybe.xyz")
                        .into(),
                    theme.comment(" (preferred)\n").into(),
                ],
            )
            .into(),
        theme.title("\nOther Platforms:\n\n").into(),
        theme
            .label(
                "Git",
                vec![
                    theme
                        .link_colored("2kybe3", "https://git.kybe.xyz/2kybe3")
                        .into(),
                    theme.text(", ").into(),
                    theme
                        .link_colored("renovate", "https://git.kybe.xyz/renovate")
                        .into(),
                    theme.text(" ").into(),
                    theme
                        .link_colored("archive", "https://git.kybe.xyz/archive")
                        .into(),
                    theme.comment(" (most used)\n").into(),
                ],
            )
            .into(),
        theme
            .label(
                "Github",
                vec![
                    theme
                        .link_colored("kybe236", "https://github.com/kybe236")
                        .into(),
                    theme.text(" ").into(),
                    theme
                        .link_colored("2kybe3\n", "https://github.com/2kybe3")
                        .into(),
                ],
            )
            .into(),
        theme
            .label(
                "Codeberg",
                vec![
                    theme
                        .link_colored("kybe\n", "https://codeberg.org/kybe")
                        .into(),
                ],
            )
            .into(),
        theme.title("\nOther Endpoints:\n\n").into(),
        theme
            .label(
                "IP",
                vec![theme.link_colored(ctx.url("/ip\n"), &ctx.url("/ip")).into()],
            )
            .into(),
        theme
            .label(
                "Canvas",
                vec![
                    theme
                        .link_colored(ctx.url("/canvas\n"), &ctx.url("/canvas"))
                        .into(),
                ],
            )
            .into(),
        theme
            .label(
                "Identity",
                vec![
                    theme
                        .link_colored(ctx.url("/ident.txt\n"), &ctx.url("/ident.txt"))
                        .into(),
                ],
            )
            .into(),
        theme
            .label(
                "Nix Config",
                vec![
                    theme
                        .link_colored(ctx.url("/nix\n"), &ctx.url("/nix"))
                        .into(),
                ],
            )
            .into(),
        theme
            .label(
                "Now Listening",
                vec![
                    theme
                        .link_colored(ctx.url("/now\n"), &ctx.url("/now"))
                        .into(),
                ],
            )
            .into(),
    ]);
    page.append(&mut common::footer::footer(trace.trace_id));

    let page = Page::from_iter("/", &state.config, page);

    let mut result = page.render(&ctx.user_agent);

    trace.request_status = RequestStatus::Success;
    trace.status_code = StatusCode::OK.into();

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, result.take_content_type())],
        result.take_data(),
    )
        .into_response()
}
