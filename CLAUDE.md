# CLAUDE.md

katrust is a GUI for KataGo, written in Rust, aimed at **absolute beginners** at Go. It is a
hand-written project: the owner writes the code to learn Rust and to keep their programming skills
sharp. Read the "Working with AI here" section before doing anything.

## Goals

1. Learn the basic principles of Rust (ownership, borrowing, enums/`match`, traits, error handling,
   modules, cargo, tests, threads/channels).
2. Build a working KataGo GUI for absolute beginners. **Keep it simple**: fewer features done well
   beats feature parity with KaTrain.

## Requirements

- **x86_64-linux only.** No macOS/Windows code paths, no cross-platform abstractions for their sake.
- **Nix/NixOS first.** The project builds and runs from a flake (`nix develop`, `nix run`).
  Flakes only see files git tracks, so new files must be `git add`ed before Nix sees them.
- **Use available hardware automatically.** The right KataGo backend and settings should be picked
  without the user configuring anything (GPU if present, otherwise CPU; sensible thread counts).
- **UI: smooth, responsive, pretty, modern, intuitive** for native Linux / shell users
  (keyboard-friendly, follows desktop conventions, never blocks the UI thread, idle app uses no CPU).

## Working with AI here

These rules come from the owner and apply to every session.

**Allowed:**
- Explaining Rust concepts, compiler errors, crates, KataGo's protocol, Go rules.
- Reviewing code the owner wrote (be honest and specific; point out non-idiomatic Rust).
- Autocompletion-sized help and generating *simple* algorithms or small helpers.
- Suggesting architecture options and an implementation order, with trade-offs.
- Push back on decisions the author makes, if they are objectively bad or overly complicated.
- Stage changes and suggest commit messages (conventional commits). Stage `CLAUDE.md` changes with
  the current commit; don't suggest separate docs commits for it.
- Suggest test cases.

**Not allowed:**
- Making architectural decisions alone. Present options and a recommendation; the owner decides.
  Record decisions in the "Decisions" section below once made.
- Generating complete features or modules. No large generated diffs, no "LGTM, 10 000 lines nobody
  read". Every piece of generated code must be small enough to read and review in one go.
- Never commit, push or rewrite history.
- **The repository is public.** Nothing committed may leak private information: no names, emails,
  hostnames, home paths, tokens, or details of the owner's machine or setup outside the repo
  (dotfiles, NixOS/Home Manager config, local directories). Check staged changes for this before
  suggesting a commit.

**Default: strict tutor mode.** The owner only learns by thinking and writing the code themselves.
- **No code generation unless explicitly asked for.** By default explain, hint, point to docs, ask
  guiding questions, and review.
- **Before every request**, judge whether the owner underestimates its size. If it's bigger than it
  sounds, say so and suggest how to break it into smaller steps, explaining what each step teaches.
- **After every code generation**, review your own change. If it is larger than the owner could
  reasonably have expected from the request, undo it and explain why, and how to split the task into
  smaller pieces instead.
- When code is written, keep it short and readable, explain anything non-obvious, and name the Rust
  concept behind it.
- Don't silently fix things during a review: point them out and let the owner fix them, unless
  asked to apply the fix.
- Stay within scope; don't add features, abstractions, or dependencies that weren't asked for.

## General Styleguide/Programming concepts

- Use conventional commits
- No `Co-Authored-By` line for AI in commit messages unless an LLM contributed significant code.
  Don't add it to suggested commit messages by default.
- Use semantic versioning (during early development, no version numbers are required.)
- Use linters, formatters before commiting.
- Prefer test driven development.

## Related projects (local)

- `~/proj/katrain`: the original KaTrain (Python). Reference for behaviour and KataGo usage.
- `~/proj/katrain-rs`: a Rust reimplementation that was fully AI-generated ("vibecoded"). Useful as a
  reference for what works (e.g. iced + KataGo on niri/NixOS, flake setup, KataGo version pitfalls),
  and small snippets may be borrowed from it, as long as they stay small and the owner reviews them.
  Never transplant whole modules or features: they weren't written by hand, which is the point here.

Known pitfalls learned there (verify before relying on them):
- nixpkgs' KataGo may be too old for current transformer models (katrain-rs overrides to 1.18.1).
- KataGo's analysis engine (`katago analysis`, JSON over stdin/stdout) is the easiest interface;
  read its output on its own thread so the UI never blocks.
- With iced on niri, the `web-colors` feature caused a 10-bit surface with broken alpha blending.

## Roadmap

Long-term target: at least what katrain-rs does today. Built in small steps, chosen by the owner.

1. **Puzzles (first milestone, used daily):** load SGF problems, display the board, let the user
   play a solution, and give feedback based on the solution tree in the SGF. KataGo isn't needed
   for this, so it's a good place to learn Rust before process/threading complexity.
   - In katrain-rs a move counts as right if a line whose comment starts with "Correct" can be
     reached from it ("Almost correct" stays wrong). Check that this fits the chosen puzzle set.
   - The Go Game Guru collection katrain-rs uses is CC BY-NC-SA 4.0: it must **not** be committed
     here. Fetch it through the flake instead.
2. Later (order to be decided): play vs. KataGo, move feedback/hints, territory/score estimate,
   game review, SGF save.

## Commands

Enter the dev shell with `nix develop`, or through direnv with a local `.envrc` containing
`use flake` (gitignored, each clone creates its own). Enter it once after cloning: its `shellHook`
installs the pre-commit hook.

```sh
cargo run                                   # build and run
cargo fmt                                   # format Rust
cargo clippy --all-targets -- -D warnings   # lint
nix fmt                                     # format Nix files (nixfmt-tree)
nix flake check                             # flake outputs + pre-commit hooks in the sandbox
```

The pre-commit hook (git-hooks.nix: nixfmt, rustfmt, clippy with denyWarnings) is installed by the
dev shell's `shellHook`. `.pre-commit-config.yaml` is generated, so it's gitignored.

`Cargo.toml` enables `missing_docs` and `clippy::missing_panics_doc`: every public item needs a doc
comment, and functions that can panic need a `# Panics` section. Doc tests only where an example helps.

## Decisions

Architectural decisions made by the owner, with a one-line reason each. Keep this up to date.

- Rust toolchain from nixpkgs (no rust-overlay/fenix): simplest, newest Rust not needed.
- Dev environment through direnv: keeps the user's own zsh setup. `.envrc` and `.vscode/` are
  gitignored: per-user setup, not part of the project.
- Editor tooling from the flake (`rust-analyzer`, `RUST_SRC_PATH` in the dev shell): the editor
  uses the same toolchain as cargo; VS Code's bundled rust-analyzer doesn't run on NixOS.
- Formatting: `nixfmt-tree` as flake formatter for Nix, `cargo fmt` for Rust.
- Pre-commit hooks through git-hooks.nix, declared in the flake.
- One package with a library (`src/lib.rs`, the logic) and a thin binary (`src/main.rs`): `pub`
  items aren't dead code, and integration tests in `tests/` become possible.
- Board points are `Point { x: u8, y: u8 }`, origin top-left and 0-based like SGF. A point is just
  coordinates; `Board` decides whether it's valid.
- Points off the board passed to `Board` methods panic: that's a bug in the caller (like `vec[i]`).
  `Result` is for expected failures, such as illegal moves.
- `Board` stores a flat grid `Vec<Option<Stone>>` (one slot per intersection, indexed by `index`),
  not a list of stones: O(1) access, and two stones on one point can't happen.

## Next steps

Puzzle milestone, board logic first (no UI, TDD). Done: `Stone`, `Point`, `Board` (get/set),
`Board::neighbors` (private, `Vec<Point>`, order unspecified; tests sort before comparing),
`Board::group` (flood fill, check on push; panics for empty points; returns `Group = HashSet<Point>`).

1. Liberties of a group: the empty neighbours of its stones, deduplicated in a `HashSet`. Owner
   still decides the contract for empty points (panic or `Option`; not 0, which looks like a capture).
2. Playing a move: captures, illegal moves as `Result` with an error enum.
3. Then SGF parsing and the solution tree. README once there's a product.

Optional later: rewrite `neighbors` with `checked_sub`/`checked_add` + `flatten`/`filter`, or as an
iterator once lifetimes come up (`TODO` in the code).

## Open questions

- GUI toolkit: the owner wants a comparison session first (candidates: iced, gtk4-rs/relm4 with
  libadwaita, egui). Present the trade-offs; don't pick one.
- Crate layout (single crate vs. workspace with a UI-free core).
