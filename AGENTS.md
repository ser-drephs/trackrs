# Instructions for coding agents

## Commits

- Write commit messages in the Conventional Commits style used in the repository history, for
  example `refactor: split settings into a configuration module`. semantic-release builds the
  version and the changelog from them.
- The messages end up in the changelog, so describe what changed for users, not the internals or
  details of the implementation.
- Keep the body as short as possible, in plain sentences without filler words, for example
  `- split settings into a configuration module`.
- Do not mention in the body that tests were added or changed.
- If the body has only one point, write it as a paragraph instead of a bullet list.
- Keep the summary line at most 120 characters long.
- Do not hard wrap the lines of the body at 80 characters, write each paragraph or list item as one
  line.
- Mark changes to the public API with a `BREAKING CHANGE:` footer, so that a major version is
  released. Before adding the footer, ask the user whether the assumed breaking change really is
  one.
- Do not add a `Co-Authored-By` trailer or any other attribution for the agent.

### Preparing a commit

Do not commit unless you are asked to. When asked for a commit message, prepare it so the user can
adjust it before committing:

1. Stage the changes with `git add`.
2. Write the message to `.git/MERGE_MSG`. `git commit` without `-m` and the source control views
   of IDEs prefill the message from it.
3. Leave the `git commit` to the user.
