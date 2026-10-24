# Adding a subcommand

The CLI is defined in two places:

- [`src/cli.usage.kdl`](../src/cli.usage.kdl) describes the commands, flags, arguments and help text. It is compiled into the binary, and [usage-rs](https://usage.jdx.dev) uses it to parse input and print help.
- [`src/cli.rs`](../src/cli.rs) turns the parsed input into the `Commands` enum and runs the command.

## Spec cheat sheet

```kdl
cmd "name" display_order=9 help="Short help" long_help="Long help" {
    flag "-t --table" help="..."              // on/off flag
    flag "-w <week>" help="..."               // flag with a value
    flag "-r --reason <reason>" help="..."    // long flag with a value
    flag "-l --list" conflicts="edit"         // cannot be combined with --edit
    flag "-w <week>" allow_hyphen_values=#true // accepts values like -1
    arg "<minutes>" help="..."                // required argument ([x] = optional)
}
```

- `display_order` sets the position in the help listing.
- `help` is shown for `-h`, and `long_help` for `--help`.
- A flag is named after its long form (`table`) or, if it has none, after the short letter (`w`). Look values up by that name.
- Booleans are written `#true` / `#false` (KDL v2).

## Steps

Example: `trackrs pause --reason <text>`.

1. **Spec**: add the command to `src/cli.usage.kdl`:

   ```kdl
   cmd "pause" display_order=9 help="Pause tracking" {
       flag "-r --reason <reason>" help="Why you pause"
   }
   ```

2. **Enum**: add a variant to `Commands` in `src/cli.rs`:

   ```rust
   Pause { reason: Option<String> },
   ```

3. **Mapping**: add an arm to the `match out.cmd.name.as_str()` in `Cli::try_parse`:

   ```rust
   "pause" => Commands::Pause {
       reason: string(&out, "reason").map(str::to_owned),
   },
   ```
   Hint: If you miss this step, the command parses, but then fails with `unrecognized subcommand '<name>'` and exit 2. The compiler does not catch this.

4. **Dispatch**: add an arm to `CliExecute::execute`, and write the `invoke_pause` method next to the other `invoke_*` methods:

   ```rust
   Commands::Pause { reason } => self.invoke_pause(reason),
   ```
   Hint: `execute` matches every `Commands` variant, so the compiler reports a missing arm.

5. **Check**: `cargo build && target/debug/trackrs pause --help`, then run the command once.

## Value helpers (`src/cli.rs`)

| Helper                    | Returns          | Use for                                   |
| ------------------------- | ---------------- | ----------------------------------------- |
| `flag(&out, "name")`      | `bool`           | on/off flags                              |
| `string(&out, "name")`    | `Option<&str>`   | flag values and arguments                 |
| `typed::<T>("name", raw)` | `Result<T, _>`   | parsing a value into a number or other type; on failure it prints an error and exits 2 |
| `count(&out, "name")`     | `usize`          | repeatable flags (`-vvv`)                 |

