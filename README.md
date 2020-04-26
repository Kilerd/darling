# Project Darling
a personal diary application.

**this project is stil WIP, and its API would be quite unstable, please do a deep consideration about using it.**

this project is using github gist as persistent backend storage. it aims to store your diary in reliable service and with stateless runtime. which means that you can deploy your darling app anytime and anywhere and any times without considering data-lost and and other considerations.

## Get started
first, you need to generate a github personal access token with gist scope, which Darling uses it for accessing your secret gist to store application data.

then, set this token as environment variable named `GITHUB_TOKEN`, then run command `cargo run`.
